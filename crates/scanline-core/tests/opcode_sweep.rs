use scanline_core::{Cpu, Memory, Ports, PresentPace, Presenter, step};

#[test]
fn every_documented_opcode_executes_once() {
    let mut opcode = 0u16;
    while opcode < 256 {
        let byte = opcode as u8;
        exercise(&[byte], false);
        exercise(&[byte], true);
        exercise(&[0xCB, byte], false);
        exercise(&[0xED, byte], true);
        exercise(&[0xDD, byte], false);
        exercise(&[0xFD, byte], true);
        exercise(&[0xDD, 0xCB, 0x01, byte], false);
        exercise(&[0xFD, 0xCB, 0xFF, byte], true);
        opcode += 1;
    }
}

#[test]
fn presenter_loads_images_and_holds_a_refresh() {
    let mut presenter = Presenter::new(PresentPace::Fixed60Hz);
    presenter.load_rom(&vec![0x76; 16384], false).expect("rom");
    presenter.set_key(3, 0x01, true);
    presenter.set_key(3, 0x01, false);
    presenter.set_key(8, 0x01, true);
    presenter.on_display_tick().expect("tick");
    presenter.on_display_tick().expect("hold");
    assert!(!presenter.take_audio().is_empty());
    assert!(presenter.take_audio().is_empty());
    presenter.reset();
    let mut wide = vec![0x76; 32768];
    wide[0] = 0x00;
    presenter.load_rom(&wide, true).expect("wide");
    presenter.load_rom(&[0; 10], true).expect_err("short");
    let mut sna = vec![0u8; 49179];
    sna[23] = 0x00;
    sna[24] = 0x40;
    sna[27] = 0x00;
    sna[28] = 0x00;
    presenter.load_sna(&sna).expect("sna");
}

#[test]
fn im2_and_the_envelope_change_the_frame() {
    let mut rom = vec![0u8; 16384];
    rom[0] = 0xED;
    rom[1] = 0x5E;
    rom[2] = 0xFB;
    rom[3] = 0x76;
    rom[0xFF] = 0x00;
    rom[0x100] = 0x02;
    rom[0x200] = 0x3E;
    rom[0x201] = 0x42;
    rom[0x202] = 0x32;
    rom[0x203] = 0x00;
    rom[0x204] = 0x40;
    rom[0x205] = 0xC9;
    let mut machine = scanline_core::Machine::new();
    machine.load_rom(&rom, false).expect("rom");
    machine.frame().expect("first");
    machine.frame().expect("second");
    assert_eq!(machine.read(0x4000), 0x42);
    let mut ay = scanline_core::Machine::new();
    ay.load_rom(&ay_program(), true).expect("rom");
    ay.frame().expect("frame");
    let audio = ay.take_audio();
    assert!(audio.iter().any(|sample| sample.abs() > 0.25));
}

#[test]
fn rejected_banks_and_the_shadow_read_stay_inside_range() {
    let mut memory = Memory::new();
    memory.load_rom(&vec![0; 32768], false).expect_err("48");
    memory.load_rom(&vec![0; 32768], true).expect("128");
    assert_eq!(memory.read_video(0x8000), 0);
    memory.write_bank(8, &[0; 16384]);
    memory.write_bank(0, &[1; 4]);
    assert_eq!(memory.bank_byte(8, 0), 0);
    assert_eq!(memory.bank_byte(0, 20_000), 0);
    let mut machine = scanline_core::Machine::new();
    machine.reset();
    assert_eq!(machine.border(), 0);
}

fn exercise(bytes: &[u8], alternate: bool) {
    let mut memory = Memory::new();
    poke(&mut memory, bytes);
    let mut cpu = Cpu::default();
    cpu.pc = 0x8000;
    cpu.sp = 0xFF00;
    cpu.ix = 0x4100;
    cpu.iy = 0x4200;
    cpu.h = 0x40;
    apply_case(&mut cpu, alternate);
    let mut ports = Ports::default();
    ports.model_128 = alternate;
    let _ = step(&mut cpu, &mut memory, &mut ports);
}

fn apply_case(cpu: &mut Cpu, alternate: bool) {
    if alternate {
        cpu.a = 0x81;
        cpu.f = 0xFF;
        cpu.b = 1;
        cpu.iff2 = true;
        return;
    }
    cpu.a = 0;
    cpu.f = 0;
    cpu.c = 1;
}

fn poke(memory: &mut Memory, bytes: &[u8]) {
    let mut index = 0u16;
    while usize::from(index) < bytes.len() {
        memory.write(0x8000 + index, bytes[usize::from(index)]);
        index += 1;
    }
}

fn ay_program() -> Vec<u8> {
    let mut rom = vec![0u8; 16384];
    let mut bytes = vec![0x3E, 0x10, 0xD3, 0xFE];
    ay_set(&mut bytes, 0, 2);
    ay_set(&mut bytes, 6, 2);
    ay_set(&mut bytes, 7, 0);
    ay_set(&mut bytes, 8, 0x1F);
    ay_set(&mut bytes, 11, 2);
    ay_set(&mut bytes, 13, 0x0E);
    bytes.push(0x76);
    let mut index = 0;
    while index < bytes.len() {
        rom[index] = bytes[index];
        index += 1;
    }
    rom
}

fn ay_set(bytes: &mut Vec<u8>, register: u8, value: u8) {
    bytes.extend_from_slice(&[0x3E, register, 0x01, 0xFD, 0xFF, 0xED, 0x79]);
    bytes.extend_from_slice(&[0x3E, value, 0x01, 0xFD, 0xBF, 0xED, 0x79]);
}
