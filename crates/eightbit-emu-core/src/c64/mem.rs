use crate::error::CoreError;

const RAM: usize = 65536;
const BASIC_LEN: usize = 8192;
const KERNAL_LEN: usize = 8192;
const CHARGEN_LEN: usize = 4096;

pub struct Map {
    ram: Vec<u8>,
    basic: Vec<u8>,
    kernal: Vec<u8>,
    chargen: Vec<u8>,
    color: Vec<u8>,
    ddr: u8,
    port: u8,
    pub vic: [u8; 64],
    pub cia1: CiaPorts,
    pub cia2: CiaPorts,
}

#[derive(Clone, Debug)]
pub struct CiaPorts {
    pub pra: u8,
    pub prb: u8,
    pub ddra: u8,
    pub ddrb: u8,
    pub keys: [[bool; 8]; 8],
}

impl Default for CiaPorts {
    fn default() -> Self {
        Self {
            pra: 0xFF,
            prb: 0xFF,
            ddra: 0,
            ddrb: 0,
            keys: [[false; 8]; 8],
        }
    }
}

impl Map {
    pub fn new() -> Self {
        Self {
            ram: vec![0; RAM],
            basic: vec![0; BASIC_LEN],
            kernal: vec![0; KERNAL_LEN],
            chargen: vec![0; CHARGEN_LEN],
            color: vec![0; 1024],
            ddr: 0x2F,
            port: 0x37,
            vic: [0; 64],
            cia1: CiaPorts::default(),
            cia2: CiaPorts::default(),
        }
    }

    pub fn load_roms(
        &mut self,
        kernal: &[u8],
        basic: &[u8],
        chargen: &[u8],
    ) -> Result<(), CoreError> {
        check_len("kernal", kernal, KERNAL_LEN)?;
        check_len("basic", basic, BASIC_LEN)?;
        check_len("chargen", chargen, CHARGEN_LEN)?;
        self.kernal.copy_from_slice(kernal);
        self.basic.copy_from_slice(basic);
        self.chargen.copy_from_slice(chargen);
        Ok(())
    }

    pub fn chargen_byte(&self, offset: usize) -> u8 {
        self.chargen[offset % CHARGEN_LEN]
    }

    pub fn ram_byte(&self, address: u16) -> u8 {
        self.ram[usize::from(address)]
    }

    pub fn color_byte(&self, offset: usize) -> u8 {
        self.color[offset % 1024] & 0x0F
    }

    pub fn write_ram(&mut self, address: u16, value: u8) {
        self.ram[usize::from(address)] = value;
    }

    pub fn set_key(&mut self, row: u8, col: u8, down: bool) {
        if row < 8 && col < 8 {
            self.cia1.keys[usize::from(row)][usize::from(col)] = down;
        }
    }

    pub fn read(&mut self, address: u16) -> u8 {
        match address {
            0x0000 => self.ddr,
            0x0001 => self.port,
            0xD000..=0xDFFF if io_visible(self.port) => read_io(self, address),
            _ => self.read_banked(address),
        }
    }

    pub fn write(&mut self, address: u16, value: u8) {
        match address {
            0x0000 => self.ddr = value,
            0x0001 => self.port = value,
            0xD000..=0xDFFF if io_visible(self.port) => write_io(self, address, value),
            _ => {
                self.ram[usize::from(address)] = value;
            }
        }
    }

    fn read_banked(&self, address: u16) -> u8 {
        if let Some(value) = rom_read(self, address) {
            return value;
        }
        self.ram[usize::from(address)]
    }
}

fn read_io(map: &Map, address: u16) -> u8 {
    match address {
        0xD000..=0xD3FF => map.vic[usize::from(address & 0x3F)],
        0xD800..=0xDBFF => map.color[usize::from(address - 0xD800)] | 0xF0,
        0xDC00..=0xDCFF => read_cia1(map, address),
        0xDD00..=0xDDFF => read_cia2(map, address),
        _ => 0xFF,
    }
}

fn write_io(map: &mut Map, address: u16, value: u8) {
    match address {
        0xD000..=0xD3FF => map.vic[usize::from(address & 0x3F)] = value,
        0xD800..=0xDBFF => map.color[usize::from(address - 0xD800)] = value & 0x0F,
        0xDC00..=0xDCFF => write_cia1(map, address, value),
        0xDD00..=0xDDFF => write_cia2(map, address, value),
        _ => {}
    }
}
fn check_len(name: &'static str, bytes: &[u8], expected: usize) -> Result<(), CoreError> {
    if bytes.len() == expected {
        return Ok(());
    }
    Err(CoreError::ImageLength {
        name,
        actual: bytes.len(),
    })
}

fn rom_read(map: &Map, address: u16) -> Option<u8> {
    if basic_visible(map.port) && (0xA000..=0xBFFF).contains(&address) {
        return Some(map.basic[usize::from(address - 0xA000)]);
    }
    if kernal_visible(map.port) && (0xE000..=0xFFFF).contains(&address) {
        return Some(map.kernal[usize::from(address - 0xE000)]);
    }
    if char_visible(map.port) && (0xD000..=0xDFFF).contains(&address) {
        return Some(map.chargen[usize::from(address - 0xD000) & 0x0FFF]);
    }
    None
}

fn loram(port: u8) -> bool {
    port & 0x01 != 0
}

fn hiram(port: u8) -> bool {
    port & 0x02 != 0
}

fn charen(port: u8) -> bool {
    port & 0x04 != 0
}

fn basic_visible(port: u8) -> bool {
    loram(port) && hiram(port)
}

fn kernal_visible(port: u8) -> bool {
    hiram(port)
}

fn io_visible(port: u8) -> bool {
    charen(port) && (loram(port) || hiram(port))
}

fn char_visible(port: u8) -> bool {
    !charen(port) && (loram(port) || hiram(port))
}

fn read_cia1(map: &Map, address: u16) -> u8 {
    match address & 0x0F {
        0x00 => map.cia1.pra | !map.cia1.ddra,
        0x01 => keyboard_prb(map),
        0x02 => map.cia1.ddra,
        0x03 => map.cia1.ddrb,
        _ => 0xFF,
    }
}

fn write_cia1(map: &mut Map, address: u16, value: u8) {
    match address & 0x0F {
        0x00 => map.cia1.pra = value,
        0x01 => map.cia1.prb = value,
        0x02 => map.cia1.ddra = value,
        0x03 => map.cia1.ddrb = value,
        _ => {}
    }
}

fn read_cia2(map: &Map, address: u16) -> u8 {
    match address & 0x0F {
        0x00 => map.cia2.pra | !map.cia2.ddra,
        0x01 => map.cia2.prb | !map.cia2.ddrb,
        0x02 => map.cia2.ddra,
        0x03 => map.cia2.ddrb,
        _ => 0xFF,
    }
}

fn write_cia2(map: &mut Map, address: u16, value: u8) {
    match address & 0x0F {
        0x00 => map.cia2.pra = value,
        0x01 => map.cia2.prb = value,
        0x02 => map.cia2.ddra = value,
        0x03 => map.cia2.ddrb = value,
        _ => {}
    }
}

fn keyboard_prb(map: &Map) -> u8 {
    let columns = map.cia1.pra | !map.cia1.ddra;
    let mut rows = 0xFFu8;
    let mut col = 0u8;
    while col < 8 {
        if columns & (1 << col) == 0 {
            rows &= !pressed_row_mask(map, col);
        }
        col += 1;
    }
    rows
}

fn pressed_row_mask(map: &Map, col: u8) -> u8 {
    let mut mask = 0u8;
    let mut row = 0u8;
    while row < 8 {
        if map.cia1.keys[usize::from(row)][usize::from(col)] {
            mask |= 1 << row;
        }
        row += 1;
    }
    mask
}
