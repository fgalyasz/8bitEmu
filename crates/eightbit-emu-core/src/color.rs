#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

const ULA: [Rgb; 16] = [
    Rgb { r: 0, g: 0, b: 0 },
    Rgb {
        r: 0,
        g: 0,
        b: 0xD7,
    },
    Rgb {
        r: 0xD7,
        g: 0,
        b: 0,
    },
    Rgb {
        r: 0xD7,
        g: 0,
        b: 0xD7,
    },
    Rgb {
        r: 0,
        g: 0xD7,
        b: 0,
    },
    Rgb {
        r: 0,
        g: 0xD7,
        b: 0xD7,
    },
    Rgb {
        r: 0xD7,
        g: 0xD7,
        b: 0,
    },
    Rgb {
        r: 0xD7,
        g: 0xD7,
        b: 0xD7,
    },
    Rgb { r: 0, g: 0, b: 0 },
    Rgb {
        r: 0,
        g: 0,
        b: 0xFF,
    },
    Rgb {
        r: 0xFF,
        g: 0,
        b: 0,
    },
    Rgb {
        r: 0xFF,
        g: 0,
        b: 0xFF,
    },
    Rgb {
        r: 0,
        g: 0xFF,
        b: 0,
    },
    Rgb {
        r: 0,
        g: 0xFF,
        b: 0xFF,
    },
    Rgb {
        r: 0xFF,
        g: 0xFF,
        b: 0,
    },
    Rgb {
        r: 0xFF,
        g: 0xFF,
        b: 0xFF,
    },
];

const COLODORE: [Rgb; 16] = [
    Rgb {
        r: 0x00,
        g: 0x00,
        b: 0x00,
    },
    Rgb {
        r: 0xFF,
        g: 0xFF,
        b: 0xFF,
    },
    Rgb {
        r: 0x96,
        g: 0x28,
        b: 0x2E,
    },
    Rgb {
        r: 0x5B,
        g: 0xD6,
        b: 0xCE,
    },
    Rgb {
        r: 0x9F,
        g: 0x2D,
        b: 0xAD,
    },
    Rgb {
        r: 0x41,
        g: 0xB9,
        b: 0x36,
    },
    Rgb {
        r: 0x27,
        g: 0x24,
        b: 0xC4,
    },
    Rgb {
        r: 0xEF,
        g: 0xF3,
        b: 0x47,
    },
    Rgb {
        r: 0x9F,
        g: 0x48,
        b: 0x15,
    },
    Rgb {
        r: 0x5E,
        g: 0x35,
        b: 0x00,
    },
    Rgb {
        r: 0xDA,
        g: 0x5F,
        b: 0x66,
    },
    Rgb {
        r: 0x47,
        g: 0x47,
        b: 0x47,
    },
    Rgb {
        r: 0x78,
        g: 0x78,
        b: 0x78,
    },
    Rgb {
        r: 0x91,
        g: 0xFF,
        b: 0x84,
    },
    Rgb {
        r: 0x68,
        g: 0x64,
        b: 0xFF,
    },
    Rgb {
        r: 0xAE,
        g: 0xAE,
        b: 0xAE,
    },
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaletteKind {
    Ula,
    C64,
}

impl PaletteKind {
    pub fn colors(self) -> [Rgb; 16] {
        match self {
            PaletteKind::Ula => ULA,
            PaletteKind::C64 => COLODORE,
        }
    }
}

pub fn mix_half(first: Rgb, second: Rgb) -> Rgb {
    mix_rgb(first, second, 1, 2)
}

pub fn mix_quarter(first: Rgb, second: Rgb) -> Rgb {
    mix_rgb(first, second, 1, 4)
}

pub fn linear_unorm16(channel: u8) -> u16 {
    (to_linear(channel) * 65_535.0).round() as u16
}

fn mix_rgb(first: Rgb, second: Rgb, second_num: u32, denom: u32) -> Rgb {
    Rgb {
        r: mix_channel(first.r, second.r, second_num, denom),
        g: mix_channel(first.g, second.g, second_num, denom),
        b: mix_channel(first.b, second.b, second_num, denom),
    }
}

fn mix_channel(first: u8, second: u8, second_num: u32, denom: u32) -> u8 {
    let first_weight = f64::from(denom - second_num);
    let mixed = to_linear(first) * first_weight + to_linear(second) * f64::from(second_num);
    from_linear(mixed / f64::from(denom))
}

fn to_linear(channel: u8) -> f64 {
    let unit = f64::from(channel) / 255.0;
    if unit <= 0.04045 {
        return unit / 12.92;
    }
    ((unit + 0.055) / 1.055).powf(2.4)
}

fn from_linear(linear: f64) -> u8 {
    let clamped = linear.clamp(0.0, 1.0);
    let encoded = encode_unit(clamped);
    (encoded * 255.0).round() as u8
}

fn encode_unit(linear: f64) -> f64 {
    if linear <= 0.0031308 {
        return 12.92 * linear;
    }
    1.055 * linear.powf(1.0 / 2.4) - 0.055
}
