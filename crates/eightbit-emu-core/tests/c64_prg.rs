use eightbit_emu_core::{C64Machine, CoreError};

#[test]
fn short_prg_is_rejected() {
    let mut machine = blank_machine();
    let error = machine.load_prg(&[0x01, 0x08]).expect_err("short");
    assert_eq!(
        error,
        CoreError::ImageLength {
            name: "prg",
            actual: 2
        }
    );
}

#[test]
fn overflow_prg_is_rejected() {
    let mut machine = blank_machine();
    let mut bytes = vec![0xFF, 0xFF];
    bytes.extend_from_slice(&[0x00, 0x01]);
    let error = machine.load_prg(&bytes).expect_err("overflow");
    assert_eq!(
        error,
        CoreError::Unsupported {
            kind: "prg",
            id: 1
        }
    );
}

#[test]
fn basic_prg_writes_ram_and_links() {
    let mut machine = blank_machine();
    let bytes = [0x01, 0x08, 0x0A, 0x00, 0x99, 0x00, 0x00, 0x00];
    machine.load_prg(&bytes).expect("prg");
    assert_eq!(machine.read(0x0801), 0x0A);
    assert_eq!(machine.read(0x0802), 0x00);
    assert_eq!(word(&mut machine, 0x002D), 0x0807);
    assert_eq!(word(&mut machine, 0x002F), 0x0807);
    assert_eq!(word(&mut machine, 0x0031), 0x0807);
    assert_eq!(machine.read(0x00C6), 4);
    assert_eq!(machine.read(0x0277), b'R');
    assert_eq!(machine.read(0x027A), 0x0D);
}

#[test]
fn machine_code_prg_sets_pc_and_runs() {
    let mut machine = blank_machine();
    let bytes = [
        0x00, 0xC0, 0xA9, 0x01, 0x8D, 0x00, 0x04, 0x4C, 0x00, 0xC0,
    ];
    machine.load_prg(&bytes).expect("prg");
    assert_eq!(machine.pc(), 0xC000);
    let mut steps = 0u32;
    while steps < 20 {
        machine.step_instruction().expect("step");
        if machine.read(0x0400) == 0x01 {
            return;
        }
        steps += 1;
    }
    panic!("store never ran");
}

fn word(machine: &mut C64Machine, address: u16) -> u16 {
    let low = machine.read(address);
    let high = machine.read(address + 1);
    u16::from_le_bytes([low, high])
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
