#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PresentPace {
    VariableRefresh,
    Fixed60Hz,
}

pub fn source_index(display_tick: u64, pace: PresentPace) -> u64 {
    match pace {
        PresentPace::VariableRefresh => display_tick,
        PresentPace::Fixed60Hz => display_tick * 5 / 6,
    }
}
