use crate::error::CoreError;
use crate::memory::Memory;
use crate::z80::{Cpu, Ports};

const HEADER: usize = 27;
const RAM_48: usize = 48 * 1024;
const BANK: usize = 16 * 1024;
const SNA_48: usize = HEADER + RAM_48;
const SNA_128: usize = 131_103;

pub fn load(
    cpu: &mut Cpu,
    memory: &mut Memory,
    ports: &mut Ports,
    bytes: &[u8],
) -> Result<(), CoreError> {
    if bytes.len() == SNA_48 {
        return load_48(cpu, memory, ports, bytes);
    }
    if bytes.len() == SNA_128 {
        return load_128(cpu, memory, ports, bytes);
    }
    Err(CoreError::ImageLength {
        name: "snapshot",
        actual: bytes.len(),
    })
}

fn load_48(cpu: &mut Cpu, memory: &mut Memory, ports: &mut Ports, bytes: &[u8]) -> Result<(), CoreError> {
    apply_header(cpu, ports, bytes);
    memory.use_banks();
    write_linear(memory, 0x4000, &bytes[HEADER..HEADER + RAM_48]);
    cpu.pc = pop_pc(cpu, memory);
    Ok(())
}

fn load_128(cpu: &mut Cpu, memory: &mut Memory, ports: &mut Ports, bytes: &[u8]) -> Result<(), CoreError> {
    apply_header(cpu, ports, bytes);
    let trailer = SNA_128 - 4;
    let port = bytes[trailer + 2];
    memory.use_banks();
    memory.page(port);
    ports.model_128 = true;
    ports.locked = port & 0x20 != 0;
    write_linear(memory, 0x4000, &bytes[HEADER..HEADER + RAM_48]);
    write_extra_banks(memory, bytes, port & 7);
    cpu.pc = word(bytes[trailer], bytes[trailer + 1]);
    Ok(())
}

fn apply_header(cpu: &mut Cpu, ports: &mut Ports, bytes: &[u8]) {
    cpu.i = bytes[0];
    cpu.l2 = bytes[1];
    cpu.h2 = bytes[2];
    cpu.e2 = bytes[3];
    cpu.d2 = bytes[4];
    cpu.c2 = bytes[5];
    cpu.b2 = bytes[6];
    cpu.f2 = bytes[7];
    cpu.a2 = bytes[8];
    cpu.l = bytes[9];
    cpu.h = bytes[10];
    cpu.e = bytes[11];
    cpu.d = bytes[12];
    cpu.c = bytes[13];
    cpu.b = bytes[14];
    cpu.iy = word(bytes[15], bytes[16]);
    cpu.ix = word(bytes[17], bytes[18]);
    let enabled = bytes[19] & 0x04 != 0;
    cpu.iff1 = enabled;
    cpu.iff2 = enabled;
    cpu.r = bytes[20];
    cpu.f = bytes[21];
    cpu.a = bytes[22];
    cpu.sp = word(bytes[23], bytes[24]);
    cpu.im = bytes[25] & 0x03;
    ports.border = bytes[26] & 0x07;
    cpu.halted = false;
    cpu.arm_ei = false;
    cpu.index_mode = 0;
}

fn write_extra_banks(memory: &mut Memory, bytes: &[u8], paged: u8) {
    let mut cursor = HEADER + RAM_48;
    let mut bank = 0u8;
    while bank < 8 {
        if bank != 5 && bank != 2 && bank != paged {
            memory.write_bank(bank, &bytes[cursor..cursor + BANK]);
            cursor += BANK;
        }
        bank += 1;
    }
}

fn write_linear(memory: &mut Memory, mut address: u16, bytes: &[u8]) {
    let mut index = 0;
    while index < bytes.len() {
        memory.write(address, bytes[index]);
        address = address.wrapping_add(1);
        index += 1;
    }
}

fn pop_pc(cpu: &mut Cpu, memory: &Memory) -> u16 {
    let pc = word(memory.read(cpu.sp), memory.read(cpu.sp.wrapping_add(1)));
    cpu.sp = cpu.sp.wrapping_add(2);
    pc
}

fn word(low: u8, high: u8) -> u16 {
    u16::from(low) | (u16::from(high) << 8)
}
