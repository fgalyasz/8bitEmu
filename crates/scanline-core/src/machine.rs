use crate::display::frame_from;
use crate::error::CoreError;
use crate::frame::Frame;
use crate::memory::Memory;
use crate::program::{self, ENTRY};
use crate::z80::{self, Cpu, Ports};

const STEP_LIMIT: u32 = 100_000;

pub struct Machine {
    cpu: Cpu,
    memory: Memory,
    ports: Ports,
}

impl Machine {
    pub fn new() -> Self {
        let mut memory = Memory::new();
        program::load(&mut memory);
        let mut cpu = Cpu::default();
        cpu.pc = ENTRY;
        Self {
            cpu,
            memory,
            ports: Ports::default(),
        }
    }

    pub fn frame(&mut self) -> Result<Frame, CoreError> {
        run_until_halt(&mut self.cpu, &mut self.memory, &mut self.ports)?;
        frame_from(&self.memory, self.ports.border)
    }

    pub fn read(&self, address: u16) -> u8 {
        self.memory.read(address)
    }
}

pub fn run_until_halt(
    cpu: &mut Cpu,
    memory: &mut Memory,
    ports: &mut Ports,
) -> Result<(), CoreError> {
    resume(cpu);
    let mut steps = 0u32;
    while !cpu.halted {
        step_bounded(cpu, memory, ports, steps)?;
        steps += 1;
    }
    Ok(())
}

fn step_bounded(
    cpu: &mut Cpu,
    memory: &mut Memory,
    ports: &mut Ports,
    steps: u32,
) -> Result<(), CoreError> {
    if steps == STEP_LIMIT {
        return Err(CoreError::StepLimit);
    }
    z80::step(cpu, memory, ports)
}

fn resume(cpu: &mut Cpu) {
    if !cpu.halted {
        return;
    }
    cpu.halted = false;
    cpu.pc = cpu.pc.wrapping_add(1);
}
