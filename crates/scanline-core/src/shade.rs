use crate::color::{Rgb, mix_half, mix_quarter};
use crate::error::CoreError;
use crate::frame::Frame;
use crate::look::Look;

pub fn shade_image(frame: &Frame, look: Look) -> Result<Vec<u8>, CoreError> {
    validate_frame(frame)?;
    let palette = frame.palette.colors();
    let base = base_image(frame, look, &palette);
    Ok(pack_image(finish_image(frame, look, &base)))
}

fn validate_frame(frame: &Frame) -> Result<(), CoreError> {
    let expected = usize::from(frame.width) * usize::from(frame.height);
    check_plane("index", frame.index.len(), expected)?;
    check_planes(frame, expected)?;
    check_indices(frame)
}

fn check_planes(frame: &Frame, expected: usize) -> Result<(), CoreError> {
    check_plane("luma", frame.luma.len(), expected)?;
    check_plane("sprite", frame.sprite.len(), expected)?;
    check_plane("sprite_on", frame.sprite_on.len(), expected)?;
    check_more_planes(frame, expected)
}

fn check_more_planes(frame: &Frame, expected: usize) -> Result<(), CoreError> {
    check_plane("sprite_luma", frame.sprite_luma.len(), expected)?;
    check_plane("blend", frame.blend.len(), expected)?;
    check_plane("previous", frame.previous.len(), expected)
}

fn check_plane(plane: &'static str, actual: usize, expected: usize) -> Result<(), CoreError> {
    if actual == expected {
        return Ok(());
    }
    Err(CoreError::FrameLength {
        plane,
        expected,
        actual,
    })
}

fn check_indices(frame: &Frame) -> Result<(), CoreError> {
    let mut index = 0;
    while index < frame.index.len() {
        check_pixel(frame, index)?;
        index += 1;
    }
    Ok(())
}

fn check_pixel(frame: &Frame, index: usize) -> Result<(), CoreError> {
    check_palette_index(frame.index[index])?;
    check_palette_index(frame.previous[index])?;
    check_sprite_index(frame, index)
}

fn check_palette_index(index: u8) -> Result<(), CoreError> {
    if index > 15 {
        return Err(CoreError::IndexRange { index });
    }
    Ok(())
}

fn check_sprite_index(frame: &Frame, index: usize) -> Result<(), CoreError> {
    if frame.sprite_on[index] == 0 {
        return Ok(());
    }
    check_palette_index(frame.sprite[index])
}

fn base_image(frame: &Frame, look: Look, palette: &[Rgb; 16]) -> Vec<Rgb> {
    let mut colors = Vec::new();
    let mut index = 0;
    while index < frame.index.len() {
        colors.push(base_color(frame, look, palette, index));
        index += 1;
    }
    colors
}

fn base_color(frame: &Frame, look: Look, palette: &[Rgb; 16], index: usize) -> Rgb {
    let background = background_color(frame, look, palette, index);
    cover_sprite(background, frame, palette, index)
}

fn background_color(frame: &Frame, look: Look, palette: &[Rgb; 16], index: usize) -> Rgb {
    let current = palette[usize::from(frame.index[index])];
    if look != Look::Temporal {
        return current;
    }
    temporal_color(current, frame, palette, index)
}

fn temporal_color(current: Rgb, frame: &Frame, palette: &[Rgb; 16], index: usize) -> Rgb {
    match frame.blend[index] {
        1 => mix_half(current, palette[usize::from(frame.previous[index])]),
        2 => mix_half(current, palette[usize::from(neighbor_index(frame, index))]),
        _ => current,
    }
}

fn neighbor_index(frame: &Frame, index: usize) -> u8 {
    let right = right_offset(frame, index);
    frame.index[right]
}

fn cover_sprite(background: Rgb, frame: &Frame, palette: &[Rgb; 16], index: usize) -> Rgb {
    if frame.sprite_on[index] == 0 {
        return background;
    }
    palette[usize::from(frame.sprite[index])]
}

fn finish_image(frame: &Frame, look: Look, base: &[Rgb]) -> Vec<Rgb> {
    let mut colors = Vec::new();
    let mut index = 0;
    while index < base.len() {
        colors.push(soften(frame, look, base, index));
        index += 1;
    }
    colors
}

fn soften(frame: &Frame, look: Look, base: &[Rgb], index: usize) -> Rgb {
    if look != Look::SoftEdge {
        return base[index];
    }
    let right = right_offset(frame, index);
    if visible_luma(frame, index) == visible_luma(frame, right) {
        return base[index];
    }
    mix_quarter(base[index], base[right])
}

fn visible_luma(frame: &Frame, index: usize) -> u8 {
    if frame.sprite_on[index] != 0 {
        return frame.sprite_luma[index];
    }
    frame.luma[index]
}

fn right_offset(frame: &Frame, index: usize) -> usize {
    let width = usize::from(frame.width);
    if width == 0 || index % width + 1 >= width {
        return index;
    }
    index + 1
}

fn pack_image(colors: Vec<Rgb>) -> Vec<u8> {
    let mut bytes = Vec::new();
    let mut index = 0;
    while index < colors.len() {
        push_rgba(&mut bytes, colors[index]);
        index += 1;
    }
    bytes
}

fn push_rgba(bytes: &mut Vec<u8>, color: Rgb) {
    bytes.push(color.r);
    bytes.push(color.g);
    bytes.push(color.b);
    bytes.push(255);
}
