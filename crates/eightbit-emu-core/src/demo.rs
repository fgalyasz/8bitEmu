use crate::color::PaletteKind;
use crate::frame::{
    Attribute, BORDER, CONTENT_HEIGHT, CONTENT_WIDTH, Content, Frame, Sprite, compose,
};

pub fn demo_frame(tick: u64) -> Frame {
    compose(&demo_content(tick)).expect("demo content matches the frame rules")
}

pub fn demo_sprite_x(tick: u64) -> i16 {
    (tick % 120) as i16 * 2
}

fn demo_content(tick: u64) -> Content {
    let mut content = base_content(tick);
    content.sprites = vec![demo_sprite(tick)];
    content
}

fn base_content(tick: u64) -> Content {
    Content {
        width: CONTENT_WIDTH,
        height: CONTENT_HEIGHT,
        bitmap: demo_bitmap(),
        attributes: demo_attributes(tick),
        flash_on: false,
        border_px: BORDER,
        border: demo_border(),
        sprites: Vec::new(),
        palette: PaletteKind::Ula,
    }
}

fn demo_bitmap() -> Vec<u8> {
    let mut bitmap = vec![0xff; plane_len()];
    paint_checker(&mut bitmap);
    bitmap
}

fn plane_len() -> usize {
    usize::from(CONTENT_WIDTH / 8) * usize::from(CONTENT_HEIGHT)
}

fn paint_checker(bitmap: &mut [u8]) {
    let mut row = 0u16;
    while row < 8 {
        bitmap[checker_offset(row)] = 0xAA;
        row += 1;
    }
}

fn checker_offset(row: u16) -> usize {
    (80 + usize::from(row)) * 32 + 10
}

fn demo_attributes(tick: u64) -> Vec<Attribute> {
    let mut attributes = Vec::new();
    let mut y = 0u16;
    while y < CONTENT_HEIGHT {
        push_attr_row(&mut attributes, y, tick);
        y += 1;
    }
    attributes
}

fn push_attr_row(attributes: &mut Vec<Attribute>, y: u16, tick: u64) {
    let mut x = 0u16;
    while x < CONTENT_WIDTH / 8 {
        attributes.push(attr_at(x, y, tick));
        x += 1;
    }
}

fn attr_at(x: u16, y: u16, tick: u64) -> Attribute {
    if y == 40 {
        return solid(2, 0, false);
    }
    if in_flicker(x, y) {
        return flicker_attr(tick);
    }
    if checker_cell(x, y) {
        return solid(7, 0, true);
    }
    solid(cell_ink(x, y), 0, false)
}

fn in_flicker(x: u16, y: u16) -> bool {
    (4..8).contains(&x) && (16..24).contains(&y)
}

fn checker_cell(x: u16, y: u16) -> bool {
    x == 10 && (80..88).contains(&y)
}

fn flicker_attr(tick: u64) -> Attribute {
    if tick % 2 == 0 {
        solid(1, 0, false)
    } else {
        solid(6, 0, false)
    }
}

fn cell_ink(x: u16, y: u16) -> u8 {
    ((x + y / 8) % 7 + 1) as u8
}

fn solid(ink: u8, paper: u8, bright: bool) -> Attribute {
    Attribute {
        ink,
        paper,
        bright,
        flash: false,
    }
}

fn demo_border() -> Vec<u8> {
    let mut border = vec![0; border_len()];
    paint_border_stripe(&mut border);
    border
}

fn border_len() -> usize {
    usize::from(CONTENT_HEIGHT + BORDER * 2)
}

fn paint_border_stripe(border: &mut [u8]) {
    let mut row = 4u16;
    while row < 10 {
        border[usize::from(row)] = 2;
        row += 1;
    }
}

fn demo_sprite(tick: u64) -> Sprite {
    Sprite {
        x: demo_sprite_x(tick),
        y: 96,
        width: 8,
        height: 8,
        pixels: vec![15; 64],
        luma: vec![1; 64],
    }
}
