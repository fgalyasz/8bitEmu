use crate::memory::Memory;
use crate::z80::{Cpu, Ports};

pub fn run(cpu: &mut Cpu, memory: &mut Memory, ports: &mut Ports) -> u32 {
    let opcode = canonical(super::fetch_m1(cpu, memory));
    if let Some(cycles) = block(cpu, memory, opcode) {
        return cycles;
    }
    if let Some(cycles) = io_or_math(cpu, memory, ports, opcode) {
        return cycles;
    }
    special(cpu, memory, opcode)
}

fn canonical(opcode: u8) -> u8 {
    match opcode {
        0x4C | 0x54 | 0x5C | 0x64 | 0x6C | 0x74 | 0x7C => 0x44,
        0x55 | 0x5D | 0x65 | 0x6D | 0x75 | 0x7D => 0x45,
        0x4E => 0x46,
        0x76 => 0x56,
        0x7E => 0x5E,
        other => other,
    }
}

fn block(cpu: &mut Cpu, memory: &mut Memory, opcode: u8) -> Option<u32> {
    match opcode {
        0xA0 => Some(transfer(cpu, memory, true, false)),
        0xA8 => Some(transfer(cpu, memory, false, false)),
        0xB0 => Some(transfer(cpu, memory, true, true)),
        0xB8 => Some(transfer(cpu, memory, false, true)),
        0xA1 => Some(compare(cpu, memory, true, false)),
        0xA9 => Some(compare(cpu, memory, false, false)),
        0xB1 => Some(compare(cpu, memory, true, true)),
        0xB9 => Some(compare(cpu, memory, false, true)),
        _ => None,
    }
}

fn io_or_math(cpu: &mut Cpu, memory: &mut Memory, ports: &mut Ports, opcode: u8) -> Option<u32> {
    if !(0x40..=0x7F).contains(&opcode) {
        return None;
    }
    let which = (opcode >> 4) & 3;
    let index = (opcode >> 3) & 7;
    match opcode & 0x0F {
        0x00 | 0x08 => {
            input_c(cpu, memory, ports, index);
            Some(12)
        }
        0x01 | 0x09 => {
            output_c(cpu, memory, ports, index);
            Some(12)
        }
        0x02 => {
            add_hl_carry(cpu, which, true);
            Some(15)
        }
        0x0A => {
            add_hl_carry(cpu, which, false);
            Some(15)
        }
        0x03 => {
            store_pair(cpu, memory, which);
            Some(20)
        }
        0x0B => {
            load_pair(cpu, memory, which);
            Some(20)
        }
        _ => None,
    }
}

fn special(cpu: &mut Cpu, memory: &mut Memory, opcode: u8) -> u32 {
    match opcode {
        0x44 => neg(cpu),
        0x45 | 0x4D => retn(cpu, memory),
        0x46 => cpu.im = 0,
        0x56 => cpu.im = 1,
        0x5E => cpu.im = 2,
        0x47 => cpu.i = cpu.a,
        0x4F => cpu.r = cpu.a,
        0x57 => load_special(cpu, cpu.i),
        0x5F => load_special(cpu, cpu.r),
        0x67 => rrd(cpu, memory),
        0x6F => rld(cpu, memory),
        _ => return 8,
    }
    special_cycles(opcode)
}

fn special_cycles(opcode: u8) -> u32 {
    match opcode {
        0x45 | 0x4D => 14,
        0x47 | 0x4F | 0x57 | 0x5F => 9,
        0x67 | 0x6F => 18,
        _ => 8,
    }
}

fn transfer(cpu: &mut Cpu, memory: &mut Memory, increment: bool, repeat: bool) -> u32 {
    let hl = super::read_pair(cpu, 2);
    let de = super::read_pair(cpu, 1);
    memory.write(de, memory.read(hl));
    step_pair(cpu, 2, increment);
    step_pair(cpu, 1, increment);
    let bc = super::read_pair(cpu, 0).wrapping_sub(1);
    super::write_pair(cpu, 0, bc);
    let mut flags = cpu.f & 0xC1;
    if bc != 0 {
        flags |= super::PARITY;
    }
    cpu.f = flags;
    repeat_cycles(cpu, repeat && bc != 0)
}

fn compare(cpu: &mut Cpu, memory: &mut Memory, increment: bool, repeat: bool) -> u32 {
    let hl = super::read_pair(cpu, 2);
    let value = memory.read(hl);
    let saved = cpu.a;
    super::apply_alu(cpu, 7, value);
    cpu.a = saved;
    let bc = super::read_pair(cpu, 0).wrapping_sub(1);
    super::write_pair(cpu, 0, bc);
    step_pair(cpu, 2, increment);
    if bc != 0 {
        cpu.f |= super::PARITY;
    } else {
        cpu.f &= !super::PARITY;
    }
    let found = (cpu.f & super::ZERO) != 0;
    repeat_cycles(cpu, repeat && bc != 0 && !found)
}

fn repeat_cycles(cpu: &mut Cpu, again: bool) -> u32 {
    if again {
        cpu.pc = cpu.pc.wrapping_sub(2);
        return 21;
    }
    16
}

fn step_pair(cpu: &mut Cpu, which: u8, increment: bool) {
    let value = super::read_pair(cpu, which);
    let next = if increment {
        value.wrapping_add(1)
    } else {
        value.wrapping_sub(1)
    };
    super::write_pair(cpu, which, next);
}

fn input_c(cpu: &mut Cpu, memory: &mut Memory, ports: &Ports, index: u8) {
    let value = ports.input(super::pair(cpu.b, cpu.c));
    if index != 6 {
        super::write_reg(cpu, memory, index, value);
    }
    cpu.f = super::sign_zero(value) | super::parity(value) | (cpu.f & super::CARRY);
}

fn output_c(cpu: &Cpu, memory: &mut Memory, ports: &mut Ports, index: u8) {
    let value = if index == 6 {
        0
    } else {
        super::read_reg(cpu, memory, index)
    };
    ports.output(memory, super::pair(cpu.b, cpu.c), value);
}

fn add_hl_carry(cpu: &mut Cpu, which: u8, subtract: bool) {
    let left = super::read_pair(cpu, 2);
    let right = super::read_pair(cpu, which);
    let carry = u16::from(cpu.f & super::CARRY);
    let (result, carry_out, half, overflow) = if subtract {
        sbc16(left, right, carry)
    } else {
        adc16(left, right, carry)
    };
    super::write_pair(cpu, 2, result);
    cpu.f = wide_flags(result, carry_out, half, overflow, subtract);
}

fn adc16(left: u16, right: u16, carry: u16) -> (u16, bool, bool, bool) {
    let sum = u32::from(left) + u32::from(right) + u32::from(carry);
    let result = sum as u16;
    let half = (left & 0x0FFF) + (right & 0x0FFF) + carry > 0x0FFF;
    let overflow = same_sign16(left, right) && sign_differs16(left, result);
    (result, sum > 0xFFFF, half, overflow)
}

fn sbc16(left: u16, right: u16, carry: u16) -> (u16, bool, bool, bool) {
    let diff = i32::from(left) - i32::from(right) - i32::from(carry);
    let result = diff as u16;
    let half = i32::from(left & 0x0FFF) - i32::from(right & 0x0FFF) - i32::from(carry) < 0;
    let overflow = !same_sign16(left, right) && sign_differs16(left, result);
    (result, diff < 0, half, overflow)
}

fn wide_flags(result: u16, carry: bool, half: bool, overflow: bool, subtract: bool) -> u8 {
    let mut flags = sign_zero16(result);
    if carry {
        flags |= super::CARRY;
    }
    if half {
        flags |= super::HALF;
    }
    if overflow {
        flags |= super::PARITY;
    }
    if subtract {
        flags |= super::NEGATIVE;
    }
    flags
}

fn sign_zero16(value: u16) -> u8 {
    let mut flags = 0u8;
    if value & 0x8000 != 0 {
        flags |= super::SIGN;
    }
    if value == 0 {
        flags |= super::ZERO;
    }
    flags
}

fn same_sign16(left: u16, right: u16) -> bool {
    (left ^ right) & 0x8000 == 0
}

fn sign_differs16(left: u16, result: u16) -> bool {
    (left ^ result) & 0x8000 != 0
}

fn store_pair(cpu: &mut Cpu, memory: &mut Memory, which: u8) {
    let address = super::peek_word(memory, cpu.pc);
    cpu.pc = cpu.pc.wrapping_add(2);
    super::write_word(memory, address, super::read_pair(cpu, which));
}

fn load_pair(cpu: &mut Cpu, memory: &mut Memory, which: u8) {
    let address = super::peek_word(memory, cpu.pc);
    cpu.pc = cpu.pc.wrapping_add(2);
    super::write_pair(cpu, which, super::peek_word(memory, address));
}

fn neg(cpu: &mut Cpu) {
    let value = cpu.a;
    cpu.a = 0;
    super::apply_alu(cpu, 2, value);
}

fn retn(cpu: &mut Cpu, memory: &mut Memory) {
    cpu.pc = super::pop(cpu, memory);
    cpu.iff1 = cpu.iff2;
}

fn load_special(cpu: &mut Cpu, value: u8) {
    cpu.a = value;
    let mut flags = super::sign_zero(value) | (cpu.f & super::CARRY);
    if cpu.iff2 {
        flags |= super::PARITY;
    }
    cpu.f = flags;
}

fn rrd(cpu: &mut Cpu, memory: &mut Memory) {
    let address = super::pair(cpu.h, cpu.l);
    let mem = memory.read(address);
    let next_a = (cpu.a & 0xF0) | (mem & 0x0F);
    let next_mem = ((cpu.a & 0x0F) << 4) | (mem >> 4);
    cpu.a = next_a;
    memory.write(address, next_mem);
    digit_flags(cpu);
}

fn rld(cpu: &mut Cpu, memory: &mut Memory) {
    let address = super::pair(cpu.h, cpu.l);
    let mem = memory.read(address);
    let next_a = (cpu.a & 0xF0) | (mem >> 4);
    let next_mem = (mem << 4) | (cpu.a & 0x0F);
    cpu.a = next_a;
    memory.write(address, next_mem);
    digit_flags(cpu);
}

fn digit_flags(cpu: &mut Cpu) {
    cpu.f = super::sign_zero(cpu.a) | super::parity(cpu.a) | (cpu.f & super::CARRY);
}
