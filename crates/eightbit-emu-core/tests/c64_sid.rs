use eightbit_emu_core::C64Machine;

#[test]
fn sid_registers_round_trip() {
    let mut machine = blank_machine();
    machine.poke_io(0xD400, 0x34);
    machine.poke_io(0xD418, 0x0F);
    assert_eq!(machine.read(0xD400), 0x34);
    assert_eq!(machine.read(0xD418), 0x0F);
    assert_eq!(machine.read(0xD420), 0x34);
}

#[test]
fn gated_triangle_makes_audio() {
    let mut machine = blank_machine();
    arm_tone(&mut machine, 0x0F);
    let mut peak = 0.0f32;
    let mut frames = 0;
    while frames < 5 {
        machine.advance().expect("frame");
        let audio = machine.take_audio();
        peak = peak.max(abs_peak(&audio));
        frames += 1;
    }
    assert!(peak > 0.05, "peak {peak}");
}

#[test]
fn zero_volume_is_silent() {
    let mut machine = blank_machine();
    arm_tone(&mut machine, 0x00);
    machine.advance().expect("frame");
    let audio = machine.take_audio();
    assert!(abs_peak(&audio) < 0.001);
}

fn arm_tone(machine: &mut C64Machine, volume: u8) {
    machine.poke_io(0xD400, 0x00);
    machine.poke_io(0xD401, 0x20);
    machine.poke_io(0xD405, 0x00);
    machine.poke_io(0xD406, 0xF0);
    machine.poke_io(0xD418, volume);
    machine.poke_io(0xD404, 0x11);
}

fn abs_peak(audio: &[f32]) -> f32 {
    let mut peak = 0.0f32;
    let mut index = 0;
    while index < audio.len() {
        peak = peak.max(audio[index].abs());
        index += 1;
    }
    peak
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
