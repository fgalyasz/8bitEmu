use crate::cadence::{PresentPace, source_index};
use crate::error::CoreError;
use crate::frame::Frame;
use crate::look::Look;
use crate::machine::Machine;
use crate::temporal::{Blend, TemporalHistory};

pub struct Presenter {
    pace: PresentPace,
    look: Look,
    display_tick: u64,
    shown: Option<u64>,
    history: TemporalHistory,
    frame: Option<Frame>,
    machine: Machine,
}

impl Presenter {
    pub fn new(pace: PresentPace) -> Self {
        Self {
            pace,
            look: Look::Sharp,
            display_tick: 0,
            shown: None,
            history: TemporalHistory::default(),
            frame: None,
            machine: Machine::new(),
        }
    }

    pub fn look(&self) -> Look {
        self.look
    }

    pub fn set_look(&mut self, look: Look) {
        self.look = look;
    }

    pub fn load_rom(&mut self, bytes: &[u8], model_128: bool) -> Result<(), CoreError> {
        self.machine.load_rom(bytes, model_128)
    }

    pub fn load_sna(&mut self, bytes: &[u8]) -> Result<(), CoreError> {
        self.machine.load_sna(bytes)
    }

    pub fn set_key(&mut self, row: u8, mask: u8, down: bool) {
        self.machine.set_key(row, mask, down);
    }

    pub fn reset(&mut self) {
        self.machine.reset();
        self.shown = None;
    }

    pub fn take_audio(&mut self) -> Vec<f32> {
        self.machine.take_audio()
    }

    pub fn on_display_tick(&mut self) -> Result<&Frame, CoreError> {
        let source = source_index(self.display_tick, self.pace);
        self.ensure_source(source)?;
        self.display_tick += 1;
        self.frame_ref()
    }

    fn ensure_source(&mut self, source: u64) -> Result<(), CoreError> {
        if self.shown == Some(source) {
            return Ok(());
        }
        self.store_source(source)
    }

    fn store_source(&mut self, source: u64) -> Result<(), CoreError> {
        let mut frame = self.machine.frame()?;
        apply_history(&mut frame, &mut self.history)?;
        self.shown = Some(source);
        self.frame = Some(frame);
        Ok(())
    }

    fn frame_ref(&self) -> Result<&Frame, CoreError> {
        self.frame.as_ref().ok_or(CoreError::EmptyFrame)
    }
}

fn apply_history(frame: &mut Frame, history: &mut TemporalHistory) -> Result<(), CoreError> {
    let decision = history.decide(&frame.index, usize::from(frame.width))?;
    frame.blend = pack_blend(&decision.blend);
    frame.previous = decision.previous;
    Ok(())
}

fn pack_blend(blend: &[Blend]) -> Vec<u8> {
    let mut packed = Vec::new();
    let mut index = 0;
    while index < blend.len() {
        packed.push(blend[index] as u8);
        index += 1;
    }
    packed
}
