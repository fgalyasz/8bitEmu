use crate::error::CoreError;
use crate::frame::Frame;

use super::cpu::{self, Bus, Cpu};
use super::mem::Map;
use super::paint;

const FRAME_CYCLES: u32 = 19_656;
const STEP_LIMIT: u32 = 200_000;

pub struct Machine {
    cpu: Cpu,
    map: Map,
    booted: bool,
}

impl Machine {
    pub fn new() -> Self {
        Self {
            cpu: Cpu::default(),
            map: Map::new(),
            booted: false,
        }
    }

    pub fn load_roms(
        &mut self,
        kernal: &[u8],
        basic: &[u8],
        chargen: &[u8],
    ) -> Result<(), CoreError> {
        self.map.load_roms(kernal, basic, chargen)?;
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

    pub fn poke_io(&mut self, address: u16, value: u8) {
        self.map.write(address, value);
    }

    pub fn pc(&self) -> u16 {
        self.cpu.pc
    }

    pub fn step_instruction(&mut self) -> Result<u32, CoreError> {
        let used = cpu::step(&mut self.cpu, &mut self.map)?;
        self.map.tick(used);
        if self.map.irq_line() {
            cpu::trigger_irq(&mut self.cpu);
        }
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
        Vec::new()
    }

    fn run_frame(&mut self) -> Result<(), CoreError> {
        let mut cycles = 0u32;
        let mut steps = 0u32;
        while cycles < FRAME_CYCLES {
            if steps >= STEP_LIMIT {
                return Err(CoreError::StepLimit);
            }
            let used = cpu::step(&mut self.cpu, &mut self.map)?;
            self.map.tick(used);
            if self.map.irq_line() {
                cpu::trigger_irq(&mut self.cpu);
            }
            cycles += used;
            steps += 1;
        }
        Ok(())
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
