#[derive(Clone, Debug)]
pub struct Ay {
    regs: [u8; 16],
    selected: u8,
    tone: [u32; 3],
    level: [bool; 3],
    noise_at: u32,
    rng: u32,
    env_at: u32,
    env_level: u8,
}

impl Default for Ay {
    fn default() -> Self {
        Self {
            regs: [0; 16],
            selected: 0,
            tone: [0; 3],
            level: [false; 3],
            noise_at: 0,
            rng: 1,
            env_at: 0,
            env_level: 0,
        }
    }
}

impl Ay {
    pub fn select(&mut self, value: u8) {
        self.selected = value & 0x0F;
    }

    pub fn write(&mut self, value: u8) {
        let index = usize::from(self.selected);
        self.regs[index] = masked(index, value);
    }

    pub fn read(&self) -> u8 {
        self.regs[usize::from(self.selected)]
    }

    pub fn tick(&mut self, clocks: u32) {
        self.tick_tone(clocks);
        self.tick_noise(clocks);
        self.tick_envelope(clocks);
    }

    pub fn sample(&self) -> f32 {
        let mut mix = 0.0f32;
        mix += self.channel(0);
        mix += self.channel(1);
        mix += self.channel(2);
        mix / 3.0
    }

    fn tick_tone(&mut self, clocks: u32) {
        let mut index = 0;
        while index < 3 {
            let period = period_of(self.regs[index * 2], self.regs[index * 2 + 1]);
            self.tone[index] += clocks;
            let flips = self.tone[index] / period;
            self.tone[index] %= period;
            if flips % 2 == 1 {
                self.level[index] = !self.level[index];
            }
            index += 1;
        }
    }

    fn tick_noise(&mut self, clocks: u32) {
        let period = u32::from((self.regs[6] & 0x1F).max(1)).saturating_mul(16);
        self.noise_at += clocks;
        let shifts = self.noise_at / period;
        self.noise_at %= period;
        self.shift_noise(shifts);
    }

    fn shift_noise(&mut self, mut shifts: u32) {
        while shifts > 0 {
            let bit = (self.rng ^ (self.rng >> 3)) & 1;
            self.rng = (self.rng >> 1) | (bit << 16);
            shifts -= 1;
        }
    }

    fn tick_envelope(&mut self, clocks: u32) {
        let period = envelope_period(self.regs[11], self.regs[12]);
        self.env_at += clocks;
        let steps = self.env_at / period;
        self.env_at %= period;
        self.env_level = self.env_level.wrapping_add(steps as u8) & 0x0F;
    }

    fn channel(&self, index: usize) -> f32 {
        let volume = self.volume(index);
        if volume == 0 || !self.audible(index) {
            return 0.0;
        }
        f32::from(volume) / 15.0
    }

    fn volume(&self, index: usize) -> u8 {
        let raw = self.regs[8 + index];
        if raw & 0x10 == 0 { raw & 0x0F } else { self.env_level }
    }

    fn audible(&self, index: usize) -> bool {
        self.tone_open(index) && self.noise_open(index)
    }

    fn tone_open(&self, index: usize) -> bool {
        let enabled = self.regs[7] & (1 << index) == 0;
        !enabled || self.level[index]
    }

    fn noise_open(&self, index: usize) -> bool {
        let enabled = self.regs[7] & (1 << (index + 3)) == 0;
        !enabled || self.rng & 1 == 1
    }
}

fn masked(index: usize, value: u8) -> u8 {
    match index {
        1 | 3 | 5 | 13 => value & 0x0F,
        6 | 8 | 9 | 10 => value & 0x1F,
        7 => value,
        _ => value,
    }
}

fn period_of(low: u8, high: u8) -> u32 {
    let period = (u32::from(high & 0x0F) << 8) | u32::from(low);
    period.max(1).saturating_mul(16)
}

fn envelope_period(low: u8, high: u8) -> u32 {
    let period = (u32::from(high) << 8) | u32::from(low);
    period.max(1).saturating_mul(256)
}

#[cfg(test)]
mod tests {
    use super::Ay;

    #[test]
    fn tone_and_noise_count_sixteen_clocks() {
        let mut tone = chip();
        write(&mut tone, 0, 4);
        write(&mut tone, 7, 0x3E);
        write(&mut tone, 8, 0x0F);
        tone.tick(63);
        assert_eq!(tone.sample(), 0.0);
        tone.tick(1);
        assert!(tone.sample() > 0.3);
    }

    #[test]
    fn noise_shifts_instead_of_toggling() {
        let mut noise = noise_channel(1);
        assert!(noise.sample() > 0.0);
        noise.tick(16);
        assert_eq!(noise.sample(), 0.0);
        noise.tick(16);
        assert_eq!(noise.sample(), 0.0);
        let mut zero = noise_channel(0);
        let mut one = noise_channel(1);
        zero.tick(16);
        one.tick(16);
        assert_eq!(zero.sample(), one.sample());
    }

    #[test]
    fn a_channel_ands_the_tone_with_the_noise() {
        let mut gated = chip();
        write(&mut gated, 0, 1);
        write(&mut gated, 6, 1);
        write(&mut gated, 7, 0x00);
        write(&mut gated, 8, 0x0F);
        gated.tick(16);
        assert_eq!(gated.sample(), 0.0);
        let mut open = chip();
        write(&mut open, 7, 0x07);
        write(&mut open, 8, 0x0F);
        assert!(open.sample() > 0.0);
    }

    #[test]
    fn the_envelope_counts_two_hundred_fifty_six_clocks() {
        let mut ay = chip();
        write(&mut ay, 0, 1);
        write(&mut ay, 7, 0x3E);
        write(&mut ay, 8, 0x10);
        write(&mut ay, 11, 1);
        ay.tick(16);
        assert_eq!(ay.sample(), 0.0);
        write(&mut ay, 0, 255);
        ay.tick(240);
        assert!(ay.sample() > 0.0);
    }

    fn chip() -> Ay {
        Ay::default()
    }

    fn noise_channel(period: u8) -> Ay {
        let mut noise = chip();
        write(&mut noise, 6, period);
        write(&mut noise, 7, 0x07);
        write(&mut noise, 8, 0x0F);
        noise
    }

    fn write(ay: &mut Ay, register: u8, value: u8) {
        ay.select(register);
        ay.write(value);
    }
}
