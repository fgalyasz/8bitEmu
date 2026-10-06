use crate::memory::Memory;
use crate::z80::Cpu;

pub fn run(cpu: &mut Cpu, memory: &mut Memory) -> u32 {
    let opcode = super::fetch_m1(cpu, memory);
    apply(cpu, memory, opcode, None);
    cb_cycles(opcode, false)
}

pub fn apply_indexed(cpu: &mut Cpu, memory: &mut Memory, opcode: u8, address: u16) -> u32 {
    apply(cpu, memory, opcode, Some(address));
    cb_cycles(opcode, true)
}

fn apply(cpu: &mut Cpu, memory: &mut Memory, opcode: u8, address: Option<u16>) {
    let index = opcode & 7;
    let value = read_operand(cpu, memory, index, address);
    if opcode < 0x40 {
        let (result, flags) = rotate(opcode, value, cpu.f);
        cpu.f = flags;
        store(cpu, memory, index, result, address);
        return;
    }
    if opcode < 0x80 {
        cpu.f = bit_flags((opcode >> 3) & 7, value, cpu.f);
        return;
    }
    let bit = (opcode >> 3) & 7;
    let result = if opcode < 0xC0 {
        value & !(1 << bit)
    } else {
        value | (1 << bit)
    };
    store(cpu, memory, index, result, address);
}

fn read_operand(cpu: &Cpu, memory: &Memory, index: u8, address: Option<u16>) -> u8 {
    if let Some(addr) = address {
        return memory.read(addr);
    }
    super::read_reg(cpu, memory, index)
}

fn store(cpu: &mut Cpu, memory: &mut Memory, index: u8, value: u8, address: Option<u16>) {
    if let Some(addr) = address {
        memory.write(addr, value);
        copy_register(cpu, memory, index, value);
        return;
    }
    super::write_reg(cpu, memory, index, value);
}

fn copy_register(cpu: &mut Cpu, memory: &mut Memory, index: u8, value: u8) {
    if index == 6 {
        return;
    }
    super::write_reg(cpu, memory, index, value);
}

fn rotate(opcode: u8, value: u8, flags: u8) -> (u8, u8) {
    let carry = flags & super::CARRY;
    match (opcode >> 3) & 7 {
        0 => rlc(value),
        1 => rrc(value),
        2 => rl(value, carry),
        3 => rr(value, carry),
        4 => sla(value),
        5 => sra(value),
        6 => sll(value),
        _ => srl(value),
    }
}

fn rlc(value: u8) -> (u8, u8) {
    let carry = value >> 7;
    finish((value << 1) | carry, carry)
}

fn rrc(value: u8) -> (u8, u8) {
    let carry = value & 1;
    finish((value >> 1) | (carry << 7), carry)
}

fn rl(value: u8, carry: u8) -> (u8, u8) {
    let carry_out = value >> 7;
    finish((value << 1) | (carry & 1), carry_out)
}

fn rr(value: u8, carry: u8) -> (u8, u8) {
    let carry_out = value & 1;
    finish((value >> 1) | ((carry & 1) << 7), carry_out)
}

fn sla(value: u8) -> (u8, u8) {
    finish(value << 1, value >> 7)
}

fn sra(value: u8) -> (u8, u8) {
    finish((value >> 1) | (value & 0x80), value & 1)
}

fn sll(value: u8) -> (u8, u8) {
    finish((value << 1) | 1, value >> 7)
}

fn srl(value: u8) -> (u8, u8) {
    finish(value >> 1, value & 1)
}

fn finish(result: u8, carry: u8) -> (u8, u8) {
    let flags = super::sign_zero(result) | super::parity(result) | (carry & 1);
    (result, flags)
}

fn bit_flags(bit: u8, value: u8, flags: u8) -> u8 {
    let set = (value & (1 << bit)) != 0;
    let mut next = super::HALF | (flags & super::CARRY);
    if !set {
        next |= super::ZERO | super::PARITY;
    }
    if bit == 7 && set {
        next |= super::SIGN;
    }
    next
}

fn cb_cycles(opcode: u8, indexed: bool) -> u32 {
    if indexed {
        return if opcode & 0xC0 == 0x40 { 16 } else { 19 };
    }
    let memory = opcode & 7 == 6;
    if opcode & 0xC0 == 0x40 && memory {
        return 12;
    }
    if memory { 15 } else { 8 }
}
