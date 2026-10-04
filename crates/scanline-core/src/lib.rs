mod cadence;
mod clock;
mod color;
mod demo;
mod display;
mod error;
mod frame;
mod look;
mod machine;
mod mailbox;
mod memory;
mod presenter;
mod program;
mod sna;
mod scale;
mod shade;
mod temporal;
mod z80;

pub use cadence::{PresentPace, source_index};
pub use clock::{FrameClock, c64_pal_clock, frames_from_samples, spectrum_clock};
pub use color::{PaletteKind, Rgb, linear_unorm16, mix_half, mix_quarter};
pub use demo::{demo_frame, demo_sprite_x};
pub use display::{attribute_address, bitmap_address, frame_from};
pub use error::CoreError;
pub use frame::{
    Attribute, BORDER, CONTENT_HEIGHT, CONTENT_WIDTH, Content, Frame, Sprite, TRANSPARENT,
    attribute_index, compose, presented_size,
};
pub use look::Look;
pub use machine::{Machine, run_until_halt};
pub use mailbox::Mailbox;
pub use memory::Memory;
pub use presenter::Presenter;
pub use scale::{Viewport, centered_viewport, integer_scale};
pub use shade::shade_image;
pub use temporal::{Blend, TemporalDecision, TemporalHistory};
pub use z80::{Cpu, Ports, step};
