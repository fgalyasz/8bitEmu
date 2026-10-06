use crate::error::CoreError;

const N: u8 = 0x80;
const V: u8 = 0x40;
const B: u8 = 0x10;
const D: u8 = 0x08;
const I: u8 = 0x04;
const Z: u8 = 0x02;
const C: u8 = 0x01;

pub trait Bus {
    fn read(&mut self, addr: u16) -> u8;
    fn write(&mut self, addr: u16, value: u8);
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Cpu {
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub sp: u8,
    pub pc: u16,
    pub status: u8,
    irq_pending: bool,
    nmi_pending: bool,
}

pub fn reset(cpu: &mut Cpu, bus: &mut impl Bus) {
    cpu.sp = 0xFD;
    cpu.status = I | B;
    cpu.pc = read16(bus, 0xFFFC);
    cpu.irq_pending = false;
    cpu.nmi_pending = false;
}

pub fn trigger_irq(cpu: &mut Cpu) {
    cpu.irq_pending = true;
}

pub fn trigger_nmi(cpu: &mut Cpu) {
    cpu.nmi_pending = true;
}

pub fn step(cpu: &mut Cpu, bus: &mut impl Bus) -> Result<u32, CoreError> {
    if cpu.nmi_pending {
        cpu.nmi_pending = false;
        return Ok(service_interrupt(cpu, bus, 0xFFFA, false));
    }
    if cpu.irq_pending && !flag_i(cpu.status) {
        cpu.irq_pending = false;
        return Ok(service_interrupt(cpu, bus, 0xFFFE, false));
    }
    let opcode = fetch(cpu, bus);
    execute(cpu, bus, opcode)
}

fn execute(cpu: &mut Cpu, bus: &mut impl Bus, opcode: u8) -> Result<u32, CoreError> {
    let entry = OPCODES[opcode as usize];
    if entry.kind == OpKind::Illegal {
        return Err(CoreError::Opcode { opcode });
    }
    let mut cycles = entry.base_cycles as u32;
    cycles += run(cpu, bus, entry)?;
    Ok(cycles)
}

fn run(cpu: &mut Cpu, bus: &mut impl Bus, entry: OpEntry) -> Result<u32, CoreError> {
    let mode = entry.mode;
    let op = entry.kind;
    let extra = match mode {
        Addr::Imp => run_imp(cpu, bus, op),
        Addr::Acc => run_acc(cpu, op),
        Addr::Imm => run_imm(cpu, bus, op),
        Addr::Zp => run_zp(cpu, bus, op),
        Addr::Zpx => run_zpx(cpu, bus, op),
        Addr::Zpy => run_zpy(cpu, bus, op),
        Addr::Abs => run_abs(cpu, bus, op),
        Addr::Absx => run_absx(cpu, bus, op, entry.page_penalty),
        Addr::Absy => run_absy(cpu, bus, op, entry.page_penalty),
        Addr::Ind => run_ind(cpu, bus, op),
        Addr::Indx => run_indx(cpu, bus, op),
        Addr::Indy => run_indy(cpu, bus, op, entry.page_penalty),
        Addr::Rel => run_rel(cpu, bus, op),
    }?;
    Ok(extra)
}

fn fetch(cpu: &mut Cpu, bus: &mut impl Bus) -> u8 {
    let value = bus.read(cpu.pc);
    cpu.pc = cpu.pc.wrapping_add(1);
    value
}

fn read16(bus: &mut impl Bus, addr: u16) -> u16 {
    let lo = bus.read(addr) as u16;
    let hi = bus.read(addr.wrapping_add(1)) as u16;
    lo | (hi << 8)
}

fn read16_zp(bus: &mut impl Bus, zp: u16) -> u16 {
    let base = zp & 0xFF;
    let lo = bus.read(base) as u16;
    let hi = bus.read((base + 1) & 0xFF) as u16;
    lo | (hi << 8)
}

fn push(cpu: &mut Cpu, bus: &mut impl Bus, value: u8) {
    bus.write(stack_addr(cpu.sp), value);
    cpu.sp = cpu.sp.wrapping_sub(1);
}

fn pop(cpu: &mut Cpu, bus: &mut impl Bus) -> u8 {
    cpu.sp = cpu.sp.wrapping_add(1);
    bus.read(stack_addr(cpu.sp))
}

fn stack_addr(sp: u8) -> u16 {
    0x0100 | sp as u16
}

fn flag_i(status: u8) -> bool {
    status & I != 0
}

fn set_nz(cpu: &mut Cpu, value: u8) {
    cpu.status = (cpu.status & !(N | Z)) | nz_flags(value);
}

fn nz_flags(value: u8) -> u8 {
    let mut flags = 0u8;
    if value & 0x80 != 0 {
        flags |= N;
    }
    if value == 0 {
        flags |= Z;
    }
    flags
}

fn set_carry(cpu: &mut Cpu, carry: bool) {
    cpu.status = set_bit(cpu.status, C, carry);
}

fn set_bit(status: u8, mask: u8, on: bool) -> u8 {
    if on {
        status | mask
    } else {
        status & !mask
    }
}

fn branch(cpu: &mut Cpu, bus: &mut impl Bus, take: bool) -> u32 {
    let offset = fetch(cpu, bus) as i8 as i16;
    if !take {
        return 0;
    }
    let old = cpu.pc;
    cpu.pc = cpu.pc.wrapping_add(offset as u16);
    page_cross(old, cpu.pc) as u32
}

fn page_cross(from: u16, to: u16) -> u8 {
    if from & 0xFF00 != to & 0xFF00 {
        1
    } else {
        0
    }
}

fn service_interrupt(cpu: &mut Cpu, bus: &mut impl Bus, vector: u16, break_flag: bool) -> u32 {
    push16(cpu, bus, cpu.pc);
    let pushed = push_status(cpu, break_flag);
    push(cpu, bus, pushed);
    cpu.status |= I;
    cpu.pc = read16(bus, vector);
    7
}

fn push16(cpu: &mut Cpu, bus: &mut impl Bus, value: u16) {
    push(cpu, bus, (value >> 8) as u8);
    push(cpu, bus, value as u8);
}

fn push_status(cpu: &mut Cpu, break_flag: bool) -> u8 {
    let b = if break_flag { B } else { 0 };
    (cpu.status & !B) | b | U
}

fn adc(cpu: &mut Cpu, value: u8) {
    if cpu.status & D != 0 {
        adc_decimal(cpu, value);
        return;
    }
    adc_binary(cpu, value);
}

fn adc_binary(cpu: &mut Cpu, value: u8) {
    let carry = if cpu.status & C != 0 { 1u16 } else { 0 };
    let sum = cpu.a as u16 + value as u16 + carry;
    let result = sum as u8;
    let overflow = (cpu.a ^ result) & (value ^ result) & 0x80 != 0;
    cpu.status = set_bit(cpu.status, V, overflow);
    set_carry(cpu, sum > 0xFF);
    cpu.a = result;
    set_nz(cpu, cpu.a);
}

fn adc_decimal(cpu: &mut Cpu, value: u8) {
    let carry = if cpu.status & C != 0 { 1u16 } else { 0 };
    let mut low = (cpu.a & 0x0F) as u16 + (value & 0x0F) as u16 + carry;
    if low > 9 {
        low += 6;
    }
    let mut high = (cpu.a >> 4) as u16 + (value >> 4) as u16 + (low >> 4);
    let carry_out = high > 9;
    if high > 9 {
        high += 6;
    }
    cpu.a = ((high << 4) | (low & 0x0F)) as u8;
    set_carry(cpu, carry_out);
    set_nz(cpu, cpu.a);
    cpu.status &= !V;
}

fn sbc(cpu: &mut Cpu, value: u8) {
    if cpu.status & D != 0 {
        sbc_decimal(cpu, value);
        return;
    }
    sbc_binary(cpu, value);
}

fn sbc_binary(cpu: &mut Cpu, value: u8) {
    let carry = cpu.status & C != 0;
    cpu.status = set_bit(cpu.status, C, !carry);
    adc_binary(cpu, value ^ 0xFF);
}

fn sbc_decimal(cpu: &mut Cpu, value: u8) {
    let carry = cpu.status & C != 0;
    cpu.status = set_bit(cpu.status, C, !carry);
    adc_decimal(cpu, value ^ 0xFF);
}

fn compare(cpu: &mut Cpu, reg: u8, value: u8) {
    let result = reg.wrapping_sub(value);
    set_nz(cpu, result);
    set_carry(cpu, reg >= value);
}

fn bit(cpu: &mut Cpu, value: u8) {
    set_nz(cpu, value & cpu.a);
    cpu.status = set_bit(cpu.status, V, value & 0x40 != 0);
}

fn asl(cpu: &mut Cpu, value: u8) -> u8 {
    set_carry(cpu, value & 0x80 != 0);
    let result = value << 1;
    set_nz(cpu, result);
    result
}

fn lsr(cpu: &mut Cpu, value: u8) -> u8 {
    set_carry(cpu, value & 0x01 != 0);
    let result = value >> 1;
    set_nz(cpu, result);
    result
}

fn rol(cpu: &mut Cpu, value: u8) -> u8 {
    let carry_in = if cpu.status & C != 0 { 1u8 } else { 0u8 };
    set_carry(cpu, value & 0x80 != 0);
    let result = (value << 1) | carry_in;
    set_nz(cpu, result);
    result
}

fn ror(cpu: &mut Cpu, value: u8) -> u8 {
    let carry_in = if cpu.status & C != 0 { 0x80u8 } else { 0u8 };
    set_carry(cpu, value & 0x01 != 0);
    let result = (value >> 1) | carry_in;
    set_nz(cpu, result);
    result
}

fn inc(cpu: &mut Cpu, value: u8) -> u8 {
    let result = value.wrapping_add(1);
    set_nz(cpu, result);
    result
}

fn dec(cpu: &mut Cpu, value: u8) -> u8 {
    let result = value.wrapping_sub(1);
    set_nz(cpu, result);
    result
}

fn run_imp(cpu: &mut Cpu, bus: &mut impl Bus, op: OpKind) -> Result<u32, CoreError> {
    match op {
        OpKind::Brk => {
            cpu.pc = cpu.pc.wrapping_add(1);
            push16(cpu, bus, cpu.pc);
            let status = push_status(cpu, true);
            push(cpu, bus, status);
            cpu.status |= I;
            cpu.pc = read16(bus, 0xFFFE);
            Ok(0)
        }
        OpKind::Php => {
            push(cpu, bus, cpu.status | B);
            Ok(0)
        }
        OpKind::Pla => {
            cpu.a = pop(cpu, bus);
            set_nz(cpu, cpu.a);
            Ok(0)
        }
        OpKind::Plp => {
            cpu.status = (pop(cpu, bus) & !(B | U)) | B;
            Ok(0)
        }
        OpKind::Pha => {
            push(cpu, bus, cpu.a);
            Ok(0)
        }
        OpKind::Clc => {
            cpu.status &= !C;
            Ok(0)
        }
        OpKind::Sec => {
            cpu.status |= C;
            Ok(0)
        }
        OpKind::Cld => {
            cpu.status &= !D;
            Ok(0)
        }
        OpKind::Sed => {
            cpu.status |= D;
            Ok(0)
        }
        OpKind::Cli => {
            cpu.status &= !I;
            Ok(0)
        }
        OpKind::Sei => {
            cpu.status |= I;
            Ok(0)
        }
        OpKind::Clv => {
            cpu.status &= !V;
            Ok(0)
        }
        OpKind::Dex => {
            cpu.x = cpu.x.wrapping_sub(1);
            set_nz(cpu, cpu.x);
            Ok(0)
        }
        OpKind::Dey => {
            cpu.y = cpu.y.wrapping_sub(1);
            set_nz(cpu, cpu.y);
            Ok(0)
        }
        OpKind::Inx => {
            cpu.x = cpu.x.wrapping_add(1);
            set_nz(cpu, cpu.x);
            Ok(0)
        }
        OpKind::Iny => {
            cpu.y = cpu.y.wrapping_add(1);
            set_nz(cpu, cpu.y);
            Ok(0)
        }
        OpKind::Tax => {
            cpu.x = cpu.a;
            set_nz(cpu, cpu.x);
            Ok(0)
        }
        OpKind::Tay => {
            cpu.y = cpu.a;
            set_nz(cpu, cpu.y);
            Ok(0)
        }
        OpKind::Tya => {
            cpu.a = cpu.y;
            set_nz(cpu, cpu.a);
            Ok(0)
        }
        OpKind::Txa => {
            cpu.a = cpu.x;
            set_nz(cpu, cpu.a);
            Ok(0)
        }
        OpKind::Tsx => {
            cpu.x = cpu.sp;
            set_nz(cpu, cpu.x);
            Ok(0)
        }
        OpKind::Txs => {
            cpu.sp = cpu.x;
            Ok(0)
        }
        OpKind::Rti => {
            cpu.status = (pop(cpu, bus) & !(B | U)) | B;
            cpu.pc = pop16(cpu, bus);
            Ok(0)
        }
        OpKind::Rts => {
            cpu.pc = pop16(cpu, bus).wrapping_add(1);
            Ok(0)
        }
        OpKind::Nop => Ok(0),
        _ => Err(CoreError::Opcode { opcode: 0 }),
    }
}

const U: u8 = 0x20;

fn pop16(cpu: &mut Cpu, bus: &mut impl Bus) -> u16 {
    let lo = pop(cpu, bus) as u16;
    let hi = pop(cpu, bus) as u16;
    lo | (hi << 8)
}

fn run_acc(cpu: &mut Cpu, op: OpKind) -> Result<u32, CoreError> {
    match op {
        OpKind::Asl => {
            cpu.a = asl(cpu, cpu.a);
            Ok(0)
        }
        OpKind::Lsr => {
            cpu.a = lsr(cpu, cpu.a);
            Ok(0)
        }
        OpKind::Rol => {
            cpu.a = rol(cpu, cpu.a);
            Ok(0)
        }
        OpKind::Ror => {
            cpu.a = ror(cpu, cpu.a);
            Ok(0)
        }
        _ => Err(CoreError::Opcode { opcode: 0 }),
    }
}

fn run_imm(cpu: &mut Cpu, bus: &mut impl Bus, op: OpKind) -> Result<u32, CoreError> {
    let value = fetch(cpu, bus);
    apply_load_alu(cpu, op, value)?;
    Ok(0)
}

fn run_zp(cpu: &mut Cpu, bus: &mut impl Bus, op: OpKind) -> Result<u32, CoreError> {
    let addr = fetch(cpu, bus) as u16;
    run_mem(cpu, bus, op, addr)
}

fn run_zpx(cpu: &mut Cpu, bus: &mut impl Bus, op: OpKind) -> Result<u32, CoreError> {
    let base = fetch(cpu, bus) as u16;
    let addr = base.wrapping_add(cpu.x as u16) & 0xFF;
    run_mem(cpu, bus, op, addr)
}

fn run_zpy(cpu: &mut Cpu, bus: &mut impl Bus, op: OpKind) -> Result<u32, CoreError> {
    let base = fetch(cpu, bus) as u16;
    let addr = base.wrapping_add(cpu.y as u16) & 0xFF;
    run_mem(cpu, bus, op, addr)
}

fn run_abs(cpu: &mut Cpu, bus: &mut impl Bus, op: OpKind) -> Result<u32, CoreError> {
    let addr = fetch16(cpu, bus);
    if op == OpKind::Jsr {
        return jsr(cpu, bus, addr);
    }
    if op == OpKind::Jmp {
        cpu.pc = addr;
        return Ok(0);
    }
    run_mem(cpu, bus, op, addr)
}

fn jsr(cpu: &mut Cpu, bus: &mut impl Bus, target: u16) -> Result<u32, CoreError> {
    push16(cpu, bus, cpu.pc.wrapping_sub(1));
    cpu.pc = target;
    Ok(0)
}

fn run_absx(
    cpu: &mut Cpu,
    bus: &mut impl Bus,
    op: OpKind,
    penalty: bool,
) -> Result<u32, CoreError> {
    let base = fetch16(cpu, bus);
    let addr = base.wrapping_add(cpu.x as u16);
    let extra = mem_penalty(penalty, op, base, addr);
    run_mem(cpu, bus, op, addr).map(|_| extra as u32)
}

fn run_absy(
    cpu: &mut Cpu,
    bus: &mut impl Bus,
    op: OpKind,
    penalty: bool,
) -> Result<u32, CoreError> {
    let base = fetch16(cpu, bus);
    let addr = base.wrapping_add(cpu.y as u16);
    let extra = mem_penalty(penalty, op, base, addr);
    run_mem(cpu, bus, op, addr).map(|_| extra as u32)
}

fn mem_penalty(penalty: bool, op: OpKind, base: u16, addr: u16) -> u8 {
    if !penalty || op.is_store() {
        return 0;
    }
    page_cross(base, addr)
}

fn fetch16(cpu: &mut Cpu, bus: &mut impl Bus) -> u16 {
    let lo = fetch(cpu, bus) as u16;
    let hi = fetch(cpu, bus) as u16;
    lo | (hi << 8)
}

fn run_ind(cpu: &mut Cpu, bus: &mut impl Bus, op: OpKind) -> Result<u32, CoreError> {
    let ptr = fetch16(cpu, bus);
    let addr = jmp_indirect(bus, ptr);
    match op {
        OpKind::Jmp => {
            cpu.pc = addr;
            Ok(0)
        }
        _ => Err(CoreError::Opcode { opcode: 0 }),
    }
}

fn jmp_indirect(bus: &mut impl Bus, ptr: u16) -> u16 {
    let lo = bus.read(ptr);
    let hi_ptr = if ptr & 0xFF == 0xFF {
        ptr & 0xFF00
    } else {
        ptr.wrapping_add(1)
    };
    let hi = bus.read(hi_ptr);
    u16::from_le_bytes([lo, hi])
}

fn run_indx(cpu: &mut Cpu, bus: &mut impl Bus, op: OpKind) -> Result<u32, CoreError> {
    let zp = fetch(cpu, bus) as u16;
    let ptr = zp.wrapping_add(cpu.x as u16) & 0xFF;
    let addr = read16_zp(bus, ptr);
    run_mem(cpu, bus, op, addr)
}

fn run_indy(cpu: &mut Cpu, bus: &mut impl Bus, op: OpKind, penalty: bool) -> Result<u32, CoreError> {
    let zp = fetch(cpu, bus) as u16;
    let base = read16_zp(bus, zp);
    let addr = base.wrapping_add(cpu.y as u16);
    let extra = mem_penalty(penalty, op, base, addr);
    run_mem(cpu, bus, op, addr).map(|_| extra as u32)
}

fn run_rel(cpu: &mut Cpu, bus: &mut impl Bus, op: OpKind) -> Result<u32, CoreError> {
    let take = match op {
        OpKind::Bpl => cpu.status & N == 0,
        OpKind::Bmi => cpu.status & N != 0,
        OpKind::Bvc => cpu.status & V == 0,
        OpKind::Bvs => cpu.status & V != 0,
        OpKind::Bcc => cpu.status & C == 0,
        OpKind::Bcs => cpu.status & C != 0,
        OpKind::Bne => cpu.status & Z == 0,
        OpKind::Beq => cpu.status & Z != 0,
        _ => return Err(CoreError::Opcode { opcode: 0 }),
    };
    Ok(branch(cpu, bus, take))
}

fn run_mem(cpu: &mut Cpu, bus: &mut impl Bus, op: OpKind, addr: u16) -> Result<u32, CoreError> {
    if op.is_store() {
        store_op(cpu, bus, op, addr)?;
        return Ok(0);
    }
    if op.is_modify() {
        modify_op(cpu, bus, op, addr)?;
        return Ok(0);
    }
    apply_read(cpu, bus, op, addr).map(|_| 0)
}

fn store_op(cpu: &mut Cpu, bus: &mut impl Bus, op: OpKind, addr: u16) -> Result<(), CoreError> {
    let value = match op {
        OpKind::Sta => cpu.a,
        OpKind::Stx => cpu.x,
        OpKind::Sty => cpu.y,
        _ => return Err(CoreError::Opcode { opcode: 0 }),
    };
    bus.write(addr, value);
    Ok(())
}

fn modify_op(cpu: &mut Cpu, bus: &mut impl Bus, op: OpKind, addr: u16) -> Result<(), CoreError> {
    let value = bus.read(addr);
    let result = match op {
        OpKind::Asl => asl(cpu, value),
        OpKind::Lsr => lsr(cpu, value),
        OpKind::Rol => rol(cpu, value),
        OpKind::Ror => ror(cpu, value),
        OpKind::Inc => inc(cpu, value),
        OpKind::Dec => dec(cpu, value),
        _ => return Err(CoreError::Opcode { opcode: 0 }),
    };
    bus.write(addr, result);
    Ok(())
}

fn apply_read(cpu: &mut Cpu, bus: &mut impl Bus, op: OpKind, addr: u16) -> Result<u32, CoreError> {
    let value = bus.read(addr);
    apply_load_alu(cpu, op, value)?;
    Ok(0)
}

fn apply_load_alu(cpu: &mut Cpu, op: OpKind, value: u8) -> Result<(), CoreError> {
    match op {
        OpKind::Lda => {
            cpu.a = value;
            set_nz(cpu, cpu.a);
        }
        OpKind::Ldx => {
            cpu.x = value;
            set_nz(cpu, cpu.x);
        }
        OpKind::Ldy => {
            cpu.y = value;
            set_nz(cpu, cpu.y);
        }
        OpKind::Ora => {
            cpu.a |= value;
            set_nz(cpu, cpu.a);
        }
        OpKind::And => {
            cpu.a &= value;
            set_nz(cpu, cpu.a);
        }
        OpKind::Eor => {
            cpu.a ^= value;
            set_nz(cpu, cpu.a);
        }
        OpKind::Adc => adc(cpu, value),
        OpKind::Sbc => sbc(cpu, value),
        OpKind::Cmp => compare(cpu, cpu.a, value),
        OpKind::Cpx => compare(cpu, cpu.x, value),
        OpKind::Cpy => compare(cpu, cpu.y, value),
        OpKind::Bit => bit(cpu, value),
        _ => return Err(CoreError::Opcode { opcode: 0 }),
    }
    Ok(())
}

impl OpKind {
    fn is_store(self) -> bool {
        matches!(self, OpKind::Sta | OpKind::Stx | OpKind::Sty)
    }

    fn is_modify(self) -> bool {
        matches!(
            self,
            OpKind::Asl | OpKind::Lsr | OpKind::Rol | OpKind::Ror | OpKind::Inc | OpKind::Dec
        )
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Addr {
    Imp,
    Acc,
    Imm,
    Zp,
    Zpx,
    Zpy,
    Abs,
    Absx,
    Absy,
    Ind,
    Indx,
    Indy,
    Rel,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum OpKind {
    Illegal,
    Adc,
    And,
    Asl,
    Bcc,
    Bcs,
    Beq,
    Bit,
    Bmi,
    Bne,
    Bpl,
    Brk,
    Bvc,
    Bvs,
    Clc,
    Cld,
    Cli,
    Clv,
    Cmp,
    Cpx,
    Cpy,
    Dec,
    Dex,
    Dey,
    Eor,
    Inc,
    Inx,
    Iny,
    Jmp,
    Jsr,
    Lda,
    Ldx,
    Ldy,
    Lsr,
    Nop,
    Ora,
    Pha,
    Php,
    Pla,
    Plp,
    Rol,
    Ror,
    Rti,
    Rts,
    Sbc,
    Sec,
    Sed,
    Sei,
    Sta,
    Stx,
    Sty,
    Tax,
    Tay,
    Tsx,
    Txa,
    Txs,
    Tya,
}

#[derive(Clone, Copy)]
struct OpEntry {
    kind: OpKind,
    mode: Addr,
    base_cycles: u8,
    page_penalty: bool,
}

const OPCODES: [OpEntry; 256] = build_opcodes();

const fn op(
    kind: OpKind,
    mode: Addr,
    base_cycles: u8,
    page_penalty: bool,
) -> OpEntry {
    OpEntry {
        kind,
        mode,
        base_cycles,
        page_penalty,
    }
}

const fn illegal() -> OpEntry {
    op(OpKind::Illegal, Addr::Imp, 0, false)
}

const fn build_opcodes() -> [OpEntry; 256] {
    [
        op(OpKind::Brk, Addr::Imp, 7, false),
        op(OpKind::Ora, Addr::Indx, 6, false),
        illegal(),
        illegal(),
        illegal(),
        op(OpKind::Ora, Addr::Zp, 3, false),
        op(OpKind::Asl, Addr::Zp, 5, false),
        illegal(),
        op(OpKind::Php, Addr::Imp, 3, false),
        op(OpKind::Ora, Addr::Imm, 2, false),
        op(OpKind::Asl, Addr::Acc, 2, false),
        illegal(),
        illegal(),
        op(OpKind::Ora, Addr::Abs, 4, false),
        op(OpKind::Asl, Addr::Abs, 6, false),
        illegal(),
        op(OpKind::Bpl, Addr::Rel, 2, false),
        op(OpKind::Ora, Addr::Indy, 5, true),
        illegal(),
        illegal(),
        illegal(),
        op(OpKind::Ora, Addr::Zpx, 4, false),
        op(OpKind::Asl, Addr::Zpx, 6, false),
        illegal(),
        op(OpKind::Clc, Addr::Imp, 2, false),
        op(OpKind::Ora, Addr::Absy, 4, true),
        illegal(),
        illegal(),
        illegal(),
        op(OpKind::Ora, Addr::Absx, 4, true),
        op(OpKind::Asl, Addr::Absx, 7, false),
        illegal(),
        op(OpKind::Jsr, Addr::Abs, 6, false),
        op(OpKind::And, Addr::Indx, 6, false),
        illegal(),
        illegal(),
        op(OpKind::Bit, Addr::Zp, 3, false),
        op(OpKind::And, Addr::Zp, 3, false),
        op(OpKind::Rol, Addr::Zp, 5, false),
        illegal(),
        op(OpKind::Plp, Addr::Imp, 4, false),
        op(OpKind::And, Addr::Imm, 2, false),
        op(OpKind::Rol, Addr::Acc, 2, false),
        illegal(),
        op(OpKind::Bit, Addr::Abs, 4, false),
        op(OpKind::And, Addr::Abs, 4, false),
        op(OpKind::Rol, Addr::Abs, 6, false),
        illegal(),
        op(OpKind::Bmi, Addr::Rel, 2, false),
        op(OpKind::And, Addr::Indy, 5, true),
        illegal(),
        illegal(),
        illegal(),
        op(OpKind::And, Addr::Zpx, 4, false),
        op(OpKind::Rol, Addr::Zpx, 6, false),
        illegal(),
        op(OpKind::Sec, Addr::Imp, 2, false),
        op(OpKind::And, Addr::Absy, 4, true),
        illegal(),
        illegal(),
        illegal(),
        op(OpKind::And, Addr::Absx, 4, true),
        op(OpKind::Rol, Addr::Absx, 7, false),
        illegal(),
        op(OpKind::Rti, Addr::Imp, 6, false),
        op(OpKind::Eor, Addr::Indx, 6, false),
        illegal(),
        illegal(),
        illegal(),
        op(OpKind::Eor, Addr::Zp, 3, false),
        op(OpKind::Lsr, Addr::Zp, 5, false),
        illegal(),
        op(OpKind::Pha, Addr::Imp, 3, false),
        op(OpKind::Eor, Addr::Imm, 2, false),
        op(OpKind::Lsr, Addr::Acc, 2, false),
        illegal(),
        op(OpKind::Jmp, Addr::Abs, 3, false),
        op(OpKind::Eor, Addr::Abs, 4, false),
        op(OpKind::Lsr, Addr::Abs, 6, false),
        illegal(),
        op(OpKind::Bvc, Addr::Rel, 2, false),
        op(OpKind::Eor, Addr::Indy, 5, true),
        illegal(),
        illegal(),
        illegal(),
        op(OpKind::Eor, Addr::Zpx, 4, false),
        op(OpKind::Lsr, Addr::Zpx, 6, false),
        illegal(),
        op(OpKind::Cli, Addr::Imp, 2, false),
        op(OpKind::Eor, Addr::Absy, 4, true),
        illegal(),
        illegal(),
        illegal(),
        op(OpKind::Eor, Addr::Absx, 4, true),
        op(OpKind::Lsr, Addr::Absx, 7, false),
        illegal(),
        op(OpKind::Rts, Addr::Imp, 6, false),
        op(OpKind::Adc, Addr::Indx, 6, false),
        illegal(),
        illegal(),
        illegal(),
        op(OpKind::Adc, Addr::Zp, 3, false),
        op(OpKind::Ror, Addr::Zp, 5, false),
        illegal(),
        op(OpKind::Pla, Addr::Imp, 4, false),
        op(OpKind::Adc, Addr::Imm, 2, false),
        op(OpKind::Ror, Addr::Acc, 2, false),
        illegal(),
        op(OpKind::Jmp, Addr::Ind, 5, false),
        op(OpKind::Adc, Addr::Abs, 4, false),
        op(OpKind::Ror, Addr::Abs, 6, false),
        illegal(),
        op(OpKind::Bvs, Addr::Rel, 2, false),
        op(OpKind::Adc, Addr::Indy, 5, true),
        illegal(),
        illegal(),
        illegal(),
        op(OpKind::Adc, Addr::Zpx, 4, false),
        op(OpKind::Ror, Addr::Zpx, 6, false),
        illegal(),
        op(OpKind::Sei, Addr::Imp, 2, false),
        op(OpKind::Adc, Addr::Absy, 4, true),
        illegal(),
        illegal(),
        illegal(),
        op(OpKind::Adc, Addr::Absx, 4, true),
        op(OpKind::Ror, Addr::Absx, 7, false),
        illegal(),
        illegal(),
        op(OpKind::Sta, Addr::Indx, 6, false),
        illegal(),
        illegal(),
        op(OpKind::Sty, Addr::Zp, 3, false),
        op(OpKind::Sta, Addr::Zp, 3, false),
        op(OpKind::Stx, Addr::Zp, 3, false),
        illegal(),
        op(OpKind::Dey, Addr::Imp, 2, false),
        illegal(),
        op(OpKind::Txa, Addr::Imp, 2, false),
        illegal(),
        op(OpKind::Sty, Addr::Abs, 4, false),
        op(OpKind::Sta, Addr::Abs, 4, false),
        op(OpKind::Stx, Addr::Abs, 4, false),
        illegal(),
        op(OpKind::Bcc, Addr::Rel, 2, false),
        op(OpKind::Sta, Addr::Indy, 6, false),
        illegal(),
        illegal(),
        op(OpKind::Sty, Addr::Zpx, 4, false),
        op(OpKind::Sta, Addr::Zpx, 4, false),
        op(OpKind::Stx, Addr::Zpy, 4, false),
        illegal(),
        op(OpKind::Tya, Addr::Imp, 2, false),
        op(OpKind::Sta, Addr::Absy, 5, false),
        op(OpKind::Txs, Addr::Imp, 2, false),
        illegal(),
        illegal(),
        op(OpKind::Sta, Addr::Absx, 5, false),
        illegal(),
        illegal(),
        op(OpKind::Ldy, Addr::Imm, 2, false),
        op(OpKind::Lda, Addr::Indx, 6, false),
        op(OpKind::Ldx, Addr::Imm, 2, false),
        illegal(),
        op(OpKind::Ldy, Addr::Zp, 3, false),
        op(OpKind::Lda, Addr::Zp, 3, false),
        op(OpKind::Ldx, Addr::Zp, 3, false),
        illegal(),
        op(OpKind::Tay, Addr::Imp, 2, false),
        op(OpKind::Lda, Addr::Imm, 2, false),
        op(OpKind::Tax, Addr::Imp, 2, false),
        illegal(),
        op(OpKind::Ldy, Addr::Abs, 4, false),
        op(OpKind::Lda, Addr::Abs, 4, false),
        op(OpKind::Ldx, Addr::Abs, 4, false),
        illegal(),
        op(OpKind::Bcs, Addr::Rel, 2, false),
        op(OpKind::Lda, Addr::Indy, 5, true),
        illegal(),
        illegal(),
        op(OpKind::Ldy, Addr::Zpx, 4, false),
        op(OpKind::Lda, Addr::Zpx, 4, false),
        op(OpKind::Ldx, Addr::Zpy, 4, false),
        illegal(),
        op(OpKind::Clv, Addr::Imp, 2, false),
        op(OpKind::Lda, Addr::Absy, 4, true),
        op(OpKind::Tsx, Addr::Imp, 2, false),
        illegal(),
        op(OpKind::Ldy, Addr::Absx, 4, true),
        op(OpKind::Lda, Addr::Absx, 4, true),
        op(OpKind::Ldx, Addr::Absy, 4, true),
        illegal(),
        op(OpKind::Cpy, Addr::Imm, 2, false),
        op(OpKind::Cmp, Addr::Indx, 6, false),
        illegal(),
        illegal(),
        op(OpKind::Cpy, Addr::Zp, 3, false),
        op(OpKind::Cmp, Addr::Zp, 3, false),
        op(OpKind::Dec, Addr::Zp, 5, false),
        illegal(),
        op(OpKind::Iny, Addr::Imp, 2, false),
        op(OpKind::Cmp, Addr::Imm, 2, false),
        op(OpKind::Dex, Addr::Imp, 2, false),
        illegal(),
        op(OpKind::Cpy, Addr::Abs, 4, false),
        op(OpKind::Cmp, Addr::Abs, 4, false),
        op(OpKind::Dec, Addr::Abs, 6, false),
        illegal(),
        op(OpKind::Bne, Addr::Rel, 2, false),
        op(OpKind::Cmp, Addr::Indy, 5, true),
        illegal(),
        illegal(),
        illegal(),
        op(OpKind::Cmp, Addr::Zpx, 4, false),
        op(OpKind::Dec, Addr::Zpx, 6, false),
        illegal(),
        op(OpKind::Cld, Addr::Imp, 2, false),
        op(OpKind::Cmp, Addr::Absy, 4, true),
        illegal(),
        illegal(),
        illegal(),
        op(OpKind::Cmp, Addr::Absx, 4, true),
        op(OpKind::Dec, Addr::Absx, 7, false),
        illegal(),
        op(OpKind::Cpx, Addr::Imm, 2, false),
        op(OpKind::Sbc, Addr::Indx, 6, false),
        illegal(),
        illegal(),
        op(OpKind::Cpx, Addr::Zp, 3, false),
        op(OpKind::Sbc, Addr::Zp, 3, false),
        op(OpKind::Inc, Addr::Zp, 5, false),
        illegal(),
        op(OpKind::Inx, Addr::Imp, 2, false),
        op(OpKind::Sbc, Addr::Imm, 2, false),
        op(OpKind::Nop, Addr::Imp, 2, false),
        illegal(),
        op(OpKind::Cpx, Addr::Abs, 4, false),
        op(OpKind::Sbc, Addr::Abs, 4, false),
        op(OpKind::Inc, Addr::Abs, 6, false),
        illegal(),
        op(OpKind::Beq, Addr::Rel, 2, false),
        op(OpKind::Sbc, Addr::Indy, 5, true),
        illegal(),
        illegal(),
        illegal(),
        op(OpKind::Sbc, Addr::Zpx, 4, false),
        op(OpKind::Inc, Addr::Zpx, 6, false),
        illegal(),
        op(OpKind::Sed, Addr::Imp, 2, false),
        op(OpKind::Sbc, Addr::Absy, 4, true),
        illegal(),
        illegal(),
        illegal(),
        op(OpKind::Sbc, Addr::Absx, 4, true),
        op(OpKind::Inc, Addr::Absx, 7, false),
        illegal(),
    ]
}
