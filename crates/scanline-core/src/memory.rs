const ROM_END: u16 = 0x4000;
const RAM_BYTES: usize = 48 * 1024;

pub struct Memory {
    ram: Vec<u8>,
}

impl Memory {
    pub fn new() -> Self {
        Self {
            ram: vec![0; RAM_BYTES],
        }
    }

    pub fn read(&self, address: u16) -> u8 {
        if address < ROM_END {
            return 0xFF;
        }
        self.ram[ram_index(address)]
    }

    pub fn write(&mut self, address: u16, value: u8) {
        if address < ROM_END {
            return;
        }
        let index = ram_index(address);
        self.ram[index] = value;
    }
}

fn ram_index(address: u16) -> usize {
    usize::from(address.wrapping_sub(ROM_END))
}
