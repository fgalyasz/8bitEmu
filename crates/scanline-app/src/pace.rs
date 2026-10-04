use std::time::Duration;

pub const DISPLAY_FRAME: Duration = Duration::from_nanos(1_000_000_000 / 60);
pub const CATCH_UP: u32 = 4;
pub const TURBO_SLICE: Duration = Duration::from_millis(12);

pub fn due_ticks(late: Duration, frame: Duration, cap: u32) -> u32 {
    if frame.is_zero() {
        return 0;
    }
    let steps = late.as_nanos() / frame.as_nanos();
    u32::try_from(steps).unwrap_or(u32::MAX).min(cap)
}
