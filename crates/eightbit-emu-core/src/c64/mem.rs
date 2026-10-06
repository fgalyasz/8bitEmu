use crate::error::CoreError;

use super::sid::Sid;
use super::sprites;

const RAM: usize = 65536;
const BASIC_LEN: usize = 8192;
const KERNAL_LEN: usize = 8192;
const CHARGEN_LEN: usize = 4096;
const LINE_CYCLES: u32 = 63;
const RASTER_LINES: u16 = 312;

#[derive(Clone, Copy, Debug)]
pub struct SpriteDraw {
    pub x: i32,
    pub y: u8,
    pub base: u16,
    pub color: u8,
    pub mc1: u8,
    pub mc2: u8,
    pub multi: bool,
    pub x_exp: bool,
    pub y_exp: bool,
}

pub struct Map {
    ram: Vec<u8>,
    basic: Vec<u8>,
    kernal: Vec<u8>,
    chargen: Vec<u8>,
    color: Vec<u8>,
    ddr: u8,
    port: u8,
    pub vic: [u8; 64],
    pub sid: Sid,
    vic_irr: u8,
    pub raster_y: u16,
    line_cycle: u32,
    pub sprite_draws: Vec<SpriteDraw>,
    pub cia1: Cia1,
    pub cia2: CiaPorts,
}

#[derive(Clone, Debug)]
pub struct CiaPorts {
    pub pra: u8,
    pub prb: u8,
    pub ddra: u8,
    pub ddrb: u8,
}

impl Default for CiaPorts {
    fn default() -> Self {
        Self {
            pra: 0xFF,
            prb: 0xFF,
            ddra: 0,
            ddrb: 0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Cia1 {
    pub pra: u8,
    pub prb: u8,
    pub ddra: u8,
    pub ddrb: u8,
    pub keys: [[bool; 8]; 8],
    ta: u16,
    ta_latch: u16,
    cra: u8,
    icr: u8,
    icr_mask: u8,
}

impl Default for Cia1 {
    fn default() -> Self {
        Self {
            pra: 0xFF,
            prb: 0xFF,
            ddra: 0,
            ddrb: 0,
            keys: [[false; 8]; 8],
            ta: 0,
            ta_latch: 0,
            cra: 0,
            icr: 0,
            icr_mask: 0,
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
            sid: Sid::default(),
            vic_irr: 0,
            raster_y: 0,
            line_cycle: 0,
            sprite_draws: Vec::new(),
            cia1: Cia1::default(),
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

    pub fn vic_bank(&self) -> u16 {
        u16::from((!self.cia2.pra) & 0x03) << 14
    }

    pub fn video_matrix(&self) -> u16 {
        self.vic_bank() + (u16::from(self.vic[0x18] >> 4) << 10)
    }

    pub fn bitmap_base(&self) -> u16 {
        self.vic_bank() + if self.vic[0x18] & 0x08 != 0 {
            0x2000
        } else {
            0
        }
    }

    pub fn glyph_byte(&self, code: u8, line: u8) -> u8 {
        let offset = u16::from(code) * 8 + u16::from(line);
        let base = charset_base(self);
        if charset_is_rom(self, base) {
            return self.chargen_byte(usize::from(offset));
        }
        self.ram_byte(base + offset)
    }

    pub fn set_key(&mut self, row: u8, col: u8, down: bool) {
        if row < 8 && col < 8 {
            self.cia1.keys[usize::from(row)][usize::from(col)] = down;
        }
    }

    pub fn irq_line(&self) -> bool {
        let cia = self.cia1.icr & 0x80 != 0;
        let vic = self.vic_irr & self.vic[0x1A] & 0x0F != 0;
        cia || vic
    }

    pub fn tick(&mut self, cycles: u32) {
        advance_raster(self, cycles);
        tick_timer_a(&mut self.cia1, cycles);
        self.sid.tick(cycles);
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

fn advance_raster(map: &mut Map, cycles: u32) {
    map.line_cycle += cycles;
    while map.line_cycle >= LINE_CYCLES {
        map.line_cycle -= LINE_CYCLES;
        map.raster_y += 1;
        if map.raster_y >= RASTER_LINES {
            map.raster_y = 0;
        }
        if map.raster_y == 1 {
            map.sprite_draws.clear();
        }
        maybe_raster_irq(map);
        sprites::latch_line(map);
    }
}

fn maybe_raster_irq(map: &mut Map) {
    if map.raster_y != raster_compare(map) {
        return;
    }
    map.vic_irr |= 0x01;
}

fn raster_compare(map: &Map) -> u16 {
    (u16::from(map.vic[0x11] >> 7) << 8) | u16::from(map.vic[0x12])
}

fn charset_base(map: &Map) -> u16 {
    map.vic_bank() + (u16::from((map.vic[0x18] >> 1) & 0x07) << 11)
}

fn charset_is_rom(map: &Map, base: u16) -> bool {
    let bank = map.vic_bank();
    if bank != 0x0000 && bank != 0x8000 {
        return false;
    }
    let offset = base - bank;
    offset == 0x1000 || offset == 0x1800
}

fn tick_timer_a(cia: &mut Cia1, cycles: u32) {
    if cia.cra & 0x01 == 0 {
        return;
    }
    let mut left = cycles;
    while left > 0 {
        if cia.ta == 0 {
            underflow_timer_a(cia);
            if cia.cra & 0x01 == 0 {
                return;
            }
            left -= 1;
            continue;
        }
        if u32::from(cia.ta) > left {
            cia.ta -= left as u16;
            return;
        }
        left -= u32::from(cia.ta);
        cia.ta = 0;
        underflow_timer_a(cia);
        if cia.cra & 0x01 == 0 {
            return;
        }
    }
}

fn underflow_timer_a(cia: &mut Cia1) {
    cia.ta = cia.ta_latch;
    cia.icr |= 0x01;
    if cia.icr_mask & 0x01 != 0 {
        cia.icr |= 0x80;
    }
    if cia.cra & 0x08 != 0 {
        cia.cra &= !0x01;
    }
}

fn read_io(map: &mut Map, address: u16) -> u8 {
    match address {
        0xD000..=0xD3FF => read_vic(map, address),
        0xD400..=0xD7FF => map.sid.read(address),
        0xD800..=0xDBFF => map.color[usize::from(address - 0xD800)] | 0xF0,
        0xDC00..=0xDCFF => read_cia1(map, address),
        0xDD00..=0xDDFF => read_cia2(map, address),
        _ => 0xFF,
    }
}

fn write_io(map: &mut Map, address: u16, value: u8) {
    match address {
        0xD000..=0xD3FF => write_vic(map, address, value),
        0xD400..=0xD7FF => map.sid.write(address, value),
        0xD800..=0xDBFF => map.color[usize::from(address - 0xD800)] = value & 0x0F,
        0xDC00..=0xDCFF => write_cia1(map, address, value),
        0xDD00..=0xDDFF => write_cia2(map, address, value),
        _ => {}
    }
}

fn write_vic(map: &mut Map, address: u16, value: u8) {
    let reg = address & 0x3F;
    if reg == 0x19 {
        map.vic_irr &= !value;
        return;
    }
    map.vic[usize::from(reg)] = value;
}

fn read_vic(map: &Map, address: u16) -> u8 {
    match address & 0x3F {
        0x11 => (map.vic[0x11] & 0x7F) | (((map.raster_y >> 8) as u8) << 7),
        0x12 => map.raster_y as u8,
        0x19 => read_vic_irr(map),
        reg => map.vic[usize::from(reg)],
    }
}

fn read_vic_irr(map: &Map) -> u8 {
    let pending = map.vic_irr & map.vic[0x1A] & 0x0F;
    let bit7 = if pending != 0 { 0x80 } else { 0 };
    map.vic_irr | bit7
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

fn read_cia1(map: &mut Map, address: u16) -> u8 {
    match address & 0x0F {
        0x00 => map.cia1.pra | !map.cia1.ddra,
        0x01 => keyboard_prb(map),
        0x02 => map.cia1.ddra,
        0x03 => map.cia1.ddrb,
        0x04 => map.cia1.ta as u8,
        0x05 => (map.cia1.ta >> 8) as u8,
        0x0D => {
            let value = map.cia1.icr;
            map.cia1.icr = 0;
            value
        }
        0x0E => map.cia1.cra,
        _ => 0xFF,
    }
}

fn write_cia1(map: &mut Map, address: u16, value: u8) {
    match address & 0x0F {
        0x00 => map.cia1.pra = value,
        0x01 => map.cia1.prb = value,
        0x02 => map.cia1.ddra = value,
        0x03 => map.cia1.ddrb = value,
        0x04 => {
            map.cia1.ta_latch = (map.cia1.ta_latch & 0xFF00) | u16::from(value);
        }
        0x05 => {
            map.cia1.ta_latch = (map.cia1.ta_latch & 0x00FF) | (u16::from(value) << 8);
            if map.cia1.cra & 0x01 == 0 {
                map.cia1.ta = map.cia1.ta_latch;
            }
        }
        0x0D => write_icr_mask(&mut map.cia1, value),
        0x0E => write_cra(&mut map.cia1, value),
        _ => {}
    }
}

fn write_icr_mask(cia: &mut Cia1, value: u8) {
    let bits = value & 0x1F;
    if value & 0x80 != 0 {
        cia.icr_mask |= bits;
    } else {
        cia.icr_mask &= !bits;
    }
}

fn write_cra(cia: &mut Cia1, value: u8) {
    let start = value & 0x01 != 0 && cia.cra & 0x01 == 0;
    cia.cra = value;
    if start {
        cia.ta = cia.ta_latch;
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
