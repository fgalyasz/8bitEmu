use crate::memory::Memory;
use crate::z80::Cpu;

pub fn run(cpu: &mut Cpu, memory: &mut Memory, opcode: u8) -> bool {
    if shift_or_exchange(cpu, memory, opcode) {
        return true;
    }
    stack_or_math(cpu, memory, opcode)
}

fn shift_or_exchange(cpu: &mut Cpu, memory: &mut Memory, opcode: u8) -> bool {
    match opcode {
        0x07 => rlca(cpu),
        0x0F => rrca(cpu),
        0x17 => rla(cpu),
        0x1F => rra(cpu),
        0x08 => exchange_af(cpu),
        0x22 => store_hl(cpu, memory),
        0x27 => daa(cpu),
        0x2A => load_hl(cpu, memory),
        0x2F => cpl(cpu),
        0x37 => scf(cpu),
        0x3F => ccf(cpu),
        0xD9 => exchange_sets(cpu),
        0xE3 => exchange_sp(cpu, memory),
        0xE9 => jump_hl(cpu),
        0xF3 => disable(cpu),
        0xFB => enable(cpu),
        _ => return false,
    }
    true
}

fn stack_or_math(cpu: &mut Cpu, memory: &mut Memory, opcode: u8) -> bool {
    if opcode & 0xCF == 0x09 {
        add_hl(cpu, (opcode >> 4) & 3);
        return true;
    }
    if opcode & 0xC7 == 0xC0 {
        ret_when(cpu, memory, (opcode >> 3) & 7);
        return true;
    }
    if opcode & 0xCF == 0xC1 {
        pop_pair(cpu, memory, (opcode >> 4) & 3);
        return true;
    }
    if opcode & 0xC7 == 0xC4 {
        call_when(cpu, memory, (opcode >> 3) & 7);
        return true;
    }
    if opcode & 0xCF == 0xC5 {
        push_pair(cpu, memory, (opcode >> 4) & 3);
        return true;
    }
    if opcode & 0xC7 == 0xC7 {
        rst(cpu, memory, opcode);
        return true;
    }
    if opcode == 0xC9 {
        cpu.pc = super::pop(cpu, memory);
        return true;
    }
    if opcode == 0xCD {
        call(cpu, memory, true);
        return true;
    }
    false
}

fn rlca(cpu: &mut Cpu) {
    let carry = cpu.a >> 7;
    cpu.a = (cpu.a << 1) | carry;
    cpu.f = (cpu.f & 0xC4) | carry;
}

fn rrca(cpu: &mut Cpu) {
    let carry = cpu.a & 1;
    cpu.a = (cpu.a >> 1) | (carry << 7);
    cpu.f = (cpu.f & 0xC4) | carry;
}

fn rla(cpu: &mut Cpu) {
    let carry_out = cpu.a >> 7;
    cpu.a = (cpu.a << 1) | (cpu.f & 1);
    cpu.f = (cpu.f & 0xC4) | carry_out;
}

fn rra(cpu: &mut Cpu) {
    let carry_out = cpu.a & 1;
    cpu.a = (cpu.a >> 1) | ((cpu.f & 1) << 7);
    cpu.f = (cpu.f & 0xC4) | carry_out;
}

fn exchange_af(cpu: &mut Cpu) {
    let a = cpu.a;
    let f = cpu.f;
    cpu.a = cpu.a2;
    cpu.f = cpu.f2;
    cpu.a2 = a;
    cpu.f2 = f;
}

fn exchange_sets(cpu: &mut Cpu) {
    swap(&mut cpu.b, &mut cpu.b2);
    swap(&mut cpu.c, &mut cpu.c2);
    swap(&mut cpu.d, &mut cpu.d2);
    swap(&mut cpu.e, &mut cpu.e2);
    swap(&mut cpu.h, &mut cpu.h2);
    swap(&mut cpu.l, &mut cpu.l2);
}

fn swap(left: &mut u8, right: &mut u8) {
    let saved = *left;
    *left = *right;
    *right = saved;
}

fn store_hl(cpu: &mut Cpu, memory: &mut Memory) {
    let address = super::peek_word(memory, cpu.pc);
    cpu.pc = cpu.pc.wrapping_add(2);
    super::write_word(memory, address, super::read_pair(cpu, 2));
}

fn load_hl(cpu: &mut Cpu, memory: &mut Memory) {
    let address = super::peek_word(memory, cpu.pc);
    cpu.pc = cpu.pc.wrapping_add(2);
    let value = super::peek_word(memory, address);
    super::write_pair(cpu, 2, value);
}

fn daa(cpu: &mut Cpu) {
    let correction = daa_correction(cpu.a, cpu.f);
    let result = daa_apply(cpu.a, cpu.f, correction);
    cpu.f = daa_flags(cpu.a, result, cpu.f, correction);
    cpu.a = result;
}

fn daa_correction(value: u8, flags: u8) -> u8 {
    let mut correction = 0u8;
    let subtract = flag(flags, super::NEGATIVE);
    if flag(flags, super::HALF) || (!subtract && (value & 0x0F) > 9) {
        correction |= 0x06;
    }
    if flag(flags, super::CARRY) || (!subtract && value > 0x99) {
        correction |= 0x60;
    }
    correction
}

fn daa_apply(value: u8, flags: u8, correction: u8) -> u8 {
    if flag(flags, super::NEGATIVE) {
        return value.wrapping_sub(correction);
    }
    value.wrapping_add(correction)
}

fn daa_flags(previous: u8, result: u8, flags: u8, correction: u8) -> u8 {
    let mut next = super::sign_zero(result) | super::parity(result);
    if flag(flags, super::NEGATIVE) {
        next |= super::NEGATIVE;
    }
    if flag(flags, super::CARRY) || (!flag(flags, super::NEGATIVE) && previous > 0x99) {
        next |= super::CARRY;
    }
    if correction & 0x06 != 0 && !flag(flags, super::NEGATIVE) {
        next |= super::HALF;
    }
    if flag(flags, super::NEGATIVE) && flag(flags, super::HALF) {
        next |= super::HALF;
    }
    next
}

fn flag(flags: u8, bit: u8) -> bool {
    (flags & bit) != 0
}

fn cpl(cpu: &mut Cpu) {
    cpu.a = !cpu.a;
    cpu.f = (cpu.f & 0xC5) | super::HALF | super::NEGATIVE;
}

fn scf(cpu: &mut Cpu) {
    cpu.f = (cpu.f & 0xC4) | super::CARRY;
}

fn ccf(cpu: &mut Cpu) {
    let carry = cpu.f & super::CARRY;
    let half = if carry == 0 { 0 } else { super::HALF };
    cpu.f = (cpu.f & 0xC4) | half | (carry ^ super::CARRY);
}

fn add_hl(cpu: &mut Cpu, which: u8) {
    let left = u32::from(super::read_pair(cpu, 2));
    let right = u32::from(super::read_pair(cpu, which));
    let sum = left + right;
    super::write_pair(cpu, 2, sum as u16);
    let half = (left & 0x0FFF) + (right & 0x0FFF) > 0x0FFF;
    let mut flags = cpu.f & 0xC4;
    if half {
        flags |= super::HALF;
    }
    if sum > 0xFFFF {
        flags |= super::CARRY;
    }
    cpu.f = flags;
}

fn ret_when(cpu: &mut Cpu, memory: &mut Memory, code: u8) {
    let take = super::flag_met(cpu.f, code);
    cpu.branched = take;
    if take {
        cpu.pc = super::pop(cpu, memory);
    }
}

fn pop_pair(cpu: &mut Cpu, memory: &mut Memory, which: u8) {
    let value = super::pop(cpu, memory);
    write_stack(cpu, which, value);
}

fn push_pair(cpu: &mut Cpu, memory: &mut Memory, which: u8) {
    super::push(cpu, memory, read_stack(cpu, which));
}

fn call_when(cpu: &mut Cpu, memory: &mut Memory, code: u8) {
    call(cpu, memory, super::flag_met(cpu.f, code));
}

fn call(cpu: &mut Cpu, memory: &mut Memory, take: bool) {
    let target = super::peek_word(memory, cpu.pc);
    cpu.pc = cpu.pc.wrapping_add(2);
    cpu.branched = take;
    if take {
        super::push(cpu, memory, cpu.pc);
        cpu.pc = target;
    }
}

fn rst(cpu: &mut Cpu, memory: &mut Memory, opcode: u8) {
    super::push(cpu, memory, cpu.pc);
    cpu.pc = u16::from(opcode & 0x38);
}

fn exchange_sp(cpu: &mut Cpu, memory: &mut Memory) {
    let stacked = super::peek_word(memory, cpu.sp);
    super::write_word(memory, cpu.sp, super::read_pair(cpu, 2));
    super::write_pair(cpu, 2, stacked);
}

fn jump_hl(cpu: &mut Cpu) {
    cpu.pc = super::read_pair(cpu, 2);
}

fn disable(cpu: &mut Cpu) {
    cpu.iff1 = false;
    cpu.iff2 = false;
    cpu.arm_ei = false;
}

fn enable(cpu: &mut Cpu) {
    cpu.arm_ei = true;
}

fn read_stack(cpu: &Cpu, which: u8) -> u16 {
    match which & 3 {
        0 => super::pair(cpu.b, cpu.c),
        1 => super::pair(cpu.d, cpu.e),
        2 => super::read_pair(cpu, 2),
        _ => super::pair(cpu.a, cpu.f),
    }
}

fn write_stack(cpu: &mut Cpu, which: u8, value: u16) {
    let high = (value >> 8) as u8;
    let low = value as u8;
    match which & 3 {
        0 => super::set_bc(cpu, high, low),
        1 => super::set_de(cpu, high, low),
        2 => super::write_pair(cpu, 2, value),
        _ => {
            cpu.a = high;
            cpu.f = low;
        }
    }
}
