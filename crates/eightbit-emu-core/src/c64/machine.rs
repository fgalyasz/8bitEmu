use crate::error::CoreError;
use crate::frame::Frame;

use super::cpu::{self, Bus, Cpu};
use super::mem::Map;
use super::paint;
use super::prg;

const FRAME_CYCLES: u32 = 19_656;
const STEP_LIMIT: u32 = 200_000;
const SAMPLE_RATE: u32 = 48_000;
const AUDIO_CLOCK: u32 = FRAME_CYCLES * 50;

pub struct Machine {
    cpu: Cpu,
    map: Map,
    booted: bool,
    audio: Vec<f32>,
    audio_acc: u32,
}

impl Machine {
    pub fn new() -> Self {
        Self {
            cpu: Cpu::default(),
            map: Map::new(),
            booted: false,
            audio: Vec::new(),
            audio_acc: 0,
        }
    }

    pub fn load_roms(
        &mut self,
        kernal: &[u8],
        basic: &[u8],
        chargen: &[u8],
    ) -> Result<(), CoreError> {
        self.map.load_roms(kernal, basic, chargen)?;
        self.map.vic[0x18] = 0x14;
        self.map.vic[0x20] = 0x0E;
        self.map.vic[0x21] = 0x06;
        cpu::reset(&mut self.cpu, &mut self.map);
        self.booted = true;
        Ok(())
    }

    pub fn is_booted(&self) -> bool {
        self.booted
    }

    pub fn reset(&mut self) {
        if self.booted {
            cpu::reset(&mut self.cpu, &mut self.map);
        }
    }

    pub fn set_key(&mut self, row: u8, col: u8, down: bool) {
        self.map.set_key(row, col, down);
    }

    pub fn read(&mut self, address: u16) -> u8 {
        self.map.read(address)
    }

    pub fn write_ram(&mut self, address: u16, value: u8) {
        self.map.write_ram(address, value);
    }

    pub fn load_prg(&mut self, bytes: &[u8]) -> Result<(), CoreError> {
        if !self.booted {
            return Err(CoreError::Unsupported {
                kind: "prg",
                id: 0,
            });
        }
        let (address, payload) = prg::parse(bytes)?;
        prg::write_payload(&mut self.map, address, payload)?;
        self.finish_prg(address, payload.len())
    }

    pub fn poke_io(&mut self, address: u16, value: u8) {
        self.map.write(address, value);
    }

    pub fn pc(&self) -> u16 {
        self.cpu.pc
    }

    pub fn step_instruction(&mut self) -> Result<u32, CoreError> {
        let used = cpu::step(&mut self.cpu, &mut self.map)?;
        self.retire(used);
        Ok(used)
    }

    pub fn frame(&mut self) -> Result<Frame, CoreError> {
        self.run_frame()?;
        paint::frame(&self.map)
    }

    pub fn advance(&mut self) -> Result<(), CoreError> {
        self.run_frame()
    }

    pub fn paint(&self) -> Result<Frame, CoreError> {
        paint::frame(&self.map)
    }

    pub fn take_audio(&mut self) -> Vec<f32> {
        std::mem::take(&mut self.audio)
    }

    fn finish_prg(&mut self, address: u16, length: usize) -> Result<(), CoreError> {
        let end = prg::payload_end(address, length)?;
        if prg::is_basic_load(address) {
            prg::link_basic(&mut self.map, end);
            prg::queue_run(&mut self.map);
            return Ok(());
        }
        self.cpu.pc = address;
        Ok(())
    }

    fn retire(&mut self, cycles: u32) {
        self.map.tick(cycles);
        if self.map.irq_line() {
            cpu::trigger_irq(&mut self.cpu);
        }
        self.mix(cycles);
    }

    fn mix(&mut self, cycles: u32) {
        let sample = self.map.sid.sample();
        self.audio_acc += cycles.saturating_mul(SAMPLE_RATE);
        let count = self.audio_acc / AUDIO_CLOCK;
        self.audio_acc %= AUDIO_CLOCK;
        push_samples(&mut self.audio, sample, count);
    }

    fn run_frame(&mut self) -> Result<(), CoreError> {
        let mut cycles = 0u32;
        let mut steps = 0u32;
        while cycles < FRAME_CYCLES {
            if steps >= STEP_LIMIT {
                return Err(CoreError::StepLimit);
            }
            let used = cpu::step(&mut self.cpu, &mut self.map)?;
            self.retire(used);
            cycles += used;
            steps += 1;
        }
        Ok(())
    }
}

fn push_samples(audio: &mut Vec<f32>, sample: f32, count: u32) {
    let mut index = 0u32;
    while index < count {
        audio.push(sample);
        index += 1;
    }
}

impl Bus for Map {
    fn read(&mut self, addr: u16) -> u8 {
        Map::read(self, addr)
    }

    fn write(&mut self, addr: u16, value: u8) {
        Map::write(self, addr, value)
    }
}
