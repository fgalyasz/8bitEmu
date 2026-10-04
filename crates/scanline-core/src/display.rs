use crate::color::PaletteKind;
use crate::error::CoreError;
use crate::frame::{Attribute, BORDER, CONTENT_HEIGHT, CONTENT_WIDTH, Content, Frame, compose};
use crate::memory::Memory;

pub fn bitmap_address(x: u16, y: u16) -> u16 {
    let band = (y & 0x00C0) << 5;
    let row = (y & 0x0007) << 8;
    let cell = (y & 0x0038) << 2;
    0x4000 | band | row | cell | (x >> 3)
}

pub fn attribute_address(x: u16, y: u16) -> u16 {
    0x5800 + (y / 8) * 32 + (x / 8)
}

pub fn frame_from(memory: &Memory, border: u8) -> Result<Frame, CoreError> {
    frame_with_border(memory, &fill_border(border & 7))
}

pub fn frame_with_border(memory: &Memory, rows: &[u8]) -> Result<Frame, CoreError> {
    compose(&content_from(memory, rows))
}

fn content_from(memory: &Memory, rows: &[u8]) -> Content {
    Content {
        width: CONTENT_WIDTH,
        height: CONTENT_HEIGHT,
        bitmap: bitmap(memory),
        attributes: attributes(memory),
        flash_on: false,
        border_px: BORDER,
        border: rows.to_vec(),
        sprites: Vec::new(),
        palette: PaletteKind::Ula,
    }
}

fn bitmap(memory: &Memory) -> Vec<u8> {
    let mut bytes = Vec::new();
    let mut y = 0u16;
    while y < CONTENT_HEIGHT {
        push_bitmap_row(&mut bytes, memory, y);
        y += 1;
    }
    bytes
}

fn push_bitmap_row(bytes: &mut Vec<u8>, memory: &Memory, y: u16) {
    let mut x = 0u16;
    while x < CONTENT_WIDTH {
        bytes.push(memory.read_video(bitmap_address(x, y)));
        x += 8;
    }
}

fn attributes(memory: &Memory) -> Vec<Attribute> {
    let mut rows = Vec::new();
    let mut y = 0u16;
    while y < CONTENT_HEIGHT {
        push_attribute_row(&mut rows, memory, y);
        y += 1;
    }
    rows
}

fn push_attribute_row(rows: &mut Vec<Attribute>, memory: &Memory, y: u16) {
    let mut x = 0u16;
    while x < CONTENT_WIDTH {
        rows.push(decode_attribute(memory.read_video(attribute_address(x, y))));
        x += 8;
    }
}

fn decode_attribute(value: u8) -> Attribute {
    Attribute {
        ink: value & 7,
        paper: (value >> 3) & 7,
        bright: value & 0x40 != 0,
        flash: value & 0x80 != 0,
    }
}

fn fill_border(color: u8) -> Vec<u8> {
    let mut rows = Vec::new();
    let height = CONTENT_HEIGHT + BORDER * 2;
    let mut y = 0u16;
    while y < height {
        rows.push(color);
        y += 1;
    }
    rows
}
