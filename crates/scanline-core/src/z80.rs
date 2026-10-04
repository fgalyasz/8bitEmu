use crate::error::CoreError;
use crate::memory::Memory;

mod ay;
mod bits;
mod extended;
mod flow;
mod index;
mod io;

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
    pub a2: u8,
    pub f2: u8,
    pub b2: u8,
    pub c2: u8,
    pub d2: u8,
    pub e2: u8,
    pub h2: u8,
    pub l2: u8,
    pub ix: u16,
    pub iy: u16,
    pub i: u8,
    pub r: u8,
    pub im: u8,
    pub iff1: bool,
    pub iff2: bool,
    pub halted: bool,
    pub arm_ei: bool,
    pub branched: bool,
    pub index_mode: u8,
    pub index_addr: u16,
}

#[derive(Default)]
pub struct Ports {
    pub border: u8,
    pub speaker: u8,
    pub pressed: [u8; 8],
    pub model_128: bool,
    pub locked: bool,
    pub tape_on: bool,
    pub ear_high: bool,
    pub mic: bool,
    pub kempston: u8,
    pub fe_reads: u32,
    pub(crate) ay: ay::Ay,
}

pub fn step(cpu: &mut Cpu, memory: &mut Memory, ports: &mut Ports) -> Result<u32, CoreError> {
    cpu.branched = true;
    cpu.index_mode = 0;
    let opcode = fetch_m1(cpu, memory);
    dispatch(cpu, memory, ports, opcode)
}

pub fn accept_interrupt(cpu: &mut Cpu, memory: &mut Memory) -> u32 {
    if cpu.halted {
        cpu.halted = false;
        cpu.pc = cpu.pc.wrapping_add(1);
    }
    cpu.iff1 = false;
    cpu.iff2 = false;
    push(cpu, memory, cpu.pc);
    cpu.pc = interrupt_target(cpu, memory);
    13
}

fn execute(
    cpu: &mut Cpu,
    memory: &mut Memory,
    ports: &mut Ports,
    opcode: u8,
) -> Result<(), CoreError> {
    if run_fixed(cpu, memory, ports, opcode)
        || run_pattern(cpu, memory, opcode)
        || flow::run(cpu, memory, opcode)
    {
        return Ok(());
    }
    Err(CoreError::Opcode { opcode })
}

fn dispatch(
    cpu: &mut Cpu,
    memory: &mut Memory,
    ports: &mut Ports,
    opcode: u8,
) -> Result<u32, CoreError> {
    if opcode == 0xCB {
        return Ok(bits::run(cpu, memory));
    }
    if opcode == 0xED {
        return Ok(extended::run(cpu, memory, ports));
    }
    if opcode == 0xDD {
        return index::run(cpu, memory, ports, true);
    }
    if opcode == 0xFD {
        return index::run(cpu, memory, ports, false);
    }
    execute(cpu, memory, ports, opcode)?;
    Ok(unprefixed_cycles(opcode, cpu.branched))
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
        0xDB => in_port(cpu, memory, ports),
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
    if cpu.index_mode == 1 || cpu.index_mode == 2 {
        let de = pair(cpu.d, cpu.e);
        let indexed = read_pair(cpu, 2);
        write_pair(cpu, 2, de);
        cpu.d = (indexed >> 8) as u8;
        cpu.e = indexed as u8;
        return;
    }
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
    cpu.sp = read_pair(cpu, 2);
    true
}

fn out_port(cpu: &mut Cpu, memory: &mut Memory, ports: &mut Ports) -> bool {
    let low = fetch_byte(cpu, memory);
    let full = (u16::from(cpu.a) << 8) | u16::from(low);
    ports.output(memory, full, cpu.a);
    true
}

fn in_port(cpu: &mut Cpu, memory: &mut Memory, ports: &mut Ports) -> bool {
    let low = fetch_byte(cpu, memory);
    let full = (u16::from(cpu.a) << 8) | u16::from(low);
    cpu.a = ports.input(full);
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
    cpu.branched = take;
    if take {
        cpu.pc = cpu.pc.wrapping_add(offset as u16);
    }
    true
}

fn jump_absolute(cpu: &mut Cpu, memory: &Memory, take: bool) -> bool {
    let target = peek_word(memory, cpu.pc);
    cpu.pc = cpu.pc.wrapping_add(2);
    cpu.branched = take;
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
        2 => indexed_hl(cpu),
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
    let value = pair(high, low);
    match cpu.index_mode {
        1 | 3 => cpu.ix = value,
        2 | 4 => cpu.iy = value,
        _ => {
            cpu.h = high;
            cpu.l = low;
        }
    }
}

fn read_reg(cpu: &Cpu, memory: &Memory, index: u8) -> u8 {
    if index == 6 {
        return memory.read(memory_address(cpu));
    }
    if half_index(cpu, index) {
        return index_half(cpu, index);
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
        memory.write(memory_address(cpu), value);
        return;
    }
    if half_index(cpu, index) {
        write_half(cpu, index, value);
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

fn fetch_m1(cpu: &mut Cpu, memory: &Memory) -> u8 {
    let opcode = fetch_byte(cpu, memory);
    bump_r(cpu);
    opcode
}

fn fetch_byte(cpu: &mut Cpu, memory: &Memory) -> u8 {
    let value = memory.read(cpu.pc);
    cpu.pc = cpu.pc.wrapping_add(1);
    value
}

fn bump_r(cpu: &mut Cpu) {
    let next = cpu.r.wrapping_add(1) & 0x7F;
    cpu.r = (cpu.r & 0x80) | next;
}

fn interrupt_target(cpu: &Cpu, memory: &Memory) -> u16 {
    if cpu.im == 2 {
        let vector = (u16::from(cpu.i) << 8) | 0xFF;
        return peek_word(memory, vector);
    }
    0x0038
}

fn push(cpu: &mut Cpu, memory: &mut Memory, value: u16) {
    cpu.sp = cpu.sp.wrapping_sub(2);
    write_word(memory, cpu.sp, value);
}

fn pop(cpu: &mut Cpu, memory: &mut Memory) -> u16 {
    let value = peek_word(memory, cpu.sp);
    cpu.sp = cpu.sp.wrapping_add(2);
    value
}

fn write_word(memory: &mut Memory, address: u16, value: u16) {
    memory.write(address, value as u8);
    memory.write(address.wrapping_add(1), (value >> 8) as u8);
}

fn indexed_hl(cpu: &Cpu) -> u16 {
    match cpu.index_mode {
        1 | 3 => cpu.ix,
        2 | 4 => cpu.iy,
        _ => pair(cpu.h, cpu.l),
    }
}

fn memory_address(cpu: &Cpu) -> u16 {
    if cpu.index_mode >= 3 {
        return cpu.index_addr;
    }
    pair(cpu.h, cpu.l)
}

fn half_index(cpu: &Cpu, index: u8) -> bool {
    (cpu.index_mode == 1 || cpu.index_mode == 2) && (index == 4 || index == 5)
}

fn index_half(cpu: &Cpu, index: u8) -> u8 {
    let value = if cpu.index_mode == 1 { cpu.ix } else { cpu.iy };
    if index == 4 { (value >> 8) as u8 } else { value as u8 }
}

fn write_half(cpu: &mut Cpu, index: u8, value: u8) {
    let current = if cpu.index_mode == 1 { cpu.ix } else { cpu.iy };
    let next = if index == 4 {
        (current & 0x00FF) | (u16::from(value) << 8)
    } else {
        (current & 0xFF00) | u16::from(value)
    };
    if cpu.index_mode == 1 {
        cpu.ix = next;
        return;
    }
    cpu.iy = next;
}

fn unprefixed_cycles(opcode: u8, branched: bool) -> u32 {
    if opcode == 0x10 {
        return if branched { 13 } else { 8 };
    }
    if opcode == 0x18 {
        return 12;
    }
    if opcode & 0xE7 == 0x20 {
        return if branched { 12 } else { 7 };
    }
    if opcode & 0xC7 == 0xC0 {
        return if branched { 11 } else { 5 };
    }
    if opcode & 0xC7 == 0xC4 {
        return if branched { 17 } else { 10 };
    }
    class_cycles(opcode)
}

fn class_cycles(opcode: u8) -> u32 {
    if opcode & 0xCF == 0x01 {
        return 10;
    }
    if opcode & 0xCF == 0x03 || opcode & 0xCF == 0x0B || opcode & 0xCF == 0x09 {
        return if opcode & 0xCF == 0x09 { 11 } else { 6 };
    }
    if opcode & 0xC7 == 0x04 || opcode & 0xC7 == 0x05 {
        return if (opcode >> 3) & 7 == 6 { 11 } else { 4 };
    }
    if opcode & 0xC7 == 0x06 {
        return if (opcode >> 3) & 7 == 6 { 10 } else { 7 };
    }
    if let Some(cycles) = block_cycles(opcode) {
        return cycles;
    }
    named_cycles(opcode)
}

fn block_cycles(opcode: u8) -> Option<u32> {
    if (0x40..0x80).contains(&opcode) {
        if opcode == 0x76 {
            return Some(4);
        }
        let memory = opcode & 7 == 6 || (opcode >> 3) & 7 == 6;
        return Some(if memory { 7 } else { 4 });
    }
    if (0x80..0xC0).contains(&opcode) {
        return Some(if opcode & 7 == 6 { 7 } else { 4 });
    }
    if opcode & 0xC7 == 0xC6 || opcode & 0xCF == 0xC5 {
        return Some(if opcode & 0xC7 == 0xC6 { 7 } else { 11 });
    }
    if opcode & 0xCF == 0xC1 || opcode & 0xC7 == 0xC7 || opcode & 0xC7 == 0xC2 || opcode == 0xC3 {
        return Some(if opcode & 0xC7 == 0xC7 { 11 } else { 10 });
    }
    None
}

fn named_cycles(opcode: u8) -> u32 {
    match opcode {
        0x02 | 0x12 | 0x0A | 0x1A => 7,
        0x22 | 0x2A => 16,
        0x32 | 0x3A => 13,
        0xCD => 17,
        0xC9 => 10,
        0xD3 | 0xDB => 11,
        0xE3 => 19,
        0xF9 => 6,
        _ => 4,
    }
}
