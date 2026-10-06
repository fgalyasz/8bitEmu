use crate::memory::Memory;

pub const ENTRY: u16 = 0x8000;

pub fn load(memory: &mut Memory) {
    let mut cursor = ENTRY;
    emit(&mut cursor, memory, &[0x3E, 0x01, 0xD3, 0xFE]);
    paint_line(&mut cursor, memory, 0x4000);
    paint_line(&mut cursor, memory, 0x4800);
    paint_line(&mut cursor, memory, 0x5000);
    paint_attributes(&mut cursor, memory);
    emit_live(&mut cursor, memory);
}

fn paint_line(cursor: &mut u16, memory: &mut Memory, address: u16) {
    emit_pair(cursor, memory, 0x21, address);
    emit(cursor, memory, &[0x06, 32]);
    let loop_at = *cursor;
    emit(cursor, memory, &[0x36, 0xFF, 0x23]);
    emit_rel(cursor, memory, 0x10, loop_at);
}

fn paint_attributes(cursor: &mut u16, memory: &mut Memory) {
    emit_pair(cursor, memory, 0x21, 0x5800);
    emit(cursor, memory, &[0x3E, 0x47, 0x0E, 0x03]);
    let outer = *cursor;
    emit(cursor, memory, &[0x06, 0x00]);
    let inner = *cursor;
    emit(cursor, memory, &[0x77, 0x23]);
    emit_rel(cursor, memory, 0x10, inner);
    emit(cursor, memory, &[0x0D]);
    emit_rel(cursor, memory, 0x20, outer);
}

fn emit_live(cursor: &mut u16, memory: &mut Memory) {
    emit_pair(cursor, memory, 0x21, 0x5800);
    emit(cursor, memory, &[0x36, 0x00]);
    let live = *cursor;
    emit(cursor, memory, &[0x7E, 0x3C, 0xE6, 0x07, 0x77, 0x76]);
    emit_rel(cursor, memory, 0x18, live);
}

fn emit_pair(cursor: &mut u16, memory: &mut Memory, opcode: u8, value: u16) {
    emit(cursor, memory, &[opcode, value as u8, (value >> 8) as u8]);
}

fn emit_rel(cursor: &mut u16, memory: &mut Memory, opcode: u8, target: u16) {
    let next = cursor.wrapping_add(2);
    let relative = target.wrapping_sub(next) as u8;
    emit(cursor, memory, &[opcode, relative]);
}

fn emit(cursor: &mut u16, memory: &mut Memory, bytes: &[u8]) {
    let mut index = 0;
    while index < bytes.len() {
        memory.write(*cursor, bytes[index]);
        *cursor = cursor.wrapping_add(1);
        index += 1;
    }
}
