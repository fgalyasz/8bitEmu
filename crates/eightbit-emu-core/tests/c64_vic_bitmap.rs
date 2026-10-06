use eightbit_emu_core::C64Machine;

#[test]
fn hires_bitmap_uses_screen_nybbles() {
    let mut machine = blank_machine();
    machine.poke_io(0xD011, 0x3B);
    machine.poke_io(0xD018, 0x18);
    machine.write_ram(0x0400, 0xE6);
    machine.write_ram(0x2000, 0xF0);
    let frame = machine.paint().expect("paint");
    assert_eq!(content_pixel(&frame, 0, 0), 0x0E);
    assert_eq!(content_pixel(&frame, 3, 0), 0x0E);
    assert_eq!(content_pixel(&frame, 4, 0), 0x06);
}

#[test]
fn multicolor_bitmap_uses_pair_colors() {
    let mut machine = blank_machine();
    machine.poke_io(0xD011, 0x3B);
    machine.poke_io(0xD016, 0x18);
    machine.poke_io(0xD018, 0x18);
    machine.poke_io(0xD021, 0x00);
    machine.write_ram(0x0400, 0x12);
    machine.poke_io(0xD800, 0x03);
    machine.write_ram(0x2000, 0b00011011);
    let frame = machine.paint().expect("paint");
    assert_eq!(content_pixel(&frame, 0, 0), 0x00);
    assert_eq!(content_pixel(&frame, 2, 0), 0x01);
    assert_eq!(content_pixel(&frame, 4, 0), 0x02);
    assert_eq!(content_pixel(&frame, 6, 0), 0x03);
}

#[test]
fn raster_compare_raises_irq() {
    let mut machine = blank_machine();
    machine.poke_io(0xD01A, 0x01);
    let control = machine.read(0xD011) & 0x7F;
    machine.poke_io(0xD011, control);
    machine.poke_io(0xD012, 0x40);
    let mut steps = 0u32;
    while steps < 50_000 {
        machine.step_instruction().expect("step");
        let irr = machine.read(0xD019);
        if irr & 0x01 != 0 {
            assert_eq!(irr & 0x80, 0x80);
            machine.poke_io(0xD019, 0x01);
            let cleared = machine.read(0xD019);
            assert_eq!(cleared & 0x01, 0);
            return;
        }
        steps += 1;
    }
    panic!("raster IRQ never fired");
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
