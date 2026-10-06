use eightbit_emu_app::{c64_key, kempston_bit, parse_launch, quits, spectrum_key, MachineKind};
use winit::keyboard::KeyCode;

#[test]
fn flags_select_the_model_and_the_image_paths() {
    let launch = parse_launch(&args(&[
        "--model",
        "128",
        "--rom",
        "basic.rom",
        "--sna",
        "game.sna",
        "--tap",
        "game.tap",
    ]))
    .expect("launch");
    assert!(launch.model_128);
    assert_eq!(launch.rom.as_deref(), Some("basic.rom"));
    assert_eq!(launch.sna.as_deref(), Some("game.sna"));
    assert_eq!(launch.tap.as_deref(), Some("game.tap"));
    let narrow = parse_launch(&args(&["--model", "48"])).expect("48");
    assert!(!narrow.model_128);
}

#[test]
fn c64_machine_defaults_rom_paths() {
    let launch = parse_launch(&args(&["--machine", "c64"])).expect("c64");
    assert_eq!(launch.machine, MachineKind::C64);
    assert_eq!(launch.kernal.as_deref(), Some("roms/c64-kernal.rom"));
    assert_eq!(launch.basic.as_deref(), Some("roms/c64-basic.rom"));
    assert_eq!(launch.chargen.as_deref(), Some("roms/c64-chargen.rom"));
    assert_eq!(c64_key(KeyCode::KeyA), Some((1, 2)));
}

#[test]
fn unknown_arguments_name_themselves() {
    let error = parse_launch(&args(&["--tape"])).expect_err("unknown");
    assert!(error.contains("--tape"));
    let missing = parse_launch(&args(&["--rom"])).expect_err("missing");
    assert!(missing.contains("--rom"));
    let model = parse_launch(&args(&["--model", "plus3"])).expect_err("model");
    assert!(model.contains("plus3"));
}

#[test]
fn digit_keys_reach_the_spectrum_matrix() {
    assert_eq!(spectrum_key(KeyCode::Digit1), Some((3, 0x01)));
    assert_eq!(spectrum_key(KeyCode::ShiftLeft), Some((0, 0x01)));
    assert_eq!(spectrum_key(KeyCode::ControlLeft), Some((7, 0x02)));
    assert_eq!(spectrum_key(KeyCode::F1), None);
    assert_eq!(kempston_bit(KeyCode::ArrowRight), Some(0x01));
    assert_eq!(kempston_bit(KeyCode::AltRight), Some(0x10));
    assert!(!quits(KeyCode::Escape));
}

fn args(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_string()).collect()
}
