use crate::error::CoreError;
use crate::memory::Memory;
use crate::z80::{Cpu, Ports};

pub fn run(
    cpu: &mut Cpu,
    memory: &mut Memory,
    ports: &mut Ports,
    ix: bool,
) -> Result<u32, CoreError> {
    let mut cycles = 4u32;
    let mut use_ix = ix;
    loop {
        let opcode = super::fetch_m1(cpu, memory);
        if opcode == 0xDD {
            use_ix = true;
            cycles += 4;
            continue;
        }
        if opcode == 0xFD {
            use_ix = false;
            cycles += 4;
            continue;
        }
        return finish(cpu, memory, ports, use_ix, opcode, cycles);
    }
}

fn finish(
    cpu: &mut Cpu,
    memory: &mut Memory,
    ports: &mut Ports,
    ix: bool,
    opcode: u8,
    cycles: u32,
) -> Result<u32, CoreError> {
    if opcode == 0xED {
        return Ok(cycles + super::extended::run(cpu, memory, ports));
    }
    if opcode == 0xCB {
        return Ok(cycles + indexed_bits(cpu, memory, ix));
    }
    if uses_hl_memory(opcode) {
        return indexed_memory(cpu, memory, ports, ix, opcode, cycles);
    }
    indexed_pair(cpu, memory, ports, ix, opcode, cycles)
}

fn indexed_bits(cpu: &mut Cpu, memory: &mut Memory, ix: bool) -> u32 {
    let displacement = super::fetch_byte(cpu, memory) as i8;
    let opcode = super::fetch_byte(cpu, memory);
    let base = if ix { cpu.ix } else { cpu.iy };
    let address = base.wrapping_add(displacement as u16);
    super::bits::apply_indexed(cpu, memory, opcode, address)
}

fn indexed_memory(
    cpu: &mut Cpu,
    memory: &mut Memory,
    ports: &mut Ports,
    ix: bool,
    opcode: u8,
    cycles: u32,
) -> Result<u32, CoreError> {
    let displacement = super::fetch_byte(cpu, memory) as i8;
    let base = if ix { cpu.ix } else { cpu.iy };
    cpu.index_addr = base.wrapping_add(displacement as u16);
    cpu.index_mode = if ix { 3 } else { 4 };
    let result = super::execute(cpu, memory, ports, opcode);
    cpu.index_mode = 0;
    result?;
    Ok(cycles + memory_cycles(opcode))
}

fn indexed_pair(
    cpu: &mut Cpu,
    memory: &mut Memory,
    ports: &mut Ports,
    ix: bool,
    opcode: u8,
    cycles: u32,
) -> Result<u32, CoreError> {
    cpu.index_mode = if ix { 1 } else { 2 };
    let result = super::execute(cpu, memory, ports, opcode);
    cpu.index_mode = 0;
    result?;
    Ok(cycles + super::unprefixed_cycles(opcode, cpu.branched))
}

fn memory_cycles(opcode: u8) -> u32 {
    if opcode == 0x36 {
        return 15;
    }
    super::unprefixed_cycles(opcode, true) + 8
}

fn uses_hl_memory(opcode: u8) -> bool {
    if opcode == 0x76 {
        return false;
    }
    let source = opcode & 7;
    let destination = (opcode >> 3) & 7;
    if (0x40..0xC0).contains(&opcode) && (source == 6 || destination == 6) {
        return true;
    }
    destination == 6 && (opcode & 0xC7 == 0x04 || opcode & 0xC7 == 0x05 || opcode & 0xC7 == 0x06)
}
