use crate::color::PaletteKind;
use crate::error::CoreError;

pub const TRANSPARENT: u8 = 255;
pub const CONTENT_WIDTH: u16 = 256;
pub const CONTENT_HEIGHT: u16 = 192;
pub const BORDER: u16 = 16;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Attribute {
    pub ink: u8,
    pub paper: u8,
    pub bright: bool,
    pub flash: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Sprite {
    pub x: i16,
    pub y: i16,
    pub width: u16,
    pub height: u16,
    pub pixels: Vec<u8>,
    pub luma: Vec<u8>,
}

#[derive(Clone, Debug)]
pub struct Content {
    pub width: u16,
    pub height: u16,
    pub bitmap: Vec<u8>,
    pub attributes: Vec<Attribute>,
    pub flash_on: bool,
    pub border_px: u16,
    pub border: Vec<u8>,
    pub sprites: Vec<Sprite>,
    pub palette: PaletteKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Frame {
    pub width: u16,
    pub height: u16,
    pub index: Vec<u8>,
    pub luma: Vec<u8>,
    pub sprite: Vec<u8>,
    pub sprite_on: Vec<u8>,
    pub sprite_luma: Vec<u8>,
    pub blend: Vec<u8>,
    pub previous: Vec<u8>,
    pub palette: PaletteKind,
}

struct Pixel {
    index: u8,
    luma: u8,
}

pub fn compose(content: &Content) -> Result<Frame, CoreError> {
    validate(content)?;
    let pixels = raster_content(content);
    let mut frame = expand_border(content, &pixels);
    blit_all(content, &mut frame)?;
    Ok(frame)
}

pub fn attribute_index(attr: Attribute, bit: bool, flash_on: bool) -> u8 {
    let ink_on = if attr.flash && flash_on { !bit } else { bit };
    let color = if ink_on { attr.ink } else { attr.paper };
    ula_index(color, attr.bright)
}

pub fn presented_size(width: u16, height: u16, border_px: u16) -> (u16, u16) {
    (width + border_px * 2, height + border_px * 2)
}

fn ula_index(color: u8, bright: bool) -> u8 {
    let base = color & 7;
    if bright { base + 8 } else { base }
}

fn validate(content: &Content) -> Result<(), CoreError> {
    validate_grid(content)?;
    validate_sprites(&content.sprites)
}

fn validate_grid(content: &Content) -> Result<(), CoreError> {
    check_size(content.width, content.height)?;
    check_bitmap(content)?;
    check_attributes(content)?;
    check_border(content)
}

fn check_size(width: u16, height: u16) -> Result<(), CoreError> {
    if width == 0 || height == 0 {
        return Err(CoreError::EmptySize);
    }
    if width % 8 != 0 {
        return Err(CoreError::WidthAlignment { width });
    }
    Ok(())
}

fn check_bitmap(content: &Content) -> Result<(), CoreError> {
    let expected = plane_len(content.width, content.height);
    if content.bitmap.len() != expected {
        return Err(CoreError::BitmapLength {
            expected,
            actual: content.bitmap.len(),
        });
    }
    Ok(())
}

fn check_attributes(content: &Content) -> Result<(), CoreError> {
    let expected = plane_len(content.width, content.height);
    if content.attributes.len() != expected {
        return Err(CoreError::AttributeLength {
            expected,
            actual: content.attributes.len(),
        });
    }
    Ok(())
}

fn check_border(content: &Content) -> Result<(), CoreError> {
    let expected = usize::from(content.height + content.border_px * 2);
    if content.border.len() != expected {
        return Err(CoreError::BorderLength {
            expected,
            actual: content.border.len(),
        });
    }
    check_border_colors(content)
}

fn check_border_colors(content: &Content) -> Result<(), CoreError> {
    let mut index = 0;
    while index < content.border.len() {
        check_color(content.border[index])?;
        index += 1;
    }
    Ok(())
}

fn plane_len(width: u16, height: u16) -> usize {
    usize::from(width / 8) * usize::from(height)
}

fn validate_sprites(sprites: &[Sprite]) -> Result<(), CoreError> {
    let mut index = 0;
    while index < sprites.len() {
        validate_sprite(&sprites[index])?;
        index += 1;
    }
    Ok(())
}

fn validate_sprite(sprite: &Sprite) -> Result<(), CoreError> {
    let expected = usize::from(sprite.width) * usize::from(sprite.height);
    check_sprite_pixels(sprite, expected)?;
    check_sprite_luma(sprite, expected)
}

fn check_sprite_pixels(sprite: &Sprite, expected: usize) -> Result<(), CoreError> {
    if sprite.pixels.len() != expected {
        return Err(CoreError::SpritePixels {
            expected,
            actual: sprite.pixels.len(),
        });
    }
    Ok(())
}

fn check_sprite_luma(sprite: &Sprite, expected: usize) -> Result<(), CoreError> {
    if sprite.luma.len() != expected {
        return Err(CoreError::SpriteLuma {
            expected,
            actual: sprite.luma.len(),
        });
    }
    Ok(())
}

fn raster_content(content: &Content) -> Vec<Pixel> {
    let mut pixels = Vec::new();
    let mut y = 0u16;
    while y < content.height {
        push_content_row(&mut pixels, content, y);
        y += 1;
    }
    pixels
}

fn push_content_row(pixels: &mut Vec<Pixel>, content: &Content, y: u16) {
    let mut x = 0u16;
    while x < content.width {
        pixels.push(pixel_at(content, x, y));
        x += 1;
    }
}

fn pixel_at(content: &Content, x: u16, y: u16) -> Pixel {
    let bit = bitmap_bit(content, x, y);
    let attr = content.attributes[attr_offset(content, x, y)];
    Pixel {
        index: attribute_index(attr, bit, content.flash_on),
        luma: u8::from(bit),
    }
}

fn bitmap_bit(content: &Content, x: u16, y: u16) -> bool {
    let stride = usize::from(content.width / 8);
    let offset = usize::from(y) * stride + usize::from(x / 8);
    let shift = 7 - (x % 8);
    (content.bitmap[offset] >> shift) & 1 == 1
}

fn attr_offset(content: &Content, x: u16, y: u16) -> usize {
    usize::from(y) * usize::from(content.width / 8) + usize::from(x / 8)
}

fn expand_border(content: &Content, pixels: &[Pixel]) -> Frame {
    let (width, height) = presented_size(content.width, content.height, content.border_px);
    let mut frame = empty_frame(width, height, content.palette);
    paint_rows(content, pixels, &mut frame);
    frame.previous = frame.index.clone();
    frame
}

fn empty_frame(width: u16, height: u16, palette: PaletteKind) -> Frame {
    let mut frame = blank_frame(width, height, palette);
    fill_planes(&mut frame, usize::from(width) * usize::from(height));
    frame
}

fn blank_frame(width: u16, height: u16, palette: PaletteKind) -> Frame {
    Frame {
        width,
        height,
        palette,
        index: Vec::new(),
        luma: Vec::new(),
        sprite: Vec::new(),
        sprite_on: Vec::new(),
        sprite_luma: Vec::new(),
        blend: Vec::new(),
        previous: Vec::new(),
    }
}

fn fill_planes(frame: &mut Frame, count: usize) {
    frame.index = vec![0; count];
    frame.luma = vec![0; count];
    frame.sprite = vec![TRANSPARENT; count];
    frame.sprite_on = vec![0; count];
    frame.sprite_luma = vec![0; count];
    frame.blend = vec![0; count];
    frame.previous = vec![0; count];
}

fn paint_rows(content: &Content, pixels: &[Pixel], frame: &mut Frame) {
    let mut y = 0u16;
    while y < frame.height {
        paint_row(content, pixels, frame, y);
        y += 1;
    }
}

fn paint_row(content: &Content, pixels: &[Pixel], frame: &mut Frame, y: u16) {
    let mut x = 0u16;
    while x < frame.width {
        paint_pixel(content, pixels, frame, x, y);
        x += 1;
    }
}

fn paint_pixel(content: &Content, pixels: &[Pixel], frame: &mut Frame, x: u16, y: u16) {
    let offset = pixel_offset(frame.width, x, y);
    let border = content.border[usize::from(y)];
    if let Some(pixel) = content_pixel(content, pixels, x, y) {
        frame.index[offset] = pixel.index;
        frame.luma[offset] = pixel.luma;
        return;
    }
    frame.index[offset] = border;
    frame.luma[offset] = 0;
}

fn content_pixel<'a>(content: &Content, pixels: &'a [Pixel], x: u16, y: u16) -> Option<&'a Pixel> {
    let local_x = x.checked_sub(content.border_px)?;
    let local_y = y.checked_sub(content.border_px)?;
    if local_x >= content.width || local_y >= content.height {
        return None;
    }
    Some(&pixels[pixel_offset(content.width, local_x, local_y)])
}

fn pixel_offset(width: u16, x: u16, y: u16) -> usize {
    usize::from(y) * usize::from(width) + usize::from(x)
}

fn blit_all(content: &Content, frame: &mut Frame) -> Result<(), CoreError> {
    let mut index = 0;
    while index < content.sprites.len() {
        blit_sprite(content, frame, &content.sprites[index])?;
        index += 1;
    }
    Ok(())
}

fn blit_sprite(content: &Content, frame: &mut Frame, sprite: &Sprite) -> Result<(), CoreError> {
    let mut row = 0u16;
    while row < sprite.height {
        blit_sprite_row(content, frame, sprite, row)?;
        row += 1;
    }
    Ok(())
}

fn blit_sprite_row(
    content: &Content,
    frame: &mut Frame,
    sprite: &Sprite,
    row: u16,
) -> Result<(), CoreError> {
    let mut col = 0u16;
    while col < sprite.width {
        blit_sprite_pixel(content, frame, sprite, col, row)?;
        col += 1;
    }
    Ok(())
}

fn blit_sprite_pixel(
    content: &Content,
    frame: &mut Frame,
    sprite: &Sprite,
    col: u16,
    row: u16,
) -> Result<(), CoreError> {
    let color = sprite.pixels[sprite_offset(sprite, col, row)];
    stamp_color(content, frame, sprite, col, row, color)
}

fn stamp_color(
    content: &Content,
    frame: &mut Frame,
    sprite: &Sprite,
    col: u16,
    row: u16,
    color: u8,
) -> Result<(), CoreError> {
    if color == TRANSPARENT {
        return Ok(());
    }
    check_color(color)?;
    stamp(content, frame, sprite, col, row, color);
    Ok(())
}

fn check_color(color: u8) -> Result<(), CoreError> {
    if color > 15 {
        return Err(CoreError::IndexRange { index: color });
    }
    Ok(())
}

fn stamp(content: &Content, frame: &mut Frame, sprite: &Sprite, col: u16, row: u16, color: u8) {
    let Some(point) = sprite_point(content, sprite, col, row) else {
        return;
    };
    let offset = pixel_offset(frame.width, point.0, point.1);
    frame.sprite[offset] = color;
    frame.sprite_on[offset] = 1;
    frame.sprite_luma[offset] = sprite.luma[sprite_offset(sprite, col, row)];
}

fn sprite_point(content: &Content, sprite: &Sprite, col: u16, row: u16) -> Option<(u16, u16)> {
    let x = i32::from(sprite.x) + i32::from(col) + i32::from(content.border_px);
    let y = i32::from(sprite.y) + i32::from(row) + i32::from(content.border_px);
    let max_x = i32::from(content.width + content.border_px * 2);
    let max_y = i32::from(content.height + content.border_px * 2);
    if x < 0 || y < 0 || x >= max_x || y >= max_y {
        return None;
    }
    Some((x as u16, y as u16))
}

fn sprite_offset(sprite: &Sprite, col: u16, row: u16) -> usize {
    usize::from(row) * usize::from(sprite.width) + usize::from(col)
}
