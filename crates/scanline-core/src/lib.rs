mod cadence;
mod clock;
mod color;
mod demo;
mod error;
mod frame;
mod look;
mod mailbox;
mod presenter;
mod scale;
mod shade;
mod temporal;

pub use cadence::{PresentPace, source_index};
pub use clock::{FrameClock, c64_pal_clock, frames_from_samples, spectrum_clock};
pub use color::{PaletteKind, Rgb, linear_unorm16, mix_half, mix_quarter};
pub use demo::{demo_frame, demo_sprite_x};
pub use error::CoreError;
pub use frame::{
    Attribute, BORDER, CONTENT_HEIGHT, CONTENT_WIDTH, Content, Frame, Sprite, TRANSPARENT,
    attribute_index, compose, presented_size,
};
pub use look::Look;
pub use mailbox::Mailbox;
pub use presenter::Presenter;
pub use scale::{Viewport, centered_viewport, integer_scale};
pub use shade::shade_image;
pub use temporal::{Blend, TemporalDecision, TemporalHistory};
