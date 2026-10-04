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

pub fn percent_size(src_w: u32, src_h: u32, percent: u32) -> (u32, u32) {
    let width = (src_w.saturating_mul(percent) / 100).max(1);
    let height = (src_h.saturating_mul(percent) / 100).max(1);
    (width, height)
}

pub fn aspect_fit(src_w: u32, src_h: u32, dst_w: u32, dst_h: u32) -> Viewport {
    if src_w == 0 || src_h == 0 || dst_w == 0 || dst_h == 0 {
        return Viewport { x: 0, y: 0, width: 1, height: 1 };
    }
    let (width, height) = fit_pair(src_w, src_h, dst_w, dst_h);
    viewport_at(dst_w, dst_h, width, height)
}

pub fn place_percent(src_w: u32, src_h: u32, dst_w: u32, dst_h: u32, percent: u32) -> Viewport {
    let (width, height) = percent_size(src_w, src_h, percent);
    if width <= dst_w && height <= dst_h {
        return viewport_at(dst_w, dst_h, width, height);
    }
    aspect_fit(src_w, src_h, dst_w, dst_h)
}

fn fit_pair(src_w: u32, src_h: u32, dst_w: u32, dst_h: u32) -> (u32, u32) {
    if dst_w.saturating_mul(src_h) > dst_h.saturating_mul(src_w) {
        return ((dst_h.saturating_mul(src_w) / src_h).max(1), dst_h.max(1));
    }
    (dst_w.max(1), (dst_w.saturating_mul(src_h) / src_w).max(1))
}

fn viewport_at(dst_w: u32, dst_h: u32, width: u32, height: u32) -> Viewport {
    Viewport {
        x: dst_w.saturating_sub(width) / 2,
        y: dst_h.saturating_sub(height) / 2,
        width,
        height,
    }
}
