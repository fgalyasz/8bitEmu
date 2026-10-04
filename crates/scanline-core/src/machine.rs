use crate::display::frame_from;
use crate::error::CoreError;
use crate::frame::Frame;
use crate::memory::Memory;
use crate::program::{self, ENTRY};
use crate::sna;
use crate::tape::{self, Player, Recorder};
use crate::z80::{self, Cpu, Ports};

const STEP_LIMIT: u32 = 100_000;
const TURBO_FRAMES: u32 = 32;
const FRAME_CYCLES: u32 = 69_888;
const SAMPLE_RATE: u32 = 48_000;
const CPU_CLOCK: u32 = 3_500_000;

pub struct Machine {
    cpu: Cpu,
    memory: Memory,
    ports: Ports,
    booted: bool,
    audio: Vec<f32>,
    audio_acc: u32,
    player: Option<Player>,
    recorder: Recorder,
    ear_live: bool,
    loading_sound: bool,
}

impl Machine {
    pub fn new() -> Self {
        let mut memory = Memory::new();
        program::load(&mut memory);
        let mut cpu = Cpu::default();
        cpu.pc = ENTRY;
        Self {
            cpu,
            memory,
            ports: Ports::default(),
            booted: false,
            audio: Vec::new(),
            audio_acc: 0,
            player: None,
            recorder: Recorder::default(),
            ear_live: false,
            loading_sound: true,
        }
    }

    pub fn frame(&mut self) -> Result<Frame, CoreError> {
        if self.booted {
            self.run_cpu_frame()?;
        } else {
            run_until_halt(&mut self.cpu, &mut self.memory, &mut self.ports)?;
        }
        frame_from(&self.memory, self.ports.border)
    }

    pub fn read(&self, address: u16) -> u8 {
        self.memory.read(address)
    }

    pub fn bank_byte(&self, bank: u8, offset: u16) -> u8 {
        self.memory.bank_byte(bank, offset)
    }

    pub fn border(&self) -> u8 {
        self.ports.border
    }

    pub fn take_audio(&mut self) -> Vec<f32> {
        std::mem::take(&mut self.audio)
    }

    pub fn load_rom(&mut self, bytes: &[u8], model_128: bool) -> Result<(), CoreError> {
        self.memory.load_rom(bytes, model_128)?;
        self.cpu = Cpu::default();
        self.ports = Ports::default();
        self.ports.model_128 = model_128;
        self.booted = true;
        self.ear_live = false;
        Ok(())
    }

    pub fn load_sna(&mut self, bytes: &[u8]) -> Result<(), CoreError> {
        sna::load(&mut self.cpu, &mut self.memory, &mut self.ports, bytes)?;
        self.booted = true;
        self.ear_live = false;
        Ok(())
    }

    pub fn load_tape(&mut self, bytes: &[u8]) -> Result<(), CoreError> {
        self.player = Some(tape::open_tape(bytes, self.ports.model_128)?);
        self.ear_live = false;
        Ok(())
    }

    pub fn tape_at_start(&self) -> bool {
        self.player.as_ref().is_none_or(Player::at_start)
    }

    pub fn tape_edge(&self) -> usize {
        self.player.as_ref().map(Player::edge_index).unwrap_or(0)
    }

    pub fn set_loading_sound(&mut self, on: bool) {
        self.loading_sound = on;
    }

    pub fn loading_sound(&self) -> bool {
        self.loading_sound
    }

    pub fn turbo_budget(&self) -> u32 {
        if self.loading_sound || !self.loader_active() {
            return 0;
        }
        TURBO_FRAMES
    }

    pub fn hears_loading(&self) -> bool {
        self.loading_sound || !self.loader_active()
    }

    fn loader_active(&self) -> bool {
        self.ear_live && self.player.as_ref().is_some_and(Player::playing)
    }

    pub fn has_tape(&self) -> bool {
        self.player.is_some()
    }

    pub fn is_booted(&self) -> bool {
        self.booted
    }

    pub fn resume_tape(&mut self) {
        if let Some(player) = &mut self.player {
            player.resume();
        }
    }

    pub fn snapshot(&self) -> Result<Vec<u8>, CoreError> {
        sna::save(&self.cpu, &self.memory, &self.ports)
    }

    pub fn take_tap(&mut self) -> Option<Vec<u8>> {
        self.recorder.take_tap()
    }

    pub fn tap_bytes(&self) -> Option<Vec<u8>> {
        self.recorder.tap_bytes()
    }

    pub fn tzx_bytes(&self) -> Option<Vec<u8>> {
        self.recorder.tzx_bytes()
    }

    pub fn set_stick(&mut self, mask: u8, down: bool) {
        self.ports.set_stick(mask, down);
    }

    pub fn set_key(&mut self, row: u8, mask: u8, down: bool) {
        self.ports.set_key(row, mask, down);
    }

    pub fn reset(&mut self) {
        if !self.booted {
            *self = Self::new();
            return;
        }
        self.cpu = Cpu::default();
        self.ports.locked = false;
        self.ports.border = 0;
        self.ports.speaker = 0;
        self.ports.mic = false;
        self.memory.page(0);
        if let Some(player) = &mut self.player {
            player.rewind();
        }
        self.ear_live = false;
    }

    fn run_cpu_frame(&mut self) -> Result<(), CoreError> {
        self.audio.clear();
        self.audio_acc = 0;
        if self.cpu.iff1 && !self.cpu.arm_ei {
            let took = self.clocked(true)?;
            let _ = took;
        }
        let mut done = 0u32;
        while done < FRAME_CYCLES {
            let took = self.clocked(false)?;
            self.retire_ei();
            done += took;
        }
        if self.ports.fe_reads >= 64 {
            self.ear_live = true;
        }
        self.ports.fe_reads = 0;
        Ok(())
    }

    fn clocked(&mut self, interrupt: bool) -> Result<u32, CoreError> {
        let mic = self.ports.mic;
        self.show_ear();
        let took = if interrupt {
            z80::accept_interrupt(&mut self.cpu, &mut self.memory)
        } else {
            self.step_or_halt()?
        };
        self.roll_tape(took);
        self.recorder.advance(took, mic);
        self.mix(took);
        Ok(took)
    }

    fn show_ear(&mut self) {
        let playing = self.player.as_ref().is_some_and(Player::playing);
        self.ports.tape_on = playing;
        if let Some(player) = &self.player {
            self.ports.ear_high = player.ear_high();
        }
    }

    fn roll_tape(&mut self, cycles: u32) {
        if !self.ear_live {
            return;
        }
        if let Some(player) = &mut self.player {
            player.advance(cycles);
        }
    }

    fn step_or_halt(&mut self) -> Result<u32, CoreError> {
        if self.cpu.halted {
            return Ok(4);
        }
        z80::step(&mut self.cpu, &mut self.memory, &mut self.ports)
    }

    fn retire_ei(&mut self) {
        if !self.cpu.arm_ei {
            return;
        }
        self.cpu.iff1 = true;
        self.cpu.iff2 = true;
        self.cpu.arm_ei = false;
    }

    fn mix(&mut self, cycles: u32) {
        let beeper = if self.ports.speaker == 0 { -0.2 } else { 0.2 };
        let tone = self.ports.ay.sample();
        self.ports.ay.tick(cycles / 2);
        let mixed = (beeper + tone).clamp(-1.0, 1.0);
        self.audio_acc += cycles.saturating_mul(SAMPLE_RATE);
        let count = self.audio_acc / CPU_CLOCK;
        self.audio_acc %= CPU_CLOCK;
        push_samples(&mut self.audio, mixed, count);
    }
}

pub fn run_until_halt(cpu: &mut Cpu, memory: &mut Memory, ports: &mut Ports) -> Result<(), CoreError> {
    resume(cpu);
    let mut steps = 0u32;
    while !cpu.halted {
        step_bounded(cpu, memory, ports, steps)?;
        steps += 1;
    }
    Ok(())
}

fn step_bounded(
    cpu: &mut Cpu,
    memory: &mut Memory,
    ports: &mut Ports,
    steps: u32,
) -> Result<(), CoreError> {
    if steps == STEP_LIMIT {
        return Err(CoreError::StepLimit);
    }
    z80::step(cpu, memory, ports)?;
    Ok(())
}

fn resume(cpu: &mut Cpu) {
    if !cpu.halted {
        return;
    }
    cpu.halted = false;
    cpu.pc = cpu.pc.wrapping_add(1);
}

fn push_samples(audio: &mut Vec<f32>, sample: f32, count: u32) {
    let mut index = 0u32;
    while index < count {
        audio.push(sample);
        index += 1;
    }
}
