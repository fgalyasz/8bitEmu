use super::mem::{Map, SpriteDraw};
use super::paint::{HEIGHT, WIDTH};

const ORIGIN_X: i32 = 24;
const ORIGIN_Y: i32 = 50;

pub fn paint(map: &Map, pixels: &mut [u8]) {
    if map.sprite_draws.is_empty() {
        paint_live(map, pixels);
        return;
    }
    let mut index = 0usize;
    while index < map.sprite_draws.len() {
        blit_draw(map, pixels, &map.sprite_draws[index]);
        index += 1;
    }
}

pub fn latch_line(map: &mut Map) {
    if map.raster_y > 255 {
        return;
    }
    let line = map.raster_y as u8;
    let enable = map.vic[0x15];
    let mut index = 0u8;
    while index < 8 {
        if should_latch(map, index, line, enable) {
            map.sprite_draws.push(capture(map, index));
        }
        index += 1;
    }
}

fn should_latch(map: &Map, index: u8, line: u8, enable: u8) -> bool {
    if enable & (1 << index) == 0 {
        return false;
    }
    if map.vic[usize::from(index) * 2 + 1] != line {
        return false;
    }
    map.ram_byte(pointer_address(map, index)) != 0
}

fn capture(map: &Map, index: u8) -> SpriteDraw {
    let bit = 1 << index;
    let pointer = map.ram_byte(pointer_address(map, index));
    SpriteDraw {
        x: sprite_x(map, index),
        y: map.vic[usize::from(index) * 2 + 1],
        base: map.vic_bank() + u16::from(pointer) * 64,
        color: map.vic[0x27 + usize::from(index)] & 0x0F,
        mc1: map.vic[0x25] & 0x0F,
        mc2: map.vic[0x26] & 0x0F,
        multi: map.vic[0x1C] & bit != 0,
        x_exp: map.vic[0x1D] & bit != 0,
        y_exp: map.vic[0x17] & bit != 0,
    }
}

fn paint_live(map: &Map, pixels: &mut [u8]) {
    let enable = map.vic[0x15];
    let mut index = 0u8;
    while index < 8 {
        if enable & (1 << index) != 0 {
            blit_draw(map, pixels, &capture(map, index));
        }
        index += 1;
    }
}

fn blit_draw(map: &Map, pixels: &mut [u8], draw: &SpriteDraw) {
    let data = fill_sprite(map, draw.base);
    let origin = (draw.x - ORIGIN_X, i32::from(draw.y) - ORIGIN_Y);
    let flags = (draw.multi, draw.x_exp, draw.y_exp);
    draw_pattern(map, pixels, draw, &data, origin, flags);
}

fn draw_pattern(
    map: &Map,
    pixels: &mut [u8],
    draw: &SpriteDraw,
    data: &[u8; 63],
    origin: (i32, i32),
    flags: (bool, bool, bool),
) {
    let mut row = 0u16;
    while row < 21 {
        paint_logic_row(map, pixels, draw, data, origin, row, flags);
        row += 1;
    }
}

fn paint_logic_row(
    map: &Map,
    pixels: &mut [u8],
    draw: &SpriteDraw,
    data: &[u8; 63],
    origin: (i32, i32),
    row: u16,
    flags: (bool, bool, bool),
) {
    let base = usize::from(row) * 3;
    let y = origin.1 + i32::from(row) * if flags.2 { 2 } else { 1 };
    emit_row(map, pixels, draw, &data[base..base + 3], origin.0, y, flags);
    if flags.2 {
        emit_row(map, pixels, draw, &data[base..base + 3], origin.0, y + 1, flags);
    }
}

fn emit_row(
    map: &Map,
    pixels: &mut [u8],
    draw: &SpriteDraw,
    row: &[u8],
    x0: i32,
    y: i32,
    flags: (bool, bool, bool),
) {
    let _ = map;
    if flags.0 {
        put_multi(pixels, draw, row, x0, y, flags.1);
        return;
    }
    put_hires(pixels, row, x0, y, draw.color, flags.1);
}

fn put_hires(pixels: &mut [u8], row: &[u8], x0: i32, y: i32, color: u8, x_exp: bool) {
    put_bits(pixels, row[0], x0, y, color, x_exp, 0);
    put_bits(pixels, row[1], x0, y, color, x_exp, 8);
    put_bits(pixels, row[2], x0, y, color, x_exp, 16);
}

fn put_multi(pixels: &mut [u8], draw: &SpriteDraw, row: &[u8], x0: i32, y: i32, x_exp: bool) {
    put_pairs(pixels, draw, row[0], x0, y, x_exp, 0);
    put_pairs(pixels, draw, row[1], x0, y, x_exp, 8);
    put_pairs(pixels, draw, row[2], x0, y, x_exp, 16);
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
    pixels: &mut [u8],
    draw: &SpriteDraw,
    byte: u8,
    x0: i32,
    y: i32,
    x_exp: bool,
    offset: i32,
) {
    let mut pair = 0i32;
    while pair < 4 {
        paint_pair(pixels, draw, byte, x0, y, x_exp, offset, pair);
        pair += 1;
    }
}

fn paint_pair(
    pixels: &mut [u8],
    draw: &SpriteDraw,
    byte: u8,
    x0: i32,
    y: i32,
    x_exp: bool,
    offset: i32,
    pair: i32,
) {
    let code = (byte >> (6 - pair * 2)) & 0x03;
    let Some(color) = multi_color(draw, code) else {
        return;
    };
    plot_pixel(pixels, x0, y, color, x_exp, offset + pair * 2);
    plot_pixel(pixels, x0, y, color, x_exp, offset + pair * 2 + 1);
}

fn multi_color(draw: &SpriteDraw, code: u8) -> Option<u8> {
    match code {
        1 => Some(draw.mc1),
        2 => Some(draw.color),
        3 => Some(draw.mc2),
        _ => None,
    }
}

fn plot_pixel(pixels: &mut [u8], x0: i32, y: i32, color: u8, x_exp: bool, logical_x: i32) {
    let x = if x_exp {
        x0 + logical_x * 2
    } else {
        x0 + logical_x
    };
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

fn sprite_x(map: &Map, index: u8) -> i32 {
    let low = i32::from(map.vic[usize::from(index) * 2]);
    if map.vic[0x10] & (1 << index) != 0 {
        low + 256
    } else {
        low
    }
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
    map.video_matrix() + 0x03F8 + u16::from(index)
}
