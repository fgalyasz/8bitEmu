#[derive(Clone, Debug)]
pub struct Ay {
    regs: [u8; 16],
    selected: u8,
    tone: [u32; 3],
    level: [bool; 3],
    noise_at: u32,
    noise: bool,
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
            noise: false,
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
        let period = u32::from((self.regs[6] & 0x1F).max(1));
        self.noise_at += clocks;
        let flips = self.noise_at / period;
        self.noise_at %= period;
        if flips % 2 == 1 {
            self.noise = !self.noise;
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
        let mixer = self.regs[7];
        let tone_on = mixer & (1 << index) == 0 && self.level[index];
        let noise_on = mixer & (1 << (index + 3)) == 0 && self.noise;
        tone_on || noise_on
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
    period.max(1)
}

fn envelope_period(low: u8, high: u8) -> u32 {
    let period = (u32::from(high) << 8) | u32::from(low);
    period.max(1)
}
