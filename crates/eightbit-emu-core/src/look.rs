#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Look {
    Sharp,
    SoftEdge,
    Temporal,
}

impl Look {
    pub fn from_digit(digit: u8) -> Option<Look> {
        match digit {
            1 => Some(Look::Sharp),
            2 => Some(Look::SoftEdge),
            3 => Some(Look::Temporal),
            _ => None,
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            Look::Sharp => "8bitEmu — Sharp",
            Look::SoftEdge => "8bitEmu — Soft edge",
            Look::Temporal => "8bitEmu — Temporal color",
        }
    }

    pub fn shader_id(self) -> u32 {
        match self {
            Look::Sharp => 0,
            Look::SoftEdge => 1,
            Look::Temporal => 2,
        }
    }
}
