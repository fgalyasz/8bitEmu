use crate::cadence::{PresentPace, source_index};
use crate::error::CoreError;
use crate::frame::Frame;
use crate::look::Look;
use crate::machine::Machine;
use crate::tape::{self, KeyHold};
use crate::temporal::{Blend, TemporalHistory};

pub struct Presenter {
    pace: PresentPace,
    look: Look,
    display_tick: u64,
    shown: Option<u64>,
    history: TemporalHistory,
    frame: Option<Frame>,
    machine: Machine,
    prompt: Vec<Vec<KeyHold>>,
    prompt_at: Option<usize>,
    holds: Vec<KeyHold>,
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
            prompt: Vec::new(),
            prompt_at: None,
            holds: Vec::new(),
        }
    }

    pub fn look(&self) -> Look {
        self.look
    }

    pub fn set_look(&mut self, look: Look) {
        self.look = look;
    }

    pub fn load_rom(&mut self, bytes: &[u8], model_128: bool) -> Result<(), CoreError> {
        self.machine.load_rom(bytes, model_128)?;
        self.arm_prompt();
        Ok(())
    }

    pub fn load_sna(&mut self, bytes: &[u8]) -> Result<(), CoreError> {
        self.machine.load_sna(bytes)?;
        self.arm_prompt();
        Ok(())
    }

    pub fn load_tape(&mut self, bytes: &[u8]) -> Result<(), CoreError> {
        self.machine.load_tape(bytes)?;
        self.arm_prompt();
        Ok(())
    }

    pub fn resume_tape(&mut self) {
        self.machine.resume_tape();
    }

    pub fn snapshot(&self) -> Result<Vec<u8>, CoreError> {
        self.machine.snapshot()
    }

    pub fn take_tap(&mut self) -> Option<Vec<u8>> {
        self.machine.take_tap()
    }

    pub fn tap_bytes(&self) -> Option<Vec<u8>> {
        self.machine.tap_bytes()
    }

    pub fn tzx_bytes(&self) -> Option<Vec<u8>> {
        self.machine.tzx_bytes()
    }

    pub fn set_stick(&mut self, mask: u8, down: bool) {
        self.machine.set_stick(mask, down);
    }

    pub fn set_key(&mut self, row: u8, mask: u8, down: bool) {
        self.machine.set_key(row, mask, down);
    }

    pub fn is_booted(&self) -> bool {
        self.machine.is_booted()
    }

    pub fn reset(&mut self) {
        self.machine.reset();
        self.shown = None;
        self.arm_prompt();
    }

    pub fn take_audio(&mut self) -> Vec<f32> {
        self.machine.take_audio()
    }

    pub fn set_loading_sound(&mut self, on: bool) {
        self.machine.set_loading_sound(on);
    }

    pub fn loading_sound(&self) -> bool {
        self.machine.loading_sound()
    }

    pub fn hears_loading(&self) -> bool {
        self.machine.hears_loading()
    }

    pub fn tape_edge(&self) -> usize {
        self.machine.tape_edge()
    }

    pub fn on_display_tick(&mut self) -> Result<&Frame, CoreError> {
        let source = source_index(self.display_tick, self.pace);
        self.ensure_source(source)?;
        self.display_tick += 1;
        self.frame_ref()
    }

    pub fn rush(&mut self) -> Result<Option<Frame>, CoreError> {
        if self.prompt_at.is_some() || self.machine.turbo_budget() == 0 {
            return Ok(None);
        }
        self.store_warp_frame()?;
        Ok(self.frame.clone())
    }

    fn store_warp_frame(&mut self) -> Result<(), CoreError> {
        let mut frame = self.machine.frame()?;
        apply_history(&mut frame, &mut self.history)?;
        let _ = self.machine.take_audio();
        self.frame = Some(frame);
        Ok(())
    }

    fn ensure_source(&mut self, source: u64) -> Result<(), CoreError> {
        if self.shown == Some(source) {
            return Ok(());
        }
        self.store_source(source)
    }

    fn store_source(&mut self, source: u64) -> Result<(), CoreError> {
        self.apply_prompt();
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

impl Presenter {
    fn arm_prompt(&mut self) {
        self.release_holds();
        if self.machine.is_booted() && self.machine.has_tape() {
            self.prompt = tape::load_prompt(self.machine.is_128());
            self.prompt_at = Some(0);
            return;
        }
        self.prompt.clear();
        self.prompt_at = None;
    }

    fn apply_prompt(&mut self) {
        let Some(index) = self.prompt_at else {
            return;
        };
        self.release_holds();
        if index >= self.prompt.len() {
            self.prompt_at = None;
            return;
        }
        let holds = self.prompt[index].clone();
        self.press_holds(&holds);
        self.prompt_at = Some(index + 1);
    }

    fn press_holds(&mut self, holds: &[KeyHold]) {
        let mut index = 0;
        while index < holds.len() {
            self.machine.set_key(holds[index].row, holds[index].mask, true);
            index += 1;
        }
        self.holds = holds.to_vec();
    }

    fn release_holds(&mut self) {
        let mut index = 0;
        while index < self.holds.len() {
            self.machine.set_key(self.holds[index].row, self.holds[index].mask, false);
            index += 1;
        }
        self.holds.clear();
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
