use scanline_core::{
    BORDER, CoreError, Cpu, Machine, Memory, Ports, attribute_address, bitmap_address, frame_from,
    run_until_halt, step,
};

#[test]
fn memory_map_keeps_ram_and_ignores_rom() {
    let mut memory = Memory::new();
    memory.write(0x0000, 0x12);
    memory.write(0x3FFF, 0x12);
    memory.write(0x4000, 0x12);
    memory.write(0xFFFF, 0x34);
    assert_eq!(memory.read(0x0000), 0xFF);
    assert_eq!(memory.read(0x3FFF), 0xFF);
    assert_eq!(memory.read(0x4000), 0x12);
    assert_eq!(memory.read(0xFFFF), 0x34);
}

#[test]
fn display_addresses_follow_the_spectrum_layout() {
    assert_eq!(bitmap_address(0, 0), 0x4000);
    assert_eq!(bitmap_address(0, 1), 0x4100);
    assert_eq!(bitmap_address(0, 8), 0x4020);
    assert_eq!(bitmap_address(0, 64), 0x4800);
    assert_eq!(attribute_address(0, 0), 0x5800);
    assert_eq!(attribute_address(0, 64), 0x5900);
}

#[test]
fn display_file_linearizes_ink_paper_and_border() {
    let mut memory = Memory::new();
    memory.write(bitmap_address(0, 8), 0x80);
    memory.write(attribute_address(0, 8), 0xC7);
    memory.write(attribute_address(8, 8), 0x18);
    let frame = frame_from(&memory, 2).expect("frame");
    assert_eq!(frame.index[0], 2);
    assert_eq!(ink_at(&frame, 0, 8), 15);
    assert_eq!(ink_at(&frame, 1, 8), 8);
    assert_eq!(ink_at(&frame, 8, 8), 3);
}

#[test]
fn machine_draws_lines_and_advances_the_first_cell() {
    let mut machine = Machine::new();
    let first = machine.frame().expect("first");
    assert_eq!(first.index[0], 1);
    assert_eq!(ink_at(&first, 0, 0), 1);
    assert_eq!(ink_at(&first, 0, 1), 0);
    assert_eq!(ink_at(&first, 8, 1), 8);
    assert_eq!(ink_at(&first, 8, 0), 15);
    assert_eq!(ink_at(&first, 0, 64), 15);
    assert_eq!(ink_at(&first, 0, 128), 15);
    assert_eq!(machine.read(0x5800), 1);
    assert_eq!(machine.read(0x5801), 0x47);
    assert_eq!(machine.read(bitmap_address(0, 64)), 0xFF);
    let second = machine.frame().expect("second");
    assert_eq!(ink_at(&second, 0, 0), 2);
    assert_eq!(second.index[0], 1);
}

#[test]
fn unknown_opcode_and_runaway_loop_name_the_cause() {
    let mut memory = Memory::new();
    memory.write(0x8000, 0xCB);
    let mut cpu = Cpu::default();
    cpu.pc = 0x8000;
    let error = step(&mut cpu, &mut memory, &mut Ports::default()).expect_err("opcode");
    assert!(error.to_string().contains("0xcb"));
    assert_eq!(error, CoreError::Opcode { opcode: 0xCB });
    memory.write(0x8000, 0x18);
    memory.write(0x8001, 0xFE);
    cpu.pc = 0x8000;
    let error = run_until_halt(&mut cpu, &mut memory, &mut Ports::default()).expect_err("limit");
    assert!(error.to_string().contains("step"));
    assert_eq!(error, CoreError::StepLimit);
}

#[test]
fn alu_results_follow_the_documented_flags() {
    let (cpu, _, _) = execute(&[0x3E, 0x01, 0xC6, 0xFF, 0x76]);
    assert_eq!((cpu.a, cpu.f), (0x00, 0x51));
    let (cpu, _, _) = execute(&[0x3E, 0x7F, 0xC6, 0x01, 0x76]);
    assert_eq!((cpu.a, cpu.f), (0x80, 0x94));
    let (cpu, _, _) = execute(&[0x3E, 0xFF, 0xC6, 0x01, 0x3E, 0xFF, 0xCE, 0x00, 0x76]);
    assert_eq!((cpu.a, cpu.f), (0x00, 0x51));
    let (cpu, _, _) = execute(&[0x3E, 0x00, 0xD6, 0x01, 0x76]);
    assert_eq!((cpu.a, cpu.f), (0xFF, 0x93));
    let (cpu, _, _) = execute(&[0x3E, 0x01, 0xC6, 0xFF, 0x3E, 0x00, 0xDE, 0x00, 0x76]);
    assert_eq!((cpu.a, cpu.f), (0xFF, 0x93));
    let (cpu, _, _) = execute(&[0x3E, 0x80, 0xD6, 0x01, 0x76]);
    assert_eq!((cpu.a, cpu.f), (0x7F, 0x16));
    let (cpu, _, _) = execute(&[0x3E, 0xF0, 0xE6, 0x0F, 0x76]);
    assert_eq!((cpu.a, cpu.f), (0x00, 0x54));
    let (cpu, _, _) = execute(&[0x3E, 0xFF, 0xEE, 0xFF, 0x76]);
    assert_eq!((cpu.a, cpu.f), (0x00, 0x44));
    let (cpu, _, _) = execute(&[0x3E, 0x01, 0xEE, 0x00, 0x76]);
    assert_eq!((cpu.a, cpu.f), (0x01, 0x00));
    let (cpu, _, _) = execute(&[0x3E, 0x01, 0xF6, 0x02, 0x76]);
    assert_eq!((cpu.a, cpu.f), (0x03, 0x04));
    let (cpu, _, _) = execute(&[0x3E, 0x00, 0xFE, 0x01, 0x76]);
    assert_eq!((cpu.a, cpu.f), (0x00, 0x93));
    let (cpu, _, _) = execute(&[0x06, 0x01, 0x3E, 0x01, 0x80, 0x76]);
    assert_eq!(cpu.a, 2);
    let (cpu, memory, _) = execute(&[0x21, 0x00, 0x40, 0x36, 0x01, 0x3E, 0x02, 0x86, 0x76]);
    assert_eq!(cpu.a, 3);
    assert_eq!(memory.read(0x4000), 1);
}

#[test]
fn increments_and_decrements_keep_carry() {
    let (cpu, _, _) = execute(&[0x3E, 0x01, 0xC6, 0xFF, 0x04, 0x76]);
    assert_eq!((cpu.b, cpu.f), (1, 0x01));
    let (cpu, _, _) = execute(&[0x3E, 0x7F, 0x3C, 0x76]);
    assert_eq!((cpu.a, cpu.f), (0x80, 0x94));
    let (cpu, _, _) = execute(&[0x3E, 0x0F, 0x3C, 0x76]);
    assert_eq!(cpu.f, 0x10);
    let (cpu, _, _) = execute(&[0x3E, 0xFF, 0x3C, 0x76]);
    assert_eq!((cpu.a, cpu.f), (0x00, 0x50));
    let (cpu, _, _) = execute(&[0x3E, 0x80, 0x3D, 0x76]);
    assert_eq!((cpu.a, cpu.f), (0x7F, 0x16));
    let (cpu, _, _) = execute(&[0x3E, 0x01, 0x3D, 0x76]);
    assert_eq!((cpu.a, cpu.f), (0x00, 0x42));
    let (cpu, _, _) = execute(&[0x3E, 0x10, 0x3D, 0x76]);
    assert_eq!((cpu.a, cpu.f), (0x0F, 0x12));
    let (cpu, _, _) = execute(&[0x3E, 0x00, 0x3D, 0x76]);
    assert_eq!((cpu.a, cpu.f), (0xFF, 0x92));
    let (cpu, _, _) = execute(&[0x3E, 0x01, 0xC6, 0xFF, 0x05, 0x76]);
    assert_eq!((cpu.b, cpu.f), (0xFF, 0x93));
    let (cpu, memory, _) = execute(&[0x21, 0x00, 0x40, 0x36, 0x7F, 0x34, 0x35, 0x76]);
    assert_eq!(memory.read(0x4000), 0x7F);
    assert_eq!(cpu.f & 0x01, 0);
}

#[test]
fn loads_move_registers_memory_and_pairs() {
    let (cpu, _, _) = execute(&[
        0x06, 0x10, 0x0E, 0x20, 0x16, 0x30, 0x1E, 0x40, 0x26, 0x50, 0x2E, 0x60, 0x3E, 0x70, 0x78,
        0x79, 0x7A, 0x7B, 0x7C, 0x7D, 0x76,
    ]);
    assert_eq!(cpu.a, 0x60);
    assert_eq!(cpu.b, 0x10);
    let (cpu, memory, _) = execute(&[
        0x21, 0x02, 0x40, 0x36, 0x99, 0x3E, 0x22, 0x77, 0x3E, 0x00, 0x7E, 0x76,
    ]);
    assert_eq!(memory.read(0x4002), 0x22);
    assert_eq!(cpu.a, 0x22);
    let (cpu, memory, _) = execute(&[0x01, 0x00, 0x40, 0x3E, 0x42, 0x02, 0x3E, 0x00, 0x0A, 0x76]);
    assert_eq!((cpu.a, memory.read(0x4000)), (0x42, 0x42));
    let (cpu, memory, _) = execute(&[0x11, 0x01, 0x40, 0x3E, 0x43, 0x12, 0x3E, 0x00, 0x1A, 0x76]);
    assert_eq!((cpu.a, memory.read(0x4001)), (0x43, 0x43));
    let (cpu, memory, _) = execute(&[
        0x3E, 0x5A, 0x32, 0x10, 0x40, 0x3E, 0x00, 0x3A, 0x10, 0x40, 0x76,
    ]);
    assert_eq!((cpu.a, memory.read(0x4010)), (0x5A, 0x5A));
    let (cpu, _, _) = execute(&[0x01, 0x00, 0x00, 0x3E, 0x12, 0x02, 0x0A, 0x76]);
    assert_eq!(cpu.a, 0xFF);
    let (cpu, _, _) = execute(&[
        0x01, 0x00, 0x10, 0x03, 0x11, 0x00, 0x20, 0x13, 0x21, 0x00, 0x30, 0x23, 0x31, 0x00, 0x40,
        0x33, 0xF9, 0xEB, 0x76,
    ]);
    assert_eq!(cpu.b, 0x10);
    assert_eq!(cpu.c, 0x01);
    assert_eq!((cpu.d, cpu.e), (0x30, 0x01));
    assert_eq!((cpu.h, cpu.l), (0x20, 0x01));
    assert_eq!(cpu.sp, 0x3001);
    let (cpu, _, _) = execute(&[0x0B, 0x1B, 0x2B, 0x3B, 0x76]);
    assert_eq!(pair(cpu.b, cpu.c), 0xFFFF);
    assert_eq!(pair(cpu.d, cpu.e), 0xFFFF);
    assert_eq!(pair(cpu.h, cpu.l), 0xFFFF);
    assert_eq!(cpu.sp, 0xFFFF);
    let (cpu, _, _) = execute(&[0x00, 0x76]);
    assert!(cpu.halted);
}

#[test]
fn jumps_follow_flags_and_djnz_counts() {
    let (cpu, _, _) = execute(&[
        0x3E, 0x00, 0xFE, 0x00, 0x20, 0x03, 0x28, 0x02, 0x3E, 0x01, 0x76,
    ]);
    assert_eq!(cpu.a, 0);
    let (cpu, _, _) = execute(&[
        0x3E, 0x00, 0xC6, 0x00, 0x38, 0x03, 0x30, 0x02, 0x3E, 0x02, 0x76,
    ]);
    assert_eq!(cpu.a, 0);
    let (cpu, _, _) = execute(&[0x06, 0x02, 0x0C, 0x10, 0xFD, 0x76]);
    assert_eq!((cpu.b, cpu.c), (0, 2));
    let (cpu, _, _) = execute(&[0xC3, 0x05, 0x80, 0x3E, 0x01, 0x76]);
    assert_eq!(cpu.a, 0);
    assert_eq!(jump_target(0xC2, 0x00), 0x9000);
    assert_eq!(jump_target(0xC2, 0x40), 0x8003);
    assert_eq!(jump_target(0xCA, 0x40), 0x9000);
    assert_eq!(jump_target(0xCA, 0x00), 0x8003);
    assert_eq!(jump_target(0xD2, 0x00), 0x9000);
    assert_eq!(jump_target(0xD2, 0x01), 0x8003);
    assert_eq!(jump_target(0xDA, 0x01), 0x9000);
    assert_eq!(jump_target(0xDA, 0x00), 0x8003);
    assert_eq!(jump_target(0xE2, 0x00), 0x9000);
    assert_eq!(jump_target(0xE2, 0x04), 0x8003);
    assert_eq!(jump_target(0xEA, 0x04), 0x9000);
    assert_eq!(jump_target(0xEA, 0x00), 0x8003);
    assert_eq!(jump_target(0xF2, 0x00), 0x9000);
    assert_eq!(jump_target(0xF2, 0x80), 0x8003);
    assert_eq!(jump_target(0xFA, 0x80), 0x9000);
    assert_eq!(jump_target(0xFA, 0x00), 0x8003);
}

#[test]
fn border_port_stores_three_bits_and_in_returns_ff() {
    let (cpu, _, ports) = execute(&[0x3E, 0x05, 0xD3, 0xFE, 0x3E, 0x07, 0xD3, 0x00, 0x76]);
    assert_eq!(ports.border, 5);
    assert_eq!(cpu.a, 7);
    let (cpu, _, _) = execute(&[0x3E, 0x01, 0xC6, 0xFF, 0xDB, 0xFE, 0x76]);
    assert_eq!((cpu.a, cpu.f), (0xFF, 0x51));
}

fn execute(bytes: &[u8]) -> (Cpu, Memory, Ports) {
    let mut memory = Memory::new();
    poke(&mut memory, 0x8000, bytes);
    let mut cpu = Cpu::default();
    cpu.pc = 0x8000;
    let mut ports = Ports::default();
    let end = 0x8000 + u16::try_from(bytes.len()).expect("len");
    let mut guard = 0;
    while cpu.pc != end && !cpu.halted {
        step(&mut cpu, &mut memory, &mut ports).expect("step");
        guard += 1;
        assert!(guard < 1000);
    }
    (cpu, memory, ports)
}

fn poke(memory: &mut Memory, address: u16, bytes: &[u8]) {
    let mut index = 0;
    while index < bytes.len() {
        let at = address.wrapping_add(u16::try_from(index).expect("index"));
        memory.write(at, bytes[index]);
        index += 1;
    }
}

fn jump_target(opcode: u8, flags: u8) -> u16 {
    let mut memory = Memory::new();
    poke(&mut memory, 0x8000, &[opcode, 0x00, 0x90]);
    let mut cpu = Cpu::default();
    cpu.pc = 0x8000;
    cpu.f = flags;
    step(&mut cpu, &mut memory, &mut Ports::default()).expect("jump");
    cpu.pc
}

fn ink_at(frame: &scanline_core::Frame, x: u16, y: u16) -> u8 {
    let px = usize::from(x + BORDER);
    let py = usize::from(y + BORDER);
    frame.index[py * usize::from(frame.width) + px]
}

fn pair(high: u8, low: u8) -> u16 {
    (u16::from(high) << 8) | u16::from(low)
}
