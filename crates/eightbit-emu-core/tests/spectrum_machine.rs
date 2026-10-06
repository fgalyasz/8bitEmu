use eightbit_emu_core::{CoreError, Machine};

#[test]
fn interrupt_waits_until_the_second_frame() {
    let mut rom = vec![0u8; 16384];
    rom[0] = 0xFB;
    rom[1] = 0x76;
    rom[0x38] = 0x3E;
    rom[0x39] = 0x99;
    rom[0x3A] = 0x32;
    rom[0x3B] = 0x00;
    rom[0x3C] = 0x40;
    rom[0x3D] = 0xC9;
    let mut machine = Machine::new();
    machine.load_rom(&rom, false).expect("rom");
    machine.frame().expect("first");
    assert_eq!(machine.read(0x4000), 0);
    machine.frame().expect("second");
    assert_eq!(machine.read(0x4000), 0x99);
}

#[test]
fn keyboard_row_clears_only_the_held_bit() {
    let mut down = Machine::new();
    down.load_rom(&program(&[0x3E, 0xF7, 0xDB, 0xFE, 0x32, 0x00, 0x40, 0x76]), false)
        .expect("rom");
    down.set_key(3, 0x01, true);
    down.frame().expect("frame");
    assert_eq!(down.read(0x4000), 0xFE);
    let mut up = Machine::new();
    up.load_rom(&program(&[0x3E, 0xF7, 0xDB, 0xFE, 0x32, 0x00, 0x40, 0x76]), false)
        .expect("rom");
    up.frame().expect("frame");
    assert_eq!(up.read(0x4000), 0xFF);
}

#[test]
fn paging_keeps_banks_apart_until_the_port_locks() {
    let mut machine = Machine::new();
    machine
        .load_rom(
            &program(&[
                0x21, 0x00, 0xC0, 0x36, 0x11, 0x3E, 0x01, 0x01, 0xFD, 0x7F, 0xED, 0x79, 0x36, 0x22,
                0x3E, 0x00, 0xED, 0x79, 0x76,
            ]),
            true,
        )
        .expect("rom");
    machine.frame().expect("frame");
    assert_eq!(machine.read(0xC000), 0x11);
    assert_eq!(machine.bank_byte(1, 0), 0x22);
    let mut locked = Machine::new();
    locked
        .load_rom(
            &program(&[
                0x3E, 0x21, 0x01, 0xFD, 0x7F, 0xED, 0x79, 0x3E, 0x02, 0xED, 0x79, 0x21, 0x00, 0xC0,
                0x36, 0x55, 0x76,
            ]),
            true,
        )
        .expect("rom");
    locked.frame().expect("frame");
    assert_eq!(locked.bank_byte(1, 0), 0x55);
    assert_eq!(locked.bank_byte(2, 0), 0);
}

#[test]
fn shadow_screen_is_the_bank_selected_by_bit_3() {
    let mut machine = Machine::new();
    machine
        .load_rom(
            &program(&[
                0x3E, 0x0F, 0x01, 0xFD, 0x7F, 0xED, 0x79, 0x21, 0x00, 0xC0, 0x36, 0xFF, 0x21, 0x00,
                0xD8, 0x36, 0x47, 0x76,
            ]),
            true,
        )
        .expect("rom");
    let frame = machine.frame().expect("frame");
    assert_eq!(ink_at(&frame, 0, 0), 15);
    assert_eq!(machine.bank_byte(5, 0), 0);
}

#[test]
fn snapshots_restore_registers_banks_and_border() {
    let mut bytes = vec![0u8; 49179];
    bytes[19] = 0x04;
    bytes[21] = 0x00;
    bytes[22] = 0x42;
    bytes[23] = 0x00;
    bytes[24] = 0x40;
    bytes[26] = 4;
    bytes[27] = 0x34;
    bytes[28] = 0x12;
    bytes[29] = 0x99;
    let mut machine = Machine::new();
    machine.load_sna(&bytes).expect("sna");
    assert_eq!(machine.read(0x4002), 0x99);
    assert_eq!(machine.border(), 4);
    let frame = machine.frame().expect("frame");
    assert_eq!(frame.index[0], 4);
    let mut wide = vec![0u8; 131_103];
    wide[27] = 0x5A;
    let extra = 27 + 48 * 1024;
    wide[extra] = 0x11;
    wide[131_099] = 0x00;
    wide[131_100] = 0x80;
    let mut paged = Machine::new();
    paged.load_sna(&wide).expect("sna128");
    assert_eq!(paged.bank_byte(5, 0), 0x5A);
    assert_eq!(paged.bank_byte(1, 0), 0x11);
}

#[test]
fn a_short_image_names_the_length() {
    let mut machine = Machine::new();
    let error = machine.load_rom(&[0; 10], false).expect_err("rom");
    assert!(error.to_string().contains("rom"));
    assert!(error.to_string().contains("10"));
    assert_eq!(
        error,
        CoreError::ImageLength {
            name: "rom",
            actual: 10
        }
    );
    let snap = machine.load_sna(&[0; 4]).expect_err("sna");
    assert!(snap.to_string().contains("snapshot"));
}

#[test]
fn beeper_and_ay_reach_the_frame_buffer() {
    let mut machine = Machine::new();
    machine
        .load_rom(
            &program(&[0x3E, 0x10, 0xD3, 0xFE, 0x06, 0x00, 0x10, 0xFE, 0x3E, 0x00, 0xD3, 0xFE, 0x76]),
            false,
        )
        .expect("rom");
    machine.frame().expect("frame");
    let audio = machine.take_audio();
    assert!(audio.iter().any(|sample| *sample > 0.0));
    assert!(audio.iter().any(|sample| *sample < 0.0));
    let mut quiet = Machine::new();
    quiet.load_rom(&program(&[0x76]), false).expect("rom");
    quiet.frame().expect("frame");
    let held = quiet.take_audio();
    assert_eq!(held.len(), 960);
    assert!(held.iter().all(|sample| *sample == -0.2));
    let mut ay = Machine::new();
    ay.load_rom(
        &program(&[
            0x3E, 0x00, 0x01, 0xFD, 0xFF, 0xED, 0x79, 0x3E, 0x34, 0x01, 0xFD, 0xBF, 0xED, 0x79,
            0x01, 0xFD, 0xFF, 0xED, 0x78, 0x32, 0x00, 0x40, 0x76,
        ]),
        true,
    )
    .expect("rom");
    ay.frame().expect("frame");
    assert_eq!(ay.read(0x4000), 0x34);
}

#[test]
fn outd_writes_the_ay_data_port() {
    let mut machine = Machine::new();
    machine
        .load_rom(
            &program(&[
                0x21, 0x00, 0x40, 0x36, 0x0F, 0x3E, 0x07, 0x01, 0xFD, 0xFF, 0xED, 0x79, 0x3E,
                0x3E, 0x01, 0xFD, 0xBF, 0xED, 0x79, 0x3E, 0x00, 0x01, 0xFD, 0xFF, 0xED, 0x79,
                0x3E, 0x01, 0x01, 0xFD, 0xBF, 0xED, 0x79, 0x21, 0x00, 0x40, 0x3E, 0x08, 0x01,
                0xFD, 0xFF, 0xED, 0x79, 0x06, 0xBF, 0xED, 0xAB, 0x76,
            ]),
            true,
        )
        .expect("rom");
    machine.frame().expect("frame");
    let audio = machine.take_audio();
    assert!(audio.iter().any(|sample| *sample > -0.2));
}

#[test]
fn the_border_stripes_when_it_changes_during_the_frame() {
    let mut striped = Machine::new();
    striped.load_rom(&striped_border(), false).expect("rom");
    let frame = striped.frame().expect("frame");
    assert_eq!(border_at(&frame, 0), 1);
    assert_eq!(border_at(&frame, frame.height - 1), 2);
    let mut steady = Machine::new();
    steady.load_rom(&program(&[0x3E, 0x05, 0xD3, 0xFE, 0x76]), false).expect("rom");
    let flat = steady.frame().expect("frame");
    assert_eq!(border_at(&flat, 0), 5);
    assert_eq!(border_at(&flat, flat.height / 2), 5);
    assert_eq!(border_at(&flat, flat.height - 1), 5);
}

fn striped_border() -> Vec<u8> {
    let mut bytes = vec![0x3E, 0x01, 0xD3, 0xFE];
    let mut loops = 0;
    while loops < 4 {
        bytes.extend_from_slice(&[0x06, 0x00, 0x10, 0xFE]);
        loops += 1;
    }
    bytes.extend_from_slice(&[0x3E, 0x02, 0xD3, 0xFE, 0x76]);
    program(&bytes)
}

fn border_at(frame: &eightbit_emu_core::Frame, y: u16) -> u8 {
    frame.index[usize::from(y) * usize::from(frame.width)]
}

#[test]
fn reset_returns_to_address_zero_without_dropping_the_bank() {
    let mut machine = Machine::new();
    machine
        .load_rom(&program(&[0x3E, 0x02, 0xD3, 0xFE, 0x76]), true)
        .expect("rom");
    machine.frame().expect("frame");
    assert_eq!(machine.border(), 2);
    machine.reset();
    assert_eq!(machine.border(), 0);
}

fn program(bytes: &[u8]) -> Vec<u8> {
    let mut rom = vec![0u8; 16384];
    let mut index = 0;
    while index < bytes.len() {
        rom[index] = bytes[index];
        index += 1;
    }
    rom
}

fn ink_at(frame: &eightbit_emu_core::Frame, x: u16, y: u16) -> u8 {
    let px = usize::from(x + 16);
    let py = usize::from(y + 16);
    frame.index[py * usize::from(frame.width) + px]
}
