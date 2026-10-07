use eightbit_emu_app::{
    format_config, load_from, parse_config, resolve_existing, save_to, AppConfig, MachineKind,
    OpenKind, NAME_BASIC, NAME_CHARGEN, NAME_KERNAL, NAME_SPECTRUM_128, NAME_SPECTRUM_48,
};
use std::path::{Path, PathBuf};

#[test]
fn defaults_need_no_path_fields() {
    let config = AppConfig::default();
    assert_eq!(config.machine, MachineKind::Spectrum);
    assert!(!config.model_128);
    assert!(config.rom_folder.is_none());
    assert!(config.spectrum_rom.is_none());
}

#[test]
fn parse_and_format_round_trip() {
    let mut config = AppConfig::default();
    config.set_c64();
    config.rom_folder = Some("/roms".to_string());
    config.kernal = Some("/roms/kernal.bin".to_string());
    let text = format_config(&config);
    let loaded = parse_config(&text);
    assert_eq!(loaded, config);
}

#[test]
fn missing_file_loads_defaults() {
    let path = PathBuf::from("/tmp/8bitemu-missing-config-does-not-exist.txt");
    assert_eq!(load_from(&path), AppConfig::default());
}

#[test]
fn save_and_load_file_round_trip() {
    let dir = temp_dir("save");
    let path = dir.join("config.txt");
    let mut config = AppConfig::default();
    config.set_spectrum_128();
    config.rom_folder = Some(dir.to_string_lossy().into_owned());
    save_to(&path, &config).expect("save");
    assert_eq!(load_from(&path), config);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn discovery_prefers_rom_folder_and_model_name() {
    let dir = temp_dir("discover");
    write_stub(&dir.join(NAME_SPECTRUM_48), 16);
    write_stub(&dir.join(NAME_SPECTRUM_128), 32);
    let folder = dir.to_string_lossy();
    let found_48 = resolve_existing(None, NAME_SPECTRUM_48, Some(&folder)).expect("48");
    let found_128 = resolve_existing(None, NAME_SPECTRUM_128, Some(&folder)).expect("128");
    assert!(found_48.ends_with(NAME_SPECTRUM_48));
    assert!(found_128.ends_with(NAME_SPECTRUM_128));
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn override_wins_over_folder_name() {
    let dir = temp_dir("override");
    write_stub(&dir.join(NAME_SPECTRUM_48), 16);
    let custom = dir.join("custom.rom");
    write_stub(&custom, 16);
    let folder = dir.to_string_lossy();
    let found = resolve_existing(Some(custom.to_str().unwrap()), NAME_SPECTRUM_48, Some(&folder))
        .expect("override");
    assert_eq!(found, custom);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn to_launch_resolves_c64_trio_from_folder() {
    let dir = temp_dir("c64");
    write_stub(&dir.join(NAME_KERNAL), 8);
    write_stub(&dir.join(NAME_BASIC), 8);
    write_stub(&dir.join(NAME_CHARGEN), 4);
    let mut config = AppConfig::default();
    config.set_c64();
    config.rom_folder = Some(dir.to_string_lossy().into_owned());
    let launch = config.to_launch().expect("launch");
    assert_eq!(launch.machine, MachineKind::C64);
    assert!(launch.kernal.unwrap().ends_with(NAME_KERNAL));
    assert!(launch.basic.unwrap().ends_with(NAME_BASIC));
    assert!(launch.chargen.unwrap().ends_with(NAME_CHARGEN));
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn to_launch_picks_128_rom_name() {
    let dir = temp_dir("128");
    write_stub(&dir.join(NAME_SPECTRUM_48), 16);
    write_stub(&dir.join(NAME_SPECTRUM_128), 32);
    let mut config = AppConfig::default();
    config.set_spectrum_128();
    config.rom_folder = Some(dir.to_string_lossy().into_owned());
    let launch = config.to_launch().expect("launch");
    assert!(launch.rom.unwrap().ends_with(NAME_SPECTRUM_128));
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn open_kind_selects_the_expected_machine() {
    let mut config = AppConfig::default();
    config.apply_open_kind(OpenKind::Prg, 10);
    assert!(config.is_c64());
    config.apply_open_kind(OpenKind::Rom, 32 * 1024);
    assert!(config.is_spectrum_128());
    config.apply_open_kind(OpenKind::Tape, 0);
    assert!(config.is_spectrum_128());
    config.apply_open_kind(OpenKind::Sna, 49_179);
    assert!(config.is_spectrum_48());
}

fn temp_dir(label: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("8bitemu-{label}-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("dir");
    dir
}

fn write_stub(path: &Path, kib: usize) {
    std::fs::write(path, vec![0u8; kib * 1024]).expect("write");
}
