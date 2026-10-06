use crate::error::CoreError;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FrameClock {
    pub numerator: u32,
    pub denominator: u32,
}

pub fn spectrum_clock() -> FrameClock {
    FrameClock {
        numerator: 15_625,
        denominator: 312,
    }
}

pub fn c64_pal_clock() -> FrameClock {
    FrameClock {
        numerator: 13_684,
        denominator: 273,
    }
}

pub fn frames_from_samples(
    samples: u64,
    sample_rate: u32,
    clock: FrameClock,
) -> Result<u64, CoreError> {
    if sample_rate == 0 || clock.denominator == 0 {
        return Err(CoreError::SampleRate);
    }
    Ok(completed_frames(samples, sample_rate, clock))
}

fn completed_frames(samples: u64, sample_rate: u32, clock: FrameClock) -> u64 {
    let numer = samples * u64::from(clock.numerator);
    let denom = u64::from(sample_rate) * u64::from(clock.denominator);
    numer / denom
}
