use crate::error::CoreError;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Blend {
    Current = 0,
    Previous = 1,
    Neighbor = 2,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TemporalDecision {
    pub blend: Vec<Blend>,
    pub previous: Vec<u8>,
}

#[derive(Clone, Debug, Default)]
pub struct TemporalHistory {
    previous: Option<Vec<u8>>,
    two_ago: Option<Vec<u8>>,
}

impl TemporalHistory {
    pub fn decide(&mut self, current: &[u8], width: usize) -> Result<TemporalDecision, CoreError> {
        self.check_history(current)?;
        let decision = self.decision_for(current, width);
        self.push(current.to_vec());
        Ok(decision)
    }

    fn check_history(&self, current: &[u8]) -> Result<(), CoreError> {
        check_stored(self.previous.as_deref(), current.len())?;
        check_stored(self.two_ago.as_deref(), current.len())
    }

    fn decision_for(&self, current: &[u8], width: usize) -> TemporalDecision {
        TemporalDecision {
            blend: blends_for(self, current, width),
            previous: stored_or_current(self.previous.as_deref(), current),
        }
    }

    fn push(&mut self, current: Vec<u8>) {
        self.two_ago = self.previous.take();
        self.previous = Some(current);
    }
}

fn check_stored(stored: Option<&[u8]>, expected: usize) -> Result<(), CoreError> {
    let Some(stored) = stored else {
        return Ok(());
    };
    if stored.len() != expected {
        return Err(CoreError::HistoryLength {
            expected,
            actual: stored.len(),
        });
    }
    Ok(())
}

fn stored_or_current(stored: Option<&[u8]>, current: &[u8]) -> Vec<u8> {
    stored.unwrap_or(current).to_vec()
}

fn blends_for(history: &TemporalHistory, current: &[u8], width: usize) -> Vec<Blend> {
    let mut blend = vec![Blend::Current; current.len()];
    fill_blends(&mut blend, history, current, width);
    blend
}

fn fill_blends(blend: &mut [Blend], history: &TemporalHistory, current: &[u8], width: usize) {
    let (Some(previous), Some(two_ago)) = (history.previous.as_deref(), history.two_ago.as_deref())
    else {
        return;
    };
    let mut index = 0;
    while index < current.len() {
        blend[index] = blend_at(index, current, previous, two_ago, width);
        index += 1;
    }
}

fn blend_at(index: usize, current: &[u8], previous: &[u8], two_ago: &[u8], width: usize) -> Blend {
    if current[index] == two_ago[index] && current[index] != previous[index] {
        return Blend::Previous;
    }
    checker_blend(index, current, two_ago, width)
}

fn checker_blend(index: usize, current: &[u8], two_ago: &[u8], width: usize) -> Blend {
    if current[index] != two_ago[index] || width == 0 {
        return Blend::Current;
    }
    let right = index + 1;
    let skip = index + 2;
    if !period_two(index, right, skip, current, width) {
        return Blend::Current;
    }
    Blend::Neighbor
}

fn period_two(index: usize, right: usize, skip: usize, current: &[u8], width: usize) -> bool {
    if skip >= current.len() || index / width != skip / width {
        return false;
    }
    current[index] != current[right] && current[index] == current[skip]
}
