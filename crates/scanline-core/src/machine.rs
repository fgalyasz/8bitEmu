use crate::display::frame_from;
use crate::error::CoreError;
use crate::frame::Frame;
use crate::memory::Memory;
use crate::program::{self, ENTRY};
use crate::sna;
use crate::z80::{self, Cpu, Ports};

const STEP_LIMIT: u32 = 100_000;
const FRAME_CYCLES: u32 = 69_888;
const SAMPLE_RATE: u32 = 48_000;
const CPU_CLOCK: u32 = 3_500_000;

pub struct Machine {
    cpu: Cpu,
    memory: Memory,
    ports: Ports,
    booted: bool,
    audio: Vec<f32>,
    audio_acc: u32,
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
            booted: false,
            audio: Vec::new(),
            audio_acc: 0,
        }
    }

    pub fn frame(&mut self) -> Result<Frame, CoreError> {
        if self.booted {
            self.run_cpu_frame()?;
        } else {
            run_until_halt(&mut self.cpu, &mut self.memory, &mut self.ports)?;
        }
        frame_from(&self.memory, self.ports.border)
    }

    pub fn read(&self, address: u16) -> u8 {
        self.memory.read(address)
    }

    pub fn bank_byte(&self, bank: u8, offset: u16) -> u8 {
        self.memory.bank_byte(bank, offset)
    }

    pub fn border(&self) -> u8 {
        self.ports.border
    }

    pub fn take_audio(&mut self) -> Vec<f32> {
        std::mem::take(&mut self.audio)
    }

    pub fn load_rom(&mut self, bytes: &[u8], model_128: bool) -> Result<(), CoreError> {
        self.memory.load_rom(bytes, model_128)?;
        self.cpu = Cpu::default();
        self.ports = Ports::default();
        self.ports.model_128 = model_128;
        self.booted = true;
        Ok(())
    }

    pub fn load_sna(&mut self, bytes: &[u8]) -> Result<(), CoreError> {
        sna::load(&mut self.cpu, &mut self.memory, &mut self.ports, bytes)?;
        self.booted = true;
        Ok(())
    }

    pub fn set_key(&mut self, row: u8, mask: u8, down: bool) {
        self.ports.set_key(row, mask, down);
    }

    pub fn reset(&mut self) {
        if !self.booted {
            *self = Self::new();
            return;
        }
        self.cpu = Cpu::default();
        self.ports.locked = false;
        self.ports.border = 0;
        self.ports.speaker = 0;
        self.memory.page(0);
    }

    fn run_cpu_frame(&mut self) -> Result<(), CoreError> {
        self.audio.clear();
        self.audio_acc = 0;
        if self.cpu.iff1 && !self.cpu.arm_ei {
            let took = z80::accept_interrupt(&mut self.cpu, &mut self.memory);
            self.mix(took);
        }
        let mut done = 0u32;
        while done < FRAME_CYCLES {
            let took = self.step_or_halt()?;
            self.retire_ei();
            self.mix(took);
            done += took;
        }
        Ok(())
    }

    fn step_or_halt(&mut self) -> Result<u32, CoreError> {
        if self.cpu.halted {
            return Ok(4);
        }
        z80::step(&mut self.cpu, &mut self.memory, &mut self.ports)
    }

    fn retire_ei(&mut self) {
        if !self.cpu.arm_ei {
            return;
        }
        self.cpu.iff1 = true;
        self.cpu.iff2 = true;
        self.cpu.arm_ei = false;
    }

    fn mix(&mut self, cycles: u32) {
        let beeper = if self.ports.speaker == 0 { -0.2 } else { 0.2 };
        let tone = self.ports.ay.sample();
        self.ports.ay.tick(cycles / 2);
        let mixed = (beeper + tone).clamp(-1.0, 1.0);
        self.audio_acc += cycles.saturating_mul(SAMPLE_RATE);
        let count = self.audio_acc / CPU_CLOCK;
        self.audio_acc %= CPU_CLOCK;
        push_samples(&mut self.audio, mixed, count);
    }
}

pub fn run_until_halt(cpu: &mut Cpu, memory: &mut Memory, ports: &mut Ports) -> Result<(), CoreError> {
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
    z80::step(cpu, memory, ports)?;
    Ok(())
}

fn resume(cpu: &mut Cpu) {
    if !cpu.halted {
        return;
    }
    cpu.halted = false;
    cpu.pc = cpu.pc.wrapping_add(1);
}

fn push_samples(audio: &mut Vec<f32>, sample: f32, count: u32) {
    let mut index = 0u32;
    while index < count {
        audio.push(sample);
        index += 1;
    }
}
