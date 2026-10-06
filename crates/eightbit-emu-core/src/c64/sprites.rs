use super::mem::Map;
use super::paint::{HEIGHT, WIDTH};

const ORIGIN_X: i32 = 24;
const ORIGIN_Y: i32 = 50;

pub fn paint(map: &Map, pixels: &mut [u8]) {
    let enable = map.vic[0x15];
    let mut index = 0u8;
    while index < 8 {
        if enable & (1 << index) != 0 {
            blit_sprite(map, pixels, index);
        }
        index += 1;
    }
}

fn blit_sprite(map: &Map, pixels: &mut [u8], index: u8) {
    let data = sprite_bytes(map, index);
    let origin = sprite_origin(map, index);
    let flags = sprite_flags(map, index);
    draw_pattern(map, pixels, index, &data, origin, flags);
}

fn sprite_flags(map: &Map, index: u8) -> (bool, bool, bool) {
    let bit = 1 << index;
    (
        map.vic[0x1C] & bit != 0,
        map.vic[0x1D] & bit != 0,
        map.vic[0x17] & bit != 0,
    )
}

fn draw_pattern(
    map: &Map,
    pixels: &mut [u8],
    index: u8,
    data: &[u8; 63],
    origin: (i32, i32),
    flags: (bool, bool, bool),
) {
    let mut row = 0u16;
    while row < 21 {
        paint_logic_row(map, pixels, index, data, origin, row, flags);
        row += 1;
    }
}

fn paint_logic_row(
    map: &Map,
    pixels: &mut [u8],
    index: u8,
    data: &[u8; 63],
    origin: (i32, i32),
    row: u16,
    flags: (bool, bool, bool),
) {
    let base = usize::from(row) * 3;
    let y = origin.1 + i32::from(row) * if flags.2 { 2 } else { 1 };
    emit_row(map, pixels, index, &data[base..base + 3], origin.0, y, flags);
    if flags.2 {
        emit_row(map, pixels, index, &data[base..base + 3], origin.0, y + 1, flags);
    }
}

fn emit_row(
    map: &Map,
    pixels: &mut [u8],
    index: u8,
    row: &[u8],
    x0: i32,
    y: i32,
    flags: (bool, bool, bool),
) {
    if flags.0 {
        put_multi(map, pixels, index, row, x0, y, flags.1);
        return;
    }
    put_hires(map, pixels, index, row, x0, y, flags.1);
}

fn put_hires(map: &Map, pixels: &mut [u8], index: u8, row: &[u8], x0: i32, y: i32, x_exp: bool) {
    let color = map.vic[0x27 + usize::from(index)] & 0x0F;
    put_bits(pixels, row[0], x0, y, color, x_exp, 0);
    put_bits(pixels, row[1], x0, y, color, x_exp, 8);
    put_bits(pixels, row[2], x0, y, color, x_exp, 16);
}

fn put_multi(map: &Map, pixels: &mut [u8], index: u8, row: &[u8], x0: i32, y: i32, x_exp: bool) {
    put_pairs(map, pixels, index, row[0], x0, y, x_exp, 0);
    put_pairs(map, pixels, index, row[1], x0, y, x_exp, 8);
    put_pairs(map, pixels, index, row[2], x0, y, x_exp, 16);
}

fn put_bits(pixels: &mut [u8], byte: u8, x0: i32, y: i32, color: u8, x_exp: bool, offset: i32) {
    let mut bit = 0i32;
    while bit < 8 {
        if byte & (0x80 >> bit) != 0 {
            plot_pixel(pixels, x0, y, color, x_exp, offset + bit);
        }
        bit += 1;
    }
}

fn put_pairs(
    map: &Map,
    pixels: &mut [u8],
    index: u8,
    byte: u8,
    x0: i32,
    y: i32,
    x_exp: bool,
    offset: i32,
) {
    let mut pair = 0i32;
    while pair < 4 {
        paint_pair(map, pixels, index, byte, x0, y, x_exp, offset, pair);
        pair += 1;
    }
}

fn paint_pair(
    map: &Map,
    pixels: &mut [u8],
    index: u8,
    byte: u8,
    x0: i32,
    y: i32,
    x_exp: bool,
    offset: i32,
    pair: i32,
) {
    let code = (byte >> (6 - pair * 2)) & 0x03;
    let Some(color) = multi_color(map, index, code) else {
        return;
    };
    plot_pixel(pixels, x0, y, color, x_exp, offset + pair * 2);
    plot_pixel(pixels, x0, y, color, x_exp, offset + pair * 2 + 1);
}

fn multi_color(map: &Map, index: u8, code: u8) -> Option<u8> {
    match code {
        1 => Some(map.vic[0x25] & 0x0F),
        2 => Some(map.vic[0x27 + usize::from(index)] & 0x0F),
        3 => Some(map.vic[0x26] & 0x0F),
        _ => None,
    }
}

fn plot_pixel(pixels: &mut [u8], x0: i32, y: i32, color: u8, x_exp: bool, logical_x: i32) {
    let x = if x_exp { x0 + logical_x * 2 } else { x0 + logical_x };
    plot(pixels, x, y, color);
    if x_exp {
        plot(pixels, x + 1, y, color);
    }
}

fn plot(pixels: &mut [u8], x: i32, y: i32, color: u8) {
    if x < 0 || y < 0 || x >= i32::from(WIDTH) || y >= i32::from(HEIGHT) {
        return;
    }
    let index = usize::from(y as u16) * usize::from(WIDTH) + usize::from(x as u16);
    pixels[index] = color;
}

fn sprite_origin(map: &Map, index: u8) -> (i32, i32) {
    let x = sprite_x(map, index) - ORIGIN_X;
    let y = i32::from(map.vic[usize::from(index) * 2 + 1]) - ORIGIN_Y;
    (x, y)
}

fn sprite_x(map: &Map, index: u8) -> i32 {
    let low = i32::from(map.vic[usize::from(index) * 2]);
    if map.vic[0x10] & (1 << index) != 0 {
        low + 256
    } else {
        low
    }
}

fn sprite_bytes(map: &Map, index: u8) -> [u8; 63] {
    let pointer = map.ram_byte(pointer_address(map, index));
    let base = vic_bank(map) + u16::from(pointer) * 64;
    fill_sprite(map, base)
}

fn fill_sprite(map: &Map, base: u16) -> [u8; 63] {
    let mut data = [0u8; 63];
    let mut offset = 0u16;
    while offset < 63 {
        data[usize::from(offset)] = map.ram_byte(base + offset);
        offset += 1;
    }
    data
}

fn pointer_address(map: &Map, index: u8) -> u16 {
    screen_base(map) + 0x03F8 + u16::from(index)
}

fn screen_base(map: &Map) -> u16 {
    vic_bank(map) + (u16::from(map.vic[0x18] >> 4) << 10)
}

fn vic_bank(map: &Map) -> u16 {
    u16::from((!map.cia2.pra) & 0x03) << 14
}
