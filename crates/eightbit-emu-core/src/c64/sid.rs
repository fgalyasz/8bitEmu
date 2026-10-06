const VOICE_SPAN: usize = 7;
const MODE_VOL: usize = 0x18;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Stage {
    Idle,
    Attack,
    Decay,
    Sustain,
    Release,
}

#[derive(Clone, Debug)]
struct Voice {
    phase: u32,
    noise: u32,
    envelope: u8,
    stage: Stage,
    counter: u32,
    gate: bool,
}

impl Default for Voice {
    fn default() -> Self {
        Self {
            phase: 0,
            noise: 0x7FFFF8,
            envelope: 0,
            stage: Stage::Idle,
            counter: 0,
            gate: false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Sid {
    regs: [u8; 32],
    voices: [Voice; 3],
}

impl Default for Sid {
    fn default() -> Self {
        Self {
            regs: [0; 32],
            voices: [Voice::default(), Voice::default(), Voice::default()],
        }
    }
}

impl Sid {
    pub fn write(&mut self, address: u16, value: u8) {
        let index = usize::from(address & 0x1F);
        self.regs[index] = value;
        maybe_gate(self, index);
    }

    pub fn read(&self, address: u16) -> u8 {
        self.regs[usize::from(address & 0x1F)]
    }

    pub fn tick(&mut self, cycles: u32) {
        let mut index = 0;
        while index < 3 {
            tick_voice(self, index, cycles);
            index += 1;
        }
    }

    pub fn sample(&self) -> f32 {
        let volume = f32::from(self.regs[MODE_VOL] & 0x0F) / 15.0;
        if volume == 0.0 {
            return 0.0;
        }
        let mix = voice_out(self, 0) + voice_out(self, 1) + voice_out(self, 2);
        (mix / 3.0) * volume
    }
}

fn maybe_gate(sid: &mut Sid, index: usize) {
    if index % VOICE_SPAN != 4 || index >= 21 {
        return;
    }
    apply_gate(sid, index / VOICE_SPAN);
}

fn apply_gate(sid: &mut Sid, voice: usize) {
    let gate = sid.regs[voice * VOICE_SPAN + 4] & 0x01 != 0;
    let was = sid.voices[voice].gate;
    sid.voices[voice].gate = gate;
    if gate && !was {
        sid.voices[voice].stage = Stage::Attack;
        sid.voices[voice].counter = 0;
        return;
    }
    if !gate && was {
        sid.voices[voice].stage = Stage::Release;
        sid.voices[voice].counter = 0;
    }
}

fn tick_voice(sid: &mut Sid, index: usize, cycles: u32) {
    let freq = voice_freq(&sid.regs, index);
    sid.voices[index].phase = sid.voices[index].phase.wrapping_add(freq.wrapping_mul(cycles));
    clock_noise(sid, index, cycles);
    advance_envelope(sid, index, cycles);
}

fn voice_freq(regs: &[u8; 32], index: usize) -> u32 {
    let base = index * VOICE_SPAN;
    u32::from(regs[base]) | (u32::from(regs[base + 1]) << 8)
}

fn clock_noise(sid: &mut Sid, index: usize, cycles: u32) {
    let control = sid.regs[index * VOICE_SPAN + 4];
    if control & 0x80 == 0 {
        return;
    }
    let mut left = cycles.min(256);
    while left > 0 {
        sid.voices[index].noise = shift_lfsr(sid.voices[index].noise);
        left -= 1;
    }
}

fn shift_lfsr(state: u32) -> u32 {
    let bit = ((state >> 22) ^ (state >> 17)) & 1;
    ((state << 1) | bit) & 0x7FFFFF
}

fn advance_envelope(sid: &mut Sid, index: usize, cycles: u32) {
    let mut left = cycles;
    while left > 0 {
        if !step_envelope(sid, index) {
            return;
        }
        left -= 1;
    }
}

fn step_envelope(sid: &mut Sid, index: usize) -> bool {
    let stage = sid.voices[index].stage;
    if stage == Stage::Idle || stage == Stage::Sustain {
        return false;
    }
    sid.voices[index].counter += 1;
    let rate = envelope_rate(sid, index, stage);
    if sid.voices[index].counter < rate {
        return true;
    }
    sid.voices[index].counter = 0;
    bump_envelope(sid, index, stage);
    true
}

fn envelope_rate(sid: &Sid, index: usize, stage: Stage) -> u32 {
    let adsr = sid.regs[index * VOICE_SPAN + 5];
    let sr = sid.regs[index * VOICE_SPAN + 6];
    match stage {
        Stage::Attack => rate_table(adsr >> 4),
        Stage::Decay => rate_table(adsr & 0x0F),
        Stage::Release => rate_table(sr & 0x0F),
        _ => 1,
    }
}

fn bump_envelope(sid: &mut Sid, index: usize, stage: Stage) {
    match stage {
        Stage::Attack => raise_attack(sid, index),
        Stage::Decay => lower_decay(sid, index),
        Stage::Release => lower_release(sid, index),
        _ => {}
    }
}

fn raise_attack(sid: &mut Sid, index: usize) {
    if sid.voices[index].envelope >= 255 {
        sid.voices[index].envelope = 255;
        sid.voices[index].stage = Stage::Decay;
        return;
    }
    sid.voices[index].envelope = sid.voices[index].envelope.saturating_add(1);
}

fn lower_decay(sid: &mut Sid, index: usize) {
    let sustain = (sid.regs[index * VOICE_SPAN + 6] >> 4) * 17;
    if sid.voices[index].envelope <= sustain {
        sid.voices[index].envelope = sustain;
        sid.voices[index].stage = Stage::Sustain;
        return;
    }
    sid.voices[index].envelope = sid.voices[index].envelope.saturating_sub(1);
}

fn lower_release(sid: &mut Sid, index: usize) {
    if sid.voices[index].envelope == 0 {
        sid.voices[index].stage = Stage::Idle;
        return;
    }
    sid.voices[index].envelope = sid.voices[index].envelope.saturating_sub(1);
}

fn voice_out(sid: &Sid, index: usize) -> f32 {
    let control = sid.regs[index * VOICE_SPAN + 4];
    let wave = waveform(sid, index, control);
    let level = f32::from(sid.voices[index].envelope) / 255.0;
    wave * level
}

fn waveform(sid: &Sid, index: usize, control: u8) -> f32 {
    let phase = sid.voices[index].phase;
    let bits = combine_waves(sid, index, control, phase);
    match bits {
        None => 0.0,
        Some(value) => (f32::from(value) / 4095.0) * 2.0 - 1.0,
    }
}

fn combine_waves(sid: &Sid, index: usize, control: u8, phase: u32) -> Option<u16> {
    let mut bits = None;
    bits = merge_wave(bits, control & 0x10 != 0, triangle(phase));
    bits = merge_wave(bits, control & 0x20 != 0, saw(phase));
    bits = merge_wave(bits, control & 0x40 != 0, pulse(sid, index, phase));
    bits = merge_wave(
        bits,
        control & 0x80 != 0,
        noise_bits(sid.voices[index].noise),
    );
    bits
}

fn merge_wave(bits: Option<u16>, enabled: bool, wave: u16) -> Option<u16> {
    if !enabled {
        return bits;
    }
    Some(match bits {
        None => wave,
        Some(value) => value & wave,
    })
}

fn triangle(phase: u32) -> u16 {
    let mut value = ((phase >> 11) & 0xFFF) as u16;
    if phase & 0x800000 != 0 {
        value = !value & 0xFFF;
    }
    value
}

fn saw(phase: u32) -> u16 {
    ((phase >> 12) & 0xFFF) as u16
}

fn pulse(sid: &Sid, index: usize, phase: u32) -> u16 {
    let base = index * VOICE_SPAN;
    let width = u32::from(sid.regs[base + 2]) | (u32::from(sid.regs[base + 3] & 0x0F) << 8);
    if ((phase >> 12) & 0xFFF) as u32 >= width {
        0xFFF
    } else {
        0
    }
}

fn noise_bits(state: u32) -> u16 {
    ((((state >> 22) & 1) << 11)
        | (((state >> 20) & 1) << 10)
        | (((state >> 16) & 1) << 9)
        | (((state >> 13) & 1) << 8)
        | (((state >> 11) & 1) << 7)
        | (((state >> 7) & 1) << 6)
        | (((state >> 4) & 1) << 5)
        | (((state >> 2) & 1) << 4)) as u16
}

fn rate_table(nibble: u8) -> u32 {
    const RATES: [u32; 16] = [
        1, 2, 4, 6, 9, 13, 19, 26, 34, 43, 53, 64, 80, 96, 120, 160,
    ];
    RATES[usize::from(nibble & 0x0F)]
}
