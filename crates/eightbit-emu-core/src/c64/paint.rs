use crate::color::PaletteKind;
use crate::error::CoreError;
use crate::frame::{Frame, frame_from_indexes};

use super::mem::Map;
use super::sprites;

pub const WIDTH: u16 = 320;
pub const HEIGHT: u16 = 200;
pub const BORDER_PX: u16 = 32;

pub fn frame(map: &Map) -> Result<Frame, CoreError> {
    let border = map.vic[0x20] & 0x0F;
    let background = map.vic[0x21] & 0x0F;
    let pixels = raster(map, background);
    let rows = border_rows(border);
    frame_from_indexes(WIDTH, HEIGHT, BORDER_PX, &rows, &pixels, PaletteKind::C64)
}

fn border_rows(color: u8) -> Vec<u8> {
    let height = HEIGHT + BORDER_PX * 2;
    vec![color; usize::from(height)]
}

fn raster(map: &Map, background: u8) -> Vec<u8> {
    let mut pixels = vec![background; usize::from(WIDTH) * usize::from(HEIGHT)];
    if map.vic[0x11] & 0x20 != 0 {
        paint_bitmap(map, &mut pixels, background);
    } else {
        paint_text(map, &mut pixels, background);
    }
    sprites::paint(map, &mut pixels);
    pixels
}

fn paint_text(map: &Map, pixels: &mut [u8], background: u8) {
    let mut row = 0u16;
    while row < 25 {
        paint_text_row(map, pixels, row, background);
        row += 1;
    }
}

fn paint_text_row(map: &Map, pixels: &mut [u8], row: u16, background: u8) {
    let mut col = 0u16;
    while col < 40 {
        paint_text_cell(map, pixels, row, col, background);
        col += 1;
    }
}

fn paint_text_cell(map: &Map, pixels: &mut [u8], row: u16, col: u16, background: u8) {
    let offset = usize::from(row) * 40 + usize::from(col);
    let code = map.ram_byte(map.video_matrix() + offset as u16);
    let color = map.color_byte(offset);
    let mut line = 0u8;
    while line < 8 {
        blit_glyph_line(map, pixels, row, col, line, code, color, background);
        line += 1;
    }
}

fn blit_glyph_line(
    map: &Map,
    pixels: &mut [u8],
    row: u16,
    col: u16,
    line: u8,
    code: u8,
    color: u8,
    background: u8,
) {
    let glyph = map.glyph_byte(code, line);
    let y = row * 8 + u16::from(line);
    let mut bit = 0u16;
    while bit < 8 {
        let x = col * 8 + bit;
        let on = glyph & (0x80 >> bit) != 0;
        put(pixels, x, y, if on { color } else { background });
        bit += 1;
    }
}

fn paint_bitmap(map: &Map, pixels: &mut [u8], background: u8) {
    let multi = map.vic[0x16] & 0x10 != 0;
    let mut row = 0u16;
    while row < 25 {
        paint_bitmap_row(map, pixels, row, background, multi);
        row += 1;
    }
}

fn paint_bitmap_row(map: &Map, pixels: &mut [u8], row: u16, background: u8, multi: bool) {
    let mut col = 0u16;
    while col < 40 {
        paint_bitmap_cell(map, pixels, row, col, background, multi);
        col += 1;
    }
}

fn paint_bitmap_cell(
    map: &Map,
    pixels: &mut [u8],
    row: u16,
    col: u16,
    background: u8,
    multi: bool,
) {
    let cell = usize::from(row) * 40 + usize::from(col);
    let screen = map.ram_byte(map.video_matrix() + cell as u16);
    let color = map.color_byte(cell);
    let mut line = 0u16;
    while line < 8 {
        let bitmap = map.ram_byte(bitmap_byte(map, row, col, line));
        blit_bitmap_line(pixels, col, row * 8 + line, bitmap, screen, color, background, multi);
        line += 1;
    }
}

fn bitmap_byte(map: &Map, row: u16, col: u16, line: u16) -> u16 {
    map.bitmap_base() + row * 320 + col * 8 + line
}

fn blit_bitmap_line(
    pixels: &mut [u8],
    col: u16,
    y: u16,
    bitmap: u8,
    screen: u8,
    color: u8,
    background: u8,
    multi: bool,
) {
    if multi {
        blit_multi_line(pixels, col, y, bitmap, screen, color, background);
        return;
    }
    blit_hires_line(pixels, col, y, bitmap, screen);
}

fn blit_hires_line(pixels: &mut [u8], col: u16, y: u16, bitmap: u8, screen: u8) {
    let ink = screen >> 4;
    let paper = screen & 0x0F;
    let mut bit = 0u16;
    while bit < 8 {
        let on = bitmap & (0x80 >> bit) != 0;
        put(pixels, col * 8 + bit, y, if on { ink } else { paper });
        bit += 1;
    }
}

fn blit_multi_line(
    pixels: &mut [u8],
    col: u16,
    y: u16,
    bitmap: u8,
    screen: u8,
    color: u8,
    background: u8,
) {
    let mut pair = 0u16;
    while pair < 4 {
        let code = (bitmap >> (6 - pair * 2)) & 0x03;
        let value = multi_color(code, screen, color, background);
        let x = col * 8 + pair * 2;
        put(pixels, x, y, value);
        put(pixels, x + 1, y, value);
        pair += 1;
    }
}

fn multi_color(code: u8, screen: u8, color: u8, background: u8) -> u8 {
    match code {
        1 => screen >> 4,
        2 => screen & 0x0F,
        3 => color & 0x0F,
        _ => background,
    }
}

fn put(pixels: &mut [u8], x: u16, y: u16, color: u8) {
    let index = usize::from(y) * usize::from(WIDTH) + usize::from(x);
    pixels[index] = color & 0x0F;
}
