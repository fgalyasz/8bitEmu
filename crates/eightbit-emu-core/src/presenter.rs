use crate::c64;
use crate::cadence::{PresentPace, source_index};
use crate::error::CoreError;
use crate::frame::Frame;
use crate::look::Look;
use crate::machine::Machine;
use crate::tape::{self, KeyHold};
use crate::temporal::{Blend, TemporalHistory};

enum Host {
    Spectrum(Machine),
    C64(c64::Machine),
}

pub struct Presenter {
    pace: PresentPace,
    look: Look,
    display_tick: u64,
    shown: Option<u64>,
    history: TemporalHistory,
    frame: Option<Frame>,
    host: Host,
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
            host: Host::Spectrum(Machine::new()),
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

    pub fn is_c64(&self) -> bool {
        matches!(self.host, Host::C64(_))
    }

    pub fn load_rom(&mut self, bytes: &[u8], model_128: bool) -> Result<(), CoreError> {
        self.ensure_spectrum().load_rom(bytes, model_128)?;
        self.arm_prompt();
        Ok(())
    }

    pub fn load_c64_roms(
        &mut self,
        kernal: &[u8],
        basic: &[u8],
        chargen: &[u8],
    ) -> Result<(), CoreError> {
        let mut machine = c64::Machine::new();
        machine.load_roms(kernal, basic, chargen)?;
        self.host = Host::C64(machine);
        self.prompt.clear();
        self.prompt_at = None;
        self.holds.clear();
        self.shown = None;
        Ok(())
    }

    pub fn load_sna(&mut self, bytes: &[u8]) -> Result<(), CoreError> {
        self.ensure_spectrum().load_sna(bytes)?;
        self.arm_prompt();
        Ok(())
    }

    pub fn load_tape(&mut self, bytes: &[u8]) -> Result<(), CoreError> {
        self.ensure_spectrum().load_tape(bytes)?;
        self.arm_prompt();
        Ok(())
    }

    pub fn load_prg(&mut self, bytes: &[u8]) -> Result<(), CoreError> {
        self.c64_mut()?.load_prg(bytes)
    }

    pub fn warm_c64(&mut self, frames: u32) -> Result<(), CoreError> {
        let machine = self.c64_mut()?;
        let mut index = 0u32;
        while index < frames {
            machine.advance()?;
            index += 1;
        }
        Ok(())
    }

    fn c64_mut(&mut self) -> Result<&mut c64::Machine, CoreError> {
        match &mut self.host {
            Host::C64(machine) => Ok(machine),
            Host::Spectrum(_) => Err(CoreError::Unsupported {
                kind: "prg",
                id: 0,
            }),
        }
    }

    pub fn resume_tape(&mut self) {
        if let Host::Spectrum(machine) = &mut self.host {
            machine.resume_tape();
        }
    }

    pub fn snapshot(&self) -> Result<Vec<u8>, CoreError> {
        match &self.host {
            Host::Spectrum(machine) => machine.snapshot(),
            Host::C64(_) => Err(CoreError::Unsupported {
                kind: "snapshot",
                id: 0,
            }),
        }
    }

    pub fn take_tap(&mut self) -> Option<Vec<u8>> {
        match &mut self.host {
            Host::Spectrum(machine) => machine.take_tap(),
            Host::C64(_) => None,
        }
    }

    pub fn tap_bytes(&self) -> Option<Vec<u8>> {
        match &self.host {
            Host::Spectrum(machine) => machine.tap_bytes(),
            Host::C64(_) => None,
        }
    }

    pub fn tzx_bytes(&self) -> Option<Vec<u8>> {
        match &self.host {
            Host::Spectrum(machine) => machine.tzx_bytes(),
            Host::C64(_) => None,
        }
    }

    pub fn set_stick(&mut self, mask: u8, down: bool) {
        if let Host::Spectrum(machine) = &mut self.host {
            machine.set_stick(mask, down);
        }
    }

    pub fn set_key(&mut self, row: u8, mask: u8, down: bool) {
        if let Host::Spectrum(machine) = &mut self.host {
            machine.set_key(row, mask, down);
        }
    }

    pub fn set_c64_key(&mut self, row: u8, col: u8, down: bool) {
        if let Host::C64(machine) = &mut self.host {
            machine.set_key(row, col, down);
        }
    }

    pub fn is_booted(&self) -> bool {
        match &self.host {
            Host::Spectrum(machine) => machine.is_booted(),
            Host::C64(machine) => machine.is_booted(),
        }
    }

    pub fn reset(&mut self) {
        match &mut self.host {
            Host::Spectrum(machine) => machine.reset(),
            Host::C64(machine) => machine.reset(),
        }
        self.shown = None;
        self.arm_prompt();
    }

    pub fn take_audio(&mut self) -> Vec<f32> {
        match &mut self.host {
            Host::Spectrum(machine) => machine.take_audio(),
            Host::C64(machine) => machine.take_audio(),
        }
    }

    pub fn set_loading_sound(&mut self, on: bool) {
        if let Host::Spectrum(machine) = &mut self.host {
            machine.set_loading_sound(on);
        }
    }

    pub fn loading_sound(&self) -> bool {
        match &self.host {
            Host::Spectrum(machine) => machine.loading_sound(),
            Host::C64(_) => false,
        }
    }

    pub fn hears_loading(&self) -> bool {
        match &self.host {
            Host::Spectrum(machine) => machine.hears_loading(),
            Host::C64(_) => true,
        }
    }

    pub fn tape_edge(&self) -> usize {
        match &self.host {
            Host::Spectrum(machine) => machine.tape_edge(),
            Host::C64(_) => 0,
        }
    }

    pub fn on_display_tick(&mut self) -> Result<&Frame, CoreError> {
        let source = source_index(self.display_tick, self.pace);
        self.ensure_source(source)?;
        self.display_tick += 1;
        self.frame_ref()
    }

    pub fn rush(&mut self) -> Result<Option<Frame>, CoreError> {
        if !self.rush_cpu()? {
            return Ok(None);
        }
        Ok(Some(self.paint_latest()?))
    }

    pub fn rush_cpu(&mut self) -> Result<bool, CoreError> {
        if self.prompt_at.is_some() || self.turbo_budget() == 0 {
            return Ok(false);
        }
        self.advance_host()?;
        Ok(true)
    }

    pub fn paint_latest(&mut self) -> Result<Frame, CoreError> {
        let mut frame = self.paint_host()?;
        apply_history(&mut frame, &mut self.history)?;
        self.frame = Some(frame.clone());
        Ok(frame)
    }

    fn turbo_budget(&self) -> u32 {
        match &self.host {
            Host::Spectrum(machine) => machine.turbo_budget(),
            Host::C64(_) => 0,
        }
    }

    fn advance_host(&mut self) -> Result<(), CoreError> {
        match &mut self.host {
            Host::Spectrum(machine) => machine.advance(),
            Host::C64(machine) => machine.advance(),
        }
    }

    fn paint_host(&self) -> Result<Frame, CoreError> {
        match &self.host {
            Host::Spectrum(machine) => machine.paint(),
            Host::C64(machine) => machine.paint(),
        }
    }

    fn frame_host(&mut self) -> Result<Frame, CoreError> {
        match &mut self.host {
            Host::Spectrum(machine) => machine.frame(),
            Host::C64(machine) => machine.frame(),
        }
    }

    fn ensure_spectrum(&mut self) -> &mut Machine {
        if !matches!(self.host, Host::Spectrum(_)) {
            self.host = Host::Spectrum(Machine::new());
            self.shown = None;
        }
        match &mut self.host {
            Host::Spectrum(machine) => machine,
            Host::C64(_) => unreachable!(),
        }
    }

    fn ensure_source(&mut self, source: u64) -> Result<(), CoreError> {
        if self.shown == Some(source) {
            return Ok(());
        }
        self.store_source(source)
    }

    fn store_source(&mut self, source: u64) -> Result<(), CoreError> {
        self.apply_prompt();
        let mut frame = self.frame_host()?;
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
        let Host::Spectrum(machine) = &self.host else {
            self.prompt.clear();
            self.prompt_at = None;
            return;
        };
        if machine.is_booted() && machine.has_tape() {
            self.prompt = tape::load_prompt(machine.is_128());
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
            self.set_key(holds[index].row, holds[index].mask, true);
            index += 1;
        }
        self.holds = holds.to_vec();
    }

    fn release_holds(&mut self) {
        let mut index = 0;
        while index < self.holds.len() {
            self.set_key(self.holds[index].row, self.holds[index].mask, false);
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
