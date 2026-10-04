#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Viewport {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

pub fn integer_scale(src_w: u32, src_h: u32, dst_w: u32, dst_h: u32) -> Option<u32> {
    if src_w == 0 || src_h == 0 || dst_w == 0 || dst_h == 0 {
        return None;
    }
    Some((dst_w / src_w).min(dst_h / src_h).max(1))
}

pub fn centered_viewport(src_w: u32, src_h: u32, dst_w: u32, dst_h: u32, scale: u32) -> Viewport {
    let width = src_w.saturating_mul(scale).min(dst_w);
    let height = src_h.saturating_mul(scale).min(dst_h);
    viewport_at(dst_w, dst_h, width, height)
}

fn viewport_at(dst_w: u32, dst_h: u32, width: u32, height: u32) -> Viewport {
    Viewport {
        x: dst_w.saturating_sub(width) / 2,
        y: dst_h.saturating_sub(height) / 2,
        width,
        height,
    }
}
