use crate::color::PaletteKind;
use crate::error::CoreError;
use crate::frame::{Frame, frame_from_indexes};

use super::mem::Map;

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
    let mut row = 0u16;
    while row < 25 {
        paint_row(map, &mut pixels, row, background);
        row += 1;
    }
    pixels
}

fn paint_row(map: &Map, pixels: &mut [u8], row: u16, background: u8) {
    let mut col = 0u16;
    while col < 40 {
        paint_cell(map, pixels, row, col, background);
        col += 1;
    }
}

fn paint_cell(map: &Map, pixels: &mut [u8], row: u16, col: u16, background: u8) {
    let offset = usize::from(row) * 40 + usize::from(col);
    let code = map.ram_byte(0x0400 + offset as u16);
    let color = map.color_byte(offset);
    let mut line = 0u16;
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
    line: u16,
    code: u8,
    color: u8,
    background: u8,
) {
    let glyph = map.chargen_byte(usize::from(code) * 8 + usize::from(line));
    let y = row * 8 + line;
    let mut bit = 0u16;
    while bit < 8 {
        let x = col * 8 + bit;
        let on = glyph & (0x80 >> bit) != 0;
        let index = usize::from(y) * usize::from(WIDTH) + usize::from(x);
        pixels[index] = if on { color } else { background };
        bit += 1;
    }
}
