use eightbit_emu_core::C64Machine;

#[test]
fn latched_pointer_survives_later_change() {
    let mut machine = blank_machine();
    fill_sprite(&mut machine, 0x80, 0xFF);
    fill_sprite(&mut machine, 0x81, 0x00);
    place_sprite(&mut machine, 0, 24, 50, 0x80, 0x0A);
    machine.poke_io(0xD015, 0x01);
    run_past_line(&mut machine, 50);
    machine.write_ram(0x07F8, 0x81);
    let frame = machine.paint().expect("paint");
    assert_eq!(content_pixel(&frame, 0, 0), 0x0A);
}

#[test]
fn live_paint_still_works_without_raster() {
    let mut machine = blank_machine();
    fill_sprite(&mut machine, 0x80, 0xFF);
    place_sprite(&mut machine, 0, 24, 50, 0x80, 0x07);
    machine.poke_io(0xD015, 0x01);
    let frame = machine.paint().expect("paint");
    assert_eq!(content_pixel(&frame, 0, 0), 0x07);
}

#[test]
fn latched_multicolor_keeps_shared_colors() {
    let mut machine = blank_machine();
    machine.write_ram(0x2000, 0b01001011);
    place_sprite(&mut machine, 0, 24, 50, 0x80, 0x0C);
    machine.poke_io(0xD025, 0x0B);
    machine.poke_io(0xD026, 0x01);
    machine.poke_io(0xD01C, 0x01);
    machine.poke_io(0xD015, 0x01);
    run_past_line(&mut machine, 50);
    machine.poke_io(0xD025, 0x02);
    machine.poke_io(0xD027, 0x00);
    let frame = machine.paint().expect("paint");
    assert_eq!(content_pixel(&frame, 0, 0), 0x0B);
    assert_eq!(content_pixel(&frame, 4, 0), 0x0C);
    assert_eq!(content_pixel(&frame, 6, 0), 0x01);
}

fn run_past_line(machine: &mut C64Machine, line: u8) {
    let mut steps = 0u32;
    while steps < 100_000 {
        machine.step_instruction().expect("step");
        if machine.read(0xD012) == line.wrapping_add(1) {
            return;
        }
        steps += 1;
    }
    panic!("did not pass line {line}");
}

fn fill_sprite(machine: &mut C64Machine, page: u8, value: u8) {
    let base = u16::from(page) * 64;
    let mut offset = 0u16;
    while offset < 63 {
        machine.write_ram(base + offset, value);
        offset += 1;
    }
}

fn place_sprite(machine: &mut C64Machine, index: u8, x: u16, y: u8, page: u8, color: u8) {
    machine.poke_io(0xD000 + u16::from(index) * 2, (x & 0xFF) as u8);
    machine.poke_io(0xD001 + u16::from(index) * 2, y);
    machine.write_ram(0x07F8 + u16::from(index), page);
    machine.poke_io(0xD027 + u16::from(index), color);
}

fn content_pixel(frame: &eightbit_emu_core::Frame, x: u16, y: u16) -> u8 {
    const BORDER_PX: u16 = 32;
    let index = usize::from(BORDER_PX + y) * usize::from(frame.width) + usize::from(BORDER_PX + x);
    frame.index[index]
}

fn blank_machine() -> C64Machine {
    let mut machine = C64Machine::new();
    let (mut kernal, basic, chargen) = (vec![0; 8192], vec![0; 8192], vec![0; 4096]);
    kernal[0x1FFC] = 0x00;
    kernal[0x1FFD] = 0xFF;
    kernal[0x1F00] = 0x4C;
    kernal[0x1F01] = 0x00;
    kernal[0x1F02] = 0xFF;
    machine.load_roms(&kernal, &basic, &chargen).expect("roms");
    machine
}

#[test]
fn lower_sprite_index_paints_in_front() {
    let mut machine = blank_machine();
    fill_sprite(&mut machine, 0x80, 0xFF);
    fill_sprite(&mut machine, 0x81, 0xFF);
    place_sprite(&mut machine, 0, 24, 50, 0x80, 0x0A);
    place_sprite(&mut machine, 1, 24, 50, 0x81, 0x01);
    machine.poke_io(0xD015, 0x03);
    let frame = machine.paint().expect("paint");
    assert_eq!(content_pixel(&frame, 0, 0), 0x0A);
}
