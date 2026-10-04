use crate::error::CoreError;
use crate::memory::Memory;

const SIGN: u8 = 0x80;
const ZERO: u8 = 0x40;
const HALF: u8 = 0x10;
const PARITY: u8 = 0x04;
const NEGATIVE: u8 = 0x02;
const CARRY: u8 = 0x01;

#[derive(Default)]
pub struct Cpu {
    pub pc: u16,
    pub sp: u16,
    pub a: u8,
    pub f: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,
    pub halted: bool,
}

#[derive(Default)]
pub struct Ports {
    pub border: u8,
}

pub fn step(cpu: &mut Cpu, memory: &mut Memory, ports: &mut Ports) -> Result<(), CoreError> {
    let opcode = memory.read(cpu.pc);
    cpu.pc = cpu.pc.wrapping_add(1);
    execute(cpu, memory, ports, opcode)
}

fn execute(
    cpu: &mut Cpu,
    memory: &mut Memory,
    ports: &mut Ports,
    opcode: u8,
) -> Result<(), CoreError> {
    if run_fixed(cpu, memory, ports, opcode) || run_pattern(cpu, memory, opcode) {
        return Ok(());
    }
    Err(CoreError::Opcode { opcode })
}

fn run_fixed(cpu: &mut Cpu, memory: &mut Memory, ports: &mut Ports, opcode: u8) -> bool {
    run_move(cpu, memory, opcode) || run_flow(cpu, memory, ports, opcode)
}

fn run_move(cpu: &mut Cpu, memory: &mut Memory, opcode: u8) -> bool {
    match opcode {
        0x02 => store_a(cpu, memory, pair(cpu.b, cpu.c)),
        0x12 => store_a(cpu, memory, pair(cpu.d, cpu.e)),
        0x0A => load_a(cpu, memory, pair(cpu.b, cpu.c)),
        0x1A => load_a(cpu, memory, pair(cpu.d, cpu.e)),
        0x32 => store_absolute(cpu, memory),
        0x3A => load_absolute(cpu, memory),
        0xEB => exchange_de_hl(cpu),
        _ => return false,
    }
    true
}

fn run_flow(cpu: &mut Cpu, memory: &mut Memory, ports: &mut Ports, opcode: u8) -> bool {
    match opcode {
        0x00 => true,
        0x10 => djnz(cpu, memory),
        0x18 => jump_relative(cpu, memory, true),
        0x76 => halt(cpu),
        0xC3 => jump_absolute(cpu, memory, true),
        0xD3 => out_port(cpu, memory, ports),
        0xDB => in_port(cpu, memory),
        0xF9 => load_sp(cpu),
        _ => false,
    }
}

fn run_pattern(cpu: &mut Cpu, memory: &mut Memory, opcode: u8) -> bool {
    run_pair(cpu, memory, opcode)
        || run_byte(cpu, memory, opcode)
        || run_branch(cpu, memory, opcode)
}

fn run_pair(cpu: &mut Cpu, memory: &mut Memory, opcode: u8) -> bool {
    let kind = pair_kind(opcode);
    if kind == 0 {
        return false;
    }
    apply_pair(cpu, memory, opcode, kind);
    true
}

fn pair_kind(opcode: u8) -> u8 {
    if opcode & 0xCF == 0x01 {
        return 1;
    }
    if opcode & 0xCF == 0x03 {
        return 2;
    }
    if opcode & 0xCF == 0x0B {
        return 3;
    }
    0
}

fn apply_pair(cpu: &mut Cpu, memory: &mut Memory, opcode: u8, kind: u8) {
    let index = (opcode >> 4) & 3;
    if kind == 1 {
        load_pair(cpu, memory, index);
        return;
    }
    adjust_pair(cpu, index, kind == 2);
}

fn run_byte(cpu: &mut Cpu, memory: &mut Memory, opcode: u8) -> bool {
    let index = (opcode >> 3) & 7;
    if opcode & 0xC7 == 0x04 {
        inc_reg(cpu, memory, index);
        return true;
    }
    if opcode & 0xC7 == 0x05 {
        dec_reg(cpu, memory, index);
        return true;
    }
    if opcode & 0xC7 == 0x06 {
        load_immediate(cpu, memory, index);
        return true;
    }
    copy_or_alu(cpu, memory, opcode)
}

fn copy_or_alu(cpu: &mut Cpu, memory: &mut Memory, opcode: u8) -> bool {
    if (0x40..0x80).contains(&opcode) {
        copy_reg(cpu, memory, opcode);
        return true;
    }
    if (0x80..0xC0).contains(&opcode) {
        alu_from_reg(cpu, memory, opcode);
        return true;
    }
    false
}

fn run_branch(cpu: &mut Cpu, memory: &mut Memory, opcode: u8) -> bool {
    if opcode & 0xE7 == 0x20 {
        jump_relative(cpu, memory, flag_met(cpu.f, (opcode >> 3) & 3));
        return true;
    }
    if opcode & 0xC7 == 0xC2 {
        jump_absolute(cpu, memory, flag_met(cpu.f, (opcode >> 3) & 7));
        return true;
    }
    if opcode & 0xC7 == 0xC6 {
        alu_immediate(cpu, memory, (opcode >> 3) & 7);
        return true;
    }
    false
}

fn store_a(cpu: &Cpu, memory: &mut Memory, address: u16) {
    memory.write(address, cpu.a);
}

fn load_a(cpu: &mut Cpu, memory: &Memory, address: u16) {
    cpu.a = memory.read(address);
}

fn store_absolute(cpu: &mut Cpu, memory: &mut Memory) {
    let address = peek_word(memory, cpu.pc);
    cpu.pc = cpu.pc.wrapping_add(2);
    memory.write(address, cpu.a);
}

fn load_absolute(cpu: &mut Cpu, memory: &Memory) {
    let address = peek_word(memory, cpu.pc);
    cpu.pc = cpu.pc.wrapping_add(2);
    cpu.a = memory.read(address);
}

fn exchange_de_hl(cpu: &mut Cpu) {
    let d = cpu.d;
    let e = cpu.e;
    cpu.d = cpu.h;
    cpu.e = cpu.l;
    cpu.h = d;
    cpu.l = e;
}

fn djnz(cpu: &mut Cpu, memory: &mut Memory) -> bool {
    cpu.b = cpu.b.wrapping_sub(1);
    jump_relative(cpu, memory, cpu.b != 0);
    true
}

fn halt(cpu: &mut Cpu) -> bool {
    cpu.halted = true;
    cpu.pc = cpu.pc.wrapping_sub(1);
    true
}

fn load_sp(cpu: &mut Cpu) -> bool {
    cpu.sp = pair(cpu.h, cpu.l);
    true
}

fn out_port(cpu: &mut Cpu, memory: &Memory, ports: &mut Ports) -> bool {
    let port = memory.read(cpu.pc);
    cpu.pc = cpu.pc.wrapping_add(1);
    if port == 0xFE {
        ports.border = cpu.a & 7;
    }
    true
}

fn in_port(cpu: &mut Cpu, memory: &Memory) -> bool {
    let _port = memory.read(cpu.pc);
    cpu.pc = cpu.pc.wrapping_add(1);
    cpu.a = 0xFF;
    true
}

fn load_pair(cpu: &mut Cpu, memory: &Memory, index: u8) {
    write_pair(cpu, index, peek_word(memory, cpu.pc));
    cpu.pc = cpu.pc.wrapping_add(2);
}

fn adjust_pair(cpu: &mut Cpu, index: u8, increment: bool) {
    let value = read_pair(cpu, index);
    let next = if increment {
        value.wrapping_add(1)
    } else {
        value.wrapping_sub(1)
    };
    write_pair(cpu, index, next);
}

fn inc_reg(cpu: &mut Cpu, memory: &mut Memory, index: u8) {
    let value = read_reg(cpu, memory, index);
    let (result, flags) = inc_flags(value, cpu.f);
    write_reg(cpu, memory, index, result);
    cpu.f = flags;
}

fn dec_reg(cpu: &mut Cpu, memory: &mut Memory, index: u8) {
    let value = read_reg(cpu, memory, index);
    let (result, flags) = dec_flags(value, cpu.f);
    write_reg(cpu, memory, index, result);
    cpu.f = flags;
}

fn load_immediate(cpu: &mut Cpu, memory: &mut Memory, index: u8) {
    let value = memory.read(cpu.pc);
    cpu.pc = cpu.pc.wrapping_add(1);
    write_reg(cpu, memory, index, value);
}

fn copy_reg(cpu: &mut Cpu, memory: &mut Memory, opcode: u8) {
    let value = read_reg(cpu, memory, opcode & 7);
    write_reg(cpu, memory, (opcode >> 3) & 7, value);
}

fn alu_from_reg(cpu: &mut Cpu, memory: &Memory, opcode: u8) {
    let value = read_reg(cpu, memory, opcode & 7);
    apply_alu(cpu, (opcode >> 3) & 7, value);
}

fn alu_immediate(cpu: &mut Cpu, memory: &Memory, op: u8) {
    let value = memory.read(cpu.pc);
    cpu.pc = cpu.pc.wrapping_add(1);
    apply_alu(cpu, op, value);
}

fn jump_relative(cpu: &mut Cpu, memory: &Memory, take: bool) -> bool {
    let offset = memory.read(cpu.pc) as i8;
    cpu.pc = cpu.pc.wrapping_add(1);
    if take {
        cpu.pc = cpu.pc.wrapping_add(offset as u16);
    }
    true
}

fn jump_absolute(cpu: &mut Cpu, memory: &Memory, take: bool) -> bool {
    let target = peek_word(memory, cpu.pc);
    cpu.pc = cpu.pc.wrapping_add(2);
    if take {
        cpu.pc = target;
    }
    true
}

fn peek_word(memory: &Memory, address: u16) -> u16 {
    let low = memory.read(address);
    let high = memory.read(address.wrapping_add(1));
    pair(high, low)
}

fn pair(high: u8, low: u8) -> u16 {
    (u16::from(high) << 8) | u16::from(low)
}

fn read_pair(cpu: &Cpu, index: u8) -> u16 {
    match index & 3 {
        0 => pair(cpu.b, cpu.c),
        1 => pair(cpu.d, cpu.e),
        2 => pair(cpu.h, cpu.l),
        _ => cpu.sp,
    }
}

fn write_pair(cpu: &mut Cpu, index: u8, value: u16) {
    let high = (value >> 8) as u8;
    let low = value as u8;
    match index & 3 {
        0 => set_bc(cpu, high, low),
        1 => set_de(cpu, high, low),
        2 => set_hl(cpu, high, low),
        _ => cpu.sp = value,
    }
}

fn set_bc(cpu: &mut Cpu, high: u8, low: u8) {
    cpu.b = high;
    cpu.c = low;
}

fn set_de(cpu: &mut Cpu, high: u8, low: u8) {
    cpu.d = high;
    cpu.e = low;
}

fn set_hl(cpu: &mut Cpu, high: u8, low: u8) {
    cpu.h = high;
    cpu.l = low;
}

fn read_reg(cpu: &Cpu, memory: &Memory, index: u8) -> u8 {
    if index == 6 {
        return memory.read(pair(cpu.h, cpu.l));
    }
    named_reg(cpu, index)
}

fn named_reg(cpu: &Cpu, index: u8) -> u8 {
    match index & 7 {
        0 => cpu.b,
        1 => cpu.c,
        2 => cpu.d,
        3 => cpu.e,
        4 => cpu.h,
        5 => cpu.l,
        _ => cpu.a,
    }
}

fn write_reg(cpu: &mut Cpu, memory: &mut Memory, index: u8, value: u8) {
    if index == 6 {
        memory.write(pair(cpu.h, cpu.l), value);
        return;
    }
    write_named(cpu, index, value);
}

fn write_named(cpu: &mut Cpu, index: u8, value: u8) {
    match index & 7 {
        0 => cpu.b = value,
        1 => cpu.c = value,
        2 => cpu.d = value,
        3 => cpu.e = value,
        4 => cpu.h = value,
        5 => cpu.l = value,
        _ => cpu.a = value,
    }
}

fn apply_alu(cpu: &mut Cpu, op: u8, value: u8) {
    let (result, flags) = alu_result(cpu.a, value, cpu.f & CARRY, op);
    if op != 7 {
        cpu.a = result;
    }
    cpu.f = flags;
}

fn alu_result(left: u8, right: u8, carry: u8, op: u8) -> (u8, u8) {
    match op & 7 {
        0 => add(left, right, 0),
        1 => add(left, right, carry),
        2 => sub(left, right, 0),
        3 => sub(left, right, carry),
        4 => logic(left & right, true),
        5 => logic(left ^ right, false),
        6 => logic(left | right, false),
        _ => sub(left, right, 0),
    }
}

fn add(left: u8, right: u8, carry: u8) -> (u8, u8) {
    let sum = u16::from(left) + u16::from(right) + u16::from(carry);
    (sum as u8, add_flags(left, right, sum as u8, carry, sum))
}

fn add_flags(left: u8, right: u8, result: u8, carry: u8, sum: u16) -> u8 {
    let half = (left & 0x0F) + (right & 0x0F) + carry;
    let mut flags = sign_zero(result);
    flags = with_bit(flags, HALF, half > 0x0F);
    flags = with_bit(flags, CARRY, sum > 0xFF);
    with_bit(
        flags,
        PARITY,
        same_sign(left, right) && sign_differs(left, result),
    )
}

fn sub(left: u8, right: u8, carry: u8) -> (u8, u8) {
    let diff = i16::from(left) - i16::from(right) - i16::from(carry);
    (diff as u8, sub_flags(left, right, diff as u8, carry, diff))
}

fn sub_flags(left: u8, right: u8, result: u8, carry: u8, diff: i16) -> u8 {
    let half = i16::from(left & 0x0F) - i16::from(right & 0x0F) - i16::from(carry);
    let mut flags = sign_zero(result) | NEGATIVE;
    flags = with_bit(flags, HALF, half < 0);
    flags = with_bit(flags, CARRY, diff < 0);
    with_bit(
        flags,
        PARITY,
        !same_sign(left, right) && sign_differs(left, result),
    )
}

fn logic(result: u8, half: bool) -> (u8, u8) {
    let mut flags = sign_zero(result) | parity(result);
    if half {
        flags |= HALF;
    }
    (result, flags)
}

fn inc_flags(value: u8, previous: u8) -> (u8, u8) {
    let result = value.wrapping_add(1);
    let mut flags = sign_zero(result);
    flags = with_bit(flags, HALF, (value & 0x0F) == 0x0F);
    flags = with_bit(flags, PARITY, value == 0x7F);
    (result, flags | (previous & CARRY))
}

fn dec_flags(value: u8, previous: u8) -> (u8, u8) {
    let result = value.wrapping_sub(1);
    let mut flags = sign_zero(result) | NEGATIVE;
    flags = with_bit(flags, HALF, (value & 0x0F) == 0);
    flags = with_bit(flags, PARITY, value == 0x80);
    (result, flags | (previous & CARRY))
}

fn sign_zero(value: u8) -> u8 {
    let mut flags = value & SIGN;
    if value == 0 {
        flags |= ZERO;
    }
    flags
}

fn parity(value: u8) -> u8 {
    if value.count_ones() % 2 == 0 {
        PARITY
    } else {
        0
    }
}

fn with_bit(flags: u8, bit: u8, enabled: bool) -> u8 {
    if enabled { flags | bit } else { flags }
}

fn same_sign(left: u8, right: u8) -> bool {
    (left ^ right) & SIGN == 0
}

fn sign_differs(left: u8, result: u8) -> bool {
    (left ^ result) & SIGN != 0
}

fn flag_met(flags: u8, code: u8) -> bool {
    match code & 7 {
        0 => (flags & ZERO) == 0,
        1 => (flags & ZERO) != 0,
        2 => (flags & CARRY) == 0,
        3 => (flags & CARRY) != 0,
        4 => (flags & PARITY) == 0,
        5 => (flags & PARITY) != 0,
        6 => (flags & SIGN) == 0,
        _ => (flags & SIGN) != 0,
    }
}
