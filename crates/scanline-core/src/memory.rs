use crate::error::CoreError;

const ROM_END: u16 = 0x4000;
const RAM_BYTES: usize = 48 * 1024;
const BANK: usize = 16 * 1024;

pub struct Memory {
    ram: Vec<u8>,
    banks: Vec<u8>,
    roms: Vec<u8>,
    banked: bool,
    rom_page: usize,
    c000: usize,
    screen: usize,
}

impl Memory {
    pub fn new() -> Self {
        Self {
            ram: vec![0; RAM_BYTES],
            banks: vec![0; BANK * 8],
            roms: vec![0xFF; BANK * 2],
            banked: false,
            rom_page: 0,
            c000: 0,
            screen: 5,
        }
    }

    pub fn read(&self, address: u16) -> u8 {
        if !self.banked {
            return flat_read(self, address);
        }
        self.mapped_read(address)
    }

    pub fn write(&mut self, address: u16, value: u8) {
        if !self.banked {
            flat_write(self, address, value);
            return;
        }
        self.mapped_write(address, value);
    }

    pub fn read_video(&self, address: u16) -> u8 {
        if !self.banked {
            return self.read(address);
        }
        let offset = usize::from(address.wrapping_sub(ROM_END));
        if offset >= BANK {
            return 0;
        }
        self.banks[self.screen * BANK + offset]
    }

    pub fn load_rom(&mut self, bytes: &[u8], model_128: bool) -> Result<(), CoreError> {
        if !rom_len(bytes.len(), model_128) {
            return Err(CoreError::ImageLength {
                name: "rom",
                actual: bytes.len(),
            });
        }
        self.banked = true;
        self.banks.fill(0);
        self.roms.fill(0xFF);
        self.rom_page = 0;
        self.c000 = 0;
        self.screen = 5;
        copy_rom(&mut self.roms, bytes);
        Ok(())
    }

    pub fn use_banks(&mut self) {
        self.banked = true;
    }

    pub fn page(&mut self, value: u8) {
        if !self.banked {
            return;
        }
        self.rom_page = usize::from((value >> 4) & 1);
        self.c000 = usize::from(value & 7);
        self.screen = if value & 0x08 == 0 { 5 } else { 7 };
    }

    pub fn write_bank(&mut self, bank: u8, bytes: &[u8]) {
        if bank > 7 || bytes.len() != BANK {
            return;
        }
        let start = usize::from(bank) * BANK;
        let mut index = 0;
        while index < BANK {
            self.banks[start + index] = bytes[index];
            index += 1;
        }
    }

    pub fn bank_byte(&self, bank: u8, offset: u16) -> u8 {
        if bank > 7 || usize::from(offset) >= BANK {
            return 0;
        }
        self.banks[usize::from(bank) * BANK + usize::from(offset)]
    }

    fn mapped_read(&self, address: u16) -> u8 {
        if address < ROM_END {
            return self.roms[self.rom_page * BANK + usize::from(address)];
        }
        let (bank, offset) = ram_slot(address, self.c000);
        self.banks[bank * BANK + offset]
    }

    fn mapped_write(&mut self, address: u16, value: u8) {
        if address < ROM_END {
            return;
        }
        let (bank, offset) = ram_slot(address, self.c000);
        self.banks[bank * BANK + offset] = value;
    }
}

fn flat_read(memory: &Memory, address: u16) -> u8 {
    if address < ROM_END {
        return 0xFF;
    }
    memory.ram[flat_index(address)]
}

fn flat_write(memory: &mut Memory, address: u16, value: u8) {
    if address < ROM_END {
        return;
    }
    let index = flat_index(address);
    memory.ram[index] = value;
}

fn flat_index(address: u16) -> usize {
    usize::from(address.wrapping_sub(ROM_END))
}

fn ram_slot(address: u16, c000: usize) -> (usize, usize) {
    if address < 0x8000 {
        return (5, usize::from(address - 0x4000));
    }
    if address < 0xC000 {
        return (2, usize::from(address - 0x8000));
    }
    (c000, usize::from(address - 0xC000))
}

fn rom_len(len: usize, model_128: bool) -> bool {
    if model_128 {
        return len == BANK || len == BANK * 2;
    }
    len == BANK
}

fn copy_rom(roms: &mut [u8], bytes: &[u8]) {
    let mut index = 0;
    while index < bytes.len() {
        roms[index] = bytes[index];
        index += 1;
    }
    if bytes.len() == BANK {
        let mut copy = 0;
        while copy < BANK {
            roms[BANK + copy] = bytes[copy];
            copy += 1;
        }
    }
}
