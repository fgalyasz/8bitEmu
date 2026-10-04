use std::fmt;

#[derive(Debug, PartialEq, Eq)]
pub enum CoreError {
    EmptySize,
    WidthAlignment {
        width: u16,
    },
    BitmapLength {
        expected: usize,
        actual: usize,
    },
    AttributeLength {
        expected: usize,
        actual: usize,
    },
    BorderLength {
        expected: usize,
        actual: usize,
    },
    SpritePixels {
        expected: usize,
        actual: usize,
    },
    SpriteLuma {
        expected: usize,
        actual: usize,
    },
    IndexRange {
        index: u8,
    },
    HistoryLength {
        expected: usize,
        actual: usize,
    },
    FrameLength {
        plane: &'static str,
        expected: usize,
        actual: usize,
    },
    SampleRate,
    EmptyFrame,
}

impl fmt::Display for CoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.message())
    }
}

impl CoreError {
    fn message(&self) -> String {
        match self {
            CoreError::EmptySize => size_text(),
            CoreError::WidthAlignment { width } => align_text(*width),
            CoreError::SampleRate => rate_text(),
            CoreError::EmptyFrame => frame_text(),
            CoreError::IndexRange { index } => index_text(*index),
            other => other.length_text(),
        }
    }

    fn length_text(&self) -> String {
        match self {
            CoreError::BitmapLength { expected, actual } => len_text("bitmap", *expected, *actual),
            CoreError::AttributeLength { expected, actual } => {
                len_text("attribute", *expected, *actual)
            }
            CoreError::BorderLength { expected, actual } => len_text("border", *expected, *actual),
            other => other.sprite_text(),
        }
    }

    fn sprite_text(&self) -> String {
        match self {
            CoreError::SpritePixels { expected, actual } => {
                len_text("sprite pixels", *expected, *actual)
            }
            CoreError::SpriteLuma { expected, actual } => {
                len_text("sprite luma", *expected, *actual)
            }
            CoreError::HistoryLength { expected, actual } => {
                len_text("history", *expected, *actual)
            }
            CoreError::FrameLength {
                plane,
                expected,
                actual,
            } => len_text(plane, *expected, *actual),
            _ => size_text(),
        }
    }
}

fn size_text() -> String {
    "frame width and height must be non-zero".to_string()
}

fn align_text(width: u16) -> String {
    format!("width {width} is not a multiple of 8")
}

fn rate_text() -> String {
    "sample rate must be non-zero".to_string()
}

fn frame_text() -> String {
    "presenter has no frame".to_string()
}

fn index_text(index: u8) -> String {
    format!("palette index {index} is above 15")
}

fn len_text(name: &str, expected: usize, actual: usize) -> String {
    format!("{name} length {actual}, expected {expected}")
}
