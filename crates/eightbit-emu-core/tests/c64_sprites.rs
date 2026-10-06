use eightbit_emu_core::C64Machine;

#[test]
fn hires_sprite_paints_color() {
    let mut machine = blank_machine();
    fill_solid_sprite(&mut machine, 0x80, 0xFF);
    place_sprite(&mut machine, 0, 24, 50, 0x80, 0x0A);
    machine.poke_io(0xD015, 0x01);
    let frame = machine.paint().expect("paint");
    assert_eq!(content_pixel(&frame, 0, 0), 0x0A);
    assert_eq!(content_pixel(&frame, 23, 0), 0x0A);
}

#[test]
fn clear_sprite_bits_stay_background() {
    let mut machine = blank_machine();
    fill_solid_sprite(&mut machine, 0x80, 0x00);
    place_sprite(&mut machine, 0, 24, 50, 0x80, 0x0A);
    machine.poke_io(0xD015, 0x01);
    let frame = machine.paint().expect("paint");
    assert_eq!(content_pixel(&frame, 0, 0), 0x06);
}

#[test]
fn x_expand_doubles_width() {
    let mut machine = blank_machine();
    machine.write_ram(0x2000, 0x80);
    place_sprite(&mut machine, 0, 24, 50, 0x80, 0x07);
    machine.poke_io(0xD015, 0x01);
    machine.poke_io(0xD01D, 0x01);
    let frame = machine.paint().expect("paint");
    assert_eq!(content_pixel(&frame, 0, 0), 0x07);
    assert_eq!(content_pixel(&frame, 1, 0), 0x07);
    assert_eq!(content_pixel(&frame, 2, 0), 0x06);
}

#[test]
fn multicolor_uses_shared_colors() {
    let mut machine = blank_machine();
    machine.write_ram(0x2000, 0b01001011);
    place_sprite(&mut machine, 0, 24, 50, 0x80, 0x02);
    machine.poke_io(0xD025, 0x04);
    machine.poke_io(0xD026, 0x05);
    machine.poke_io(0xD01C, 0x01);
    machine.poke_io(0xD015, 0x01);
    let frame = machine.paint().expect("paint");
    assert_eq!(content_pixel(&frame, 0, 0), 0x04);
    assert_eq!(content_pixel(&frame, 1, 0), 0x04);
    assert_eq!(content_pixel(&frame, 2, 0), 0x06);
    assert_eq!(content_pixel(&frame, 4, 0), 0x02);
    assert_eq!(content_pixel(&frame, 6, 0), 0x05);
}

fn fill_solid_sprite(machine: &mut C64Machine, page: u8, value: u8) {
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
    if x > 255 {
        machine.poke_io(0xD010, 1 << index);
    }
    machine.write_ram(0x07F8 + u16::from(index), page);
    machine.poke_io(0xD027 + u16::from(index), color);
}

fn content_pixel(frame: &eightbit_emu_core::Frame, x: u16, y: u16) -> u8 {
    const BORDER_PX: u16 = 32;
    let px = BORDER_PX + x;
    let py = BORDER_PX + y;
    let index = usize::from(py) * usize::from(frame.width) + usize::from(px);
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
