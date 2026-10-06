use eightbit_emu_core::C64Machine;
use std::fs;
use std::path::PathBuf;

#[test]
fn key_a_types_letter_a_on_screen() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../roms");
    let mut machine = C64Machine::new();
    machine
        .load_roms(
            &fs::read(root.join("c64-kernal.rom")).unwrap(),
            &fs::read(root.join("c64-basic.rom")).unwrap(),
            &fs::read(root.join("c64-chargen.rom")).unwrap(),
        )
        .unwrap();
    for _ in 0..200 {
        machine.frame().unwrap();
    }
    let mut before = Vec::new();
    for address in 0x0400u16..0x07E8 {
        before.push(machine.read(address));
    }
    machine.set_key(2, 1, true);
    for _ in 0..40 {
        machine.frame().unwrap();
    }
    machine.set_key(2, 1, false);
    for _ in 0..20 {
        machine.frame().unwrap();
    }
    let mut found = false;
    for address in 0x0400u16..0x07E8 {
        let now = machine.read(address);
        let was = before[usize::from(address - 0x0400)];
        if now != was && now == 1 {
            found = true;
            break;
        }
    }
    assert!(found, "pressing A should write screen code 1");
}
