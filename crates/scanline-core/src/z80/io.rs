use crate::memory::Memory;
use crate::z80::Ports;

impl Ports {
    pub fn input(&mut self, port: u16) -> u8 {
        if port & 0x00FF == 0x00FE {
            self.fe_reads = self.fe_reads.saturating_add(1);
            return self.ear(self.keys((port >> 8) as u8));
        }
        if port & 0x00FF == 0x001F {
            return self.kempston;
        }
        if self.ay_selected(port) {
            return self.ay.read();
        }
        0xFF
    }

    pub fn output(&mut self, memory: &mut Memory, port: u16, value: u8) {
        if port & 0x00FF == 0x00FE {
            self.border = value & 7;
            self.mic = value & 0x08 != 0;
            self.speaker = (value >> 4) & 1;
        }
        self.page(memory, port, value);
        self.ay_write(port, value);
    }

    pub fn set_stick(&mut self, mask: u8, down: bool) {
        if down {
            self.kempston |= mask;
            return;
        }
        self.kempston &= !mask;
    }

    pub fn set_key(&mut self, row: u8, mask: u8, down: bool) {
        if row > 7 {
            return;
        }
        let index = usize::from(row);
        if down {
            self.pressed[index] |= mask;
            return;
        }
        self.pressed[index] &= !mask;
    }

    fn keys(&self, high: u8) -> u8 {
        let mut line = 0x1Fu8;
        let mut row = 0u8;
        while row < 8 {
            if high & (1 << row) == 0 {
                line &= !self.pressed[usize::from(row)] & 0x1F;
            }
            row += 1;
        }
        line | 0xE0
    }

    fn ear(&self, line: u8) -> u8 {
        if !self.tape_on {
            return line;
        }
        if self.ear_high {
            return line | 0x40;
        }
        line & !0x40
    }

    fn page(&mut self, memory: &mut Memory, port: u16, value: u8) {
        if !self.model_128 || self.locked || port & 0x8002 != 0 {
            return;
        }
        memory.page(value);
        if value & 0x20 != 0 {
            self.locked = true;
        }
    }

    fn ay_selected(&self, port: u16) -> bool {
        self.model_128 && port & 0xC002 == 0xC000
    }

    fn ay_write(&mut self, port: u16, value: u8) {
        if !self.model_128 || port & 0x8002 != 0x8000 {
            return;
        }
        if port & 0x4000 != 0 {
            self.ay.select(value);
            return;
        }
        self.ay.write(value);
    }
}
