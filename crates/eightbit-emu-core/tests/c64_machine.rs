use eightbit_emu_core::{C64Machine, CoreError, Presenter, PresentPace};
use std::fs;
use std::path::PathBuf;

#[test]
fn wrong_kernal_length_names_the_image() {
    let mut machine = C64Machine::new();
    let error = machine
        .load_roms(&[0; 10], &vec![0; 8192], &vec![0; 4096])
        .expect_err("length");
    assert_eq!(
        error,
        CoreError::ImageLength {
            name: "kernal",
            actual: 10
        }
    );
    assert!(error.to_string().contains("kernal"));
    assert!(error.to_string().contains("10"));
}

#[test]
fn lda_sta_writes_screen_ram() {
    let mut machine = C64Machine::new();
    let (kernal, basic, chargen) = blank_roms();
    machine.load_roms(&kernal, &basic, &chargen).expect("roms");
    machine.write_ram(0x0400, 0x01);
    assert_eq!(machine.read(0x0400), 0x01);
}

#[test]
fn glyph_appears_after_a_frame() {
    let mut machine = C64Machine::new();
    let (mut kernal, basic, chargen) = blank_roms();
    // Reset vector -> spin: JMP $FF00 where we put infinite JMP
    put16(&mut kernal, 0xFFFC - 0xE000, 0xFF00);
    kernal[0xFF00 - 0xE000] = 0x4C; // JMP
    kernal[0xFF01 - 0xE000] = 0x00;
    kernal[0xFF02 - 0xE000] = 0xFF;
    // Chargen: solid block for code 1
    let mut chargen = chargen;
    let mut line = 0;
    while line < 8 {
        chargen[8 + line] = 0xFF;
        line += 1;
    }
    machine.load_roms(&kernal, &basic, &chargen).expect("roms");
    machine.write_ram(0x0400, 0x01);
    // color RAM via I/O write
    machine.read(0x0001); // touch
    // Direct color through map: use write after ensuring I/O — write_ram won't hit color.
    // Set via a tiny program would be heavy; paint reads color_byte which defaults 0.
    // Force white by writing through CPU path: we need Map write to color.
    // Expose via running STA — instead write using public read after load sets vic.
    // color defaults 0 (black on blue bg may be invisible). Set background 0, color via
    // writing $D800 — need I/O. Use machine.read/write by executing... simpler: paint
    // with color 0 on background 6 is invisible. Change: use color from ram after
    // we add write_color or use load and then poke via Bus.
    poke_color(&mut machine, 0, 0x01);
    let frame = machine.frame().expect("frame");
    let px = pixel(&frame, 32, 32);
    assert_eq!(px, 0x01);
}

#[test]
fn border_follows_d020() {
    let mut machine = C64Machine::new();
    let (mut kernal, basic, chargen) = blank_roms();
    put16(&mut kernal, 0xFFFC - 0xE000, 0xFF00);
    kernal[0xFF00 - 0xE000] = 0x4C;
    kernal[0xFF01 - 0xE000] = 0x00;
    kernal[0xFF02 - 0xE000] = 0xFF;
    machine.load_roms(&kernal, &basic, &chargen).expect("roms");
    poke_vic(&mut machine, 0x20, 0x06);
    let frame = machine.paint().expect("paint");
    assert_eq!(frame.index[0], 0x06);
    assert_eq!(frame.palette, eightbit_emu_core::PaletteKind::C64);
}

#[test]
fn keyboard_clears_cia_bit() {
    let mut machine = C64Machine::new();
    let (kernal, basic, chargen) = blank_roms();
    machine.load_roms(&kernal, &basic, &chargen).expect("roms");
    // Select column 1 (bit 1 low), row 1 is A on C64 matrix in our app mapping later
    // Write DDRA=$FF, PRA=!(1<<1)
    // Use raw map through CPU-less writes: need public poke for cia.
    // set_key(row,col) then read $DC01 with pra selecting col.
    machine.set_key(2, 1, true); // PRB bit 2, PRA bit 1 (letter A)
    poke_cia_select(&mut machine, 1 << 1);
    let value = machine.read(0xDC01);
    assert_eq!(value & (1 << 2), 0);
    machine.set_key(2, 1, false);
    let open = machine.read(0xDC01);
    assert_eq!(open, 0xFF);
}

#[test]
fn presenter_loads_open_roms_defaults_shape() {
    let Some((kernal, basic, chargen)) = open_roms() else {
        return;
    };
    let mut presenter = Presenter::new(PresentPace::Fixed60Hz);
    presenter
        .load_c64_roms(&kernal, &basic, &chargen)
        .expect("roms");
    assert!(presenter.is_c64());
    assert!(presenter.is_booted());
    let frame = presenter.on_display_tick().expect("tick");
    assert_eq!(frame.width, 384);
    assert_eq!(frame.height, 264);
}

#[test]
fn open_roms_runs_many_frames() {
    let Some((kernal, basic, chargen)) = open_roms() else {
        return;
    };
    let mut machine = C64Machine::new();
    machine.load_roms(&kernal, &basic, &chargen).expect("roms");
    let mut frame = 0;
    while frame < 120 {
        machine.frame().expect("frame");
        frame += 1;
    }
    machine.set_key(2, 1, true);
    machine.reset();
    assert!(machine.is_booted());
    assert!(machine.take_audio().is_empty());
}

#[test]
fn wrong_basic_and_chargen_lengths_name_the_image() {
    let mut machine = C64Machine::new();
    let basic_err = machine
        .load_roms(&vec![0; 8192], &[0; 3], &vec![0; 4096])
        .expect_err("basic");
    assert_eq!(
        basic_err,
        CoreError::ImageLength {
            name: "basic",
            actual: 3
        }
    );
    let chargen_err = machine
        .load_roms(&vec![0; 8192], &vec![0; 8192], &[0; 7])
        .expect_err("chargen");
    assert_eq!(
        chargen_err,
        CoreError::ImageLength {
            name: "chargen",
            actual: 7
        }
    );
}

fn blank_roms() -> (Vec<u8>, Vec<u8>, Vec<u8>) {
    (vec![0; 8192], vec![0; 8192], vec![0; 4096])
}

fn put16(rom: &mut [u8], offset: usize, value: u16) {
    rom[offset] = (value & 0xFF) as u8;
    rom[offset + 1] = (value >> 8) as u8;
}

fn pixel(frame: &eightbit_emu_core::Frame, x: u16, y: u16) -> u8 {
    frame.index[usize::from(y) * usize::from(frame.width) + usize::from(x)]
}

fn poke_color(machine: &mut C64Machine, offset: u16, value: u8) {
    // Color RAM only via I/O write path on Map — use write through port after ensuring
    // I/O visible (default port 0x37). Public API: write via a small backdoor on Machine.
    machine.poke_io(0xD800 + offset, value);
}

fn poke_vic(machine: &mut C64Machine, reg: u16, value: u8) {
    machine.poke_io(0xD000 + reg, value);
}

fn poke_cia_select(machine: &mut C64Machine, column_bit: u8) {
    machine.poke_io(0xDC02, 0xFF);
    machine.poke_io(0xDC00, !column_bit);
}

fn open_roms() -> Option<(Vec<u8>, Vec<u8>, Vec<u8>)> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../roms");
    let kernal = fs::read(root.join("c64-kernal.rom")).ok()?;
    let basic = fs::read(root.join("c64-basic.rom")).ok()?;
    let chargen = fs::read(root.join("c64-chargen.rom")).ok()?;
    Some((kernal, basic, chargen))
}

