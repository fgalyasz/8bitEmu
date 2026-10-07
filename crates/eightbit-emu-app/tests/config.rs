use eightbit_emu_app::{
    format_config, load_from, parse_config, save_to, AppConfig, MachineKind,
};
use std::path::PathBuf;

#[test]
fn defaults_match_repo_rom_paths() {
    let config = AppConfig::default();
    assert_eq!(config.machine, MachineKind::Spectrum);
    assert!(!config.model_128);
    assert_eq!(config.spectrum_rom, "roms/spectrum-48.rom");
    assert_eq!(config.kernal, "roms/c64-kernal.rom");
    assert_eq!(config.basic, "roms/c64-basic.rom");
    assert_eq!(config.chargen, "roms/c64-chargen.rom");
}

#[test]
fn parse_and_format_round_trip() {
    let mut config = AppConfig::default();
    config.set_c64();
    config.model_128 = true;
    config.spectrum_rom = "/roms/spectrum-128.rom".to_string();
    config.kernal = "/roms/kernal.bin".to_string();
    config.basic = "/roms/basic.bin".to_string();
    config.chargen = "/roms/chargen.bin".to_string();
    let text = format_config(&config);
    let loaded = parse_config(&text);
    assert_eq!(loaded, config);
}

#[test]
fn missing_file_loads_defaults() {
    let path = PathBuf::from("/tmp/8bitemu-missing-config-does-not-exist.txt");
    let config = load_from(&path);
    assert_eq!(config, AppConfig::default());
}

#[test]
fn save_and_load_file_round_trip() {
    let dir = std::env::temp_dir().join(format!("8bitemu-config-{}", std::process::id()));
    let path = dir.join("config.txt");
    let mut config = AppConfig::default();
    config.set_spectrum_128();
    config.spectrum_rom = "roms/spectrum-128.rom".to_string();
    save_to(&path, &config).expect("save");
    let loaded = load_from(&path);
    assert_eq!(loaded, config);
    let _ = std::fs::remove_dir_all(dir);
}
