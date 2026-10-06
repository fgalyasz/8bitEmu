use eightbit_emu_core::C64Machine;
use std::fs;
use std::path::PathBuf;

#[test]
fn raster_line_moves_between_reads() {
    let mut machine = blank_machine();
    let first = machine.read(0xD012);
    let mut steps = 0u32;
    while steps < 500 {
        machine.step_instruction().expect("step");
        if machine.read(0xD012) != first {
            return;
        }
        steps += 1;
    }
    panic!("raster stayed at {first}");
}

#[test]
fn timer_a_underflow_sets_icr() {
    let mut machine = blank_machine();
    machine.poke_io(0xDC0D, 0x7F);
    machine.poke_io(0xDC0D, 0x81);
    machine.poke_io(0xDC04, 0x10);
    machine.poke_io(0xDC05, 0x00);
    machine.poke_io(0xDC0E, 0x01);
    let mut frames = 0;
    while frames < 20 {
        machine.advance().expect("frame");
        let icr = machine.read(0xDC0D);
        if icr & 0x01 != 0 {
            let cleared = machine.read(0xDC0D);
            assert_eq!(cleared & 0x01, 0);
            return;
        }
        frames += 1;
    }
    panic!("timer A never underflowed");
}

#[test]
fn open_roms_prints_boot_text() {
    let Some((kernal, basic, chargen)) = open_roms() else {
        return;
    };
    let mut machine = C64Machine::new();
    machine.load_roms(&kernal, &basic, &chargen).expect("roms");
    let mut frame = 0;
    while frame < 400 {
        machine.frame().expect("frame");
        if screen_has_text(&mut machine) {
            let painted = machine.paint().expect("paint");
            assert!(frame_has_ink(&painted), "banner should paint ink");
            return;
        }
        frame += 1;
    }
    panic!("no boot text in screen RAM");
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

fn screen_has_text(machine: &mut C64Machine) -> bool {
    let mut address = 0x0400u16;
    while address < 0x07E8 {
        let code = machine.read(address);
        if code != 0x20 && code != 0x00 {
            return true;
        }
        address += 1;
    }
    false
}

fn frame_has_ink(frame: &eightbit_emu_core::Frame) -> bool {
    let bg = frame.index[usize::from(frame.width) * 40 + 40];
    let mut y = 32u16;
    while y < 32 + 16 {
        let mut x = 32u16;
        while x < 32 + 80 {
            let color = frame.index[usize::from(y) * usize::from(frame.width) + usize::from(x)];
            if color != bg {
                return true;
            }
            x += 1;
        }
        y += 1;
    }
    false
}

fn open_roms() -> Option<(Vec<u8>, Vec<u8>, Vec<u8>)> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../roms");
    let kernal = fs::read(root.join("c64-kernal.rom")).ok()?;
    let basic = fs::read(root.join("c64-basic.rom")).ok()?;
    let chargen = fs::read(root.join("c64-chargen.rom")).ok()?;
    Some((kernal, basic, chargen))
}


#[test]
fn open_roms_boot_contains_ready_screen_codes() {
    let Some((kernal, basic, chargen)) = open_roms() else { return; };
    let mut machine = C64Machine::new();
    machine.load_roms(&kernal, &basic, &chargen).expect("roms");
    let mut frame = 0;
    while frame < 200 {
        machine.frame().expect("frame");
        frame += 1;
    }
    let ready = [18u8, 5, 1, 4, 25, 46];
    let mut address = 0x0400u16;
    while address + 6 <= 0x07E8 {
        let mut i = 0u16;
        let mut ok = true;
        while i < 6 {
            if machine.read(address + i) != ready[usize::from(i)] {
                ok = false;
                break;
            }
            i += 1;
        }
        if ok {
            return;
        }
        address += 1;
    }
    panic!("READY. screen codes not found");
}
