use crate::launch::{Launch, MachineKind};
use std::fs;
use std::path::{Path, PathBuf};

const DEFAULT_SPECTRUM_ROM: &str = "roms/spectrum-48.rom";
const DEFAULT_KERNAL: &str = "roms/c64-kernal.rom";
const DEFAULT_BASIC: &str = "roms/c64-basic.rom";
const DEFAULT_CHARGEN: &str = "roms/c64-chargen.rom";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AppConfig {
    pub machine: MachineKind,
    pub model_128: bool,
    pub spectrum_rom: String,
    pub kernal: String,
    pub basic: String,
    pub chargen: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            machine: MachineKind::Spectrum,
            model_128: false,
            spectrum_rom: DEFAULT_SPECTRUM_ROM.to_string(),
            kernal: DEFAULT_KERNAL.to_string(),
            basic: DEFAULT_BASIC.to_string(),
            chargen: DEFAULT_CHARGEN.to_string(),
        }
    }
}

impl AppConfig {
    pub fn load() -> Self {
        load_from(&config_path())
    }

    pub fn save(&self) -> Result<(), String> {
        save_to(&config_path(), self)
    }

    pub fn set_spectrum_48(&mut self) {
        self.machine = MachineKind::Spectrum;
        self.model_128 = false;
    }

    pub fn set_spectrum_128(&mut self) {
        self.machine = MachineKind::Spectrum;
        self.model_128 = true;
    }

    pub fn set_c64(&mut self) {
        self.machine = MachineKind::C64;
    }

    pub fn is_spectrum_48(&self) -> bool {
        self.machine == MachineKind::Spectrum && !self.model_128
    }

    pub fn is_spectrum_128(&self) -> bool {
        self.machine == MachineKind::Spectrum && self.model_128
    }

    pub fn is_c64(&self) -> bool {
        self.machine == MachineKind::C64
    }

    pub fn to_launch(&self) -> Launch {
        Launch {
            machine: self.machine,
            model_128: self.model_128,
            rom: spectrum_rom_slot(self),
            sna: None,
            tap: None,
            tzx: None,
            prg: None,
            kernal: c64_path(self, &self.kernal),
            basic: c64_path(self, &self.basic),
            chargen: c64_path(self, &self.chargen),
        }
    }
}

fn spectrum_rom_slot(config: &AppConfig) -> Option<String> {
    if config.machine == MachineKind::Spectrum {
        Some(config.spectrum_rom.clone())
    } else {
        None
    }
}

fn c64_path(config: &AppConfig, path: &str) -> Option<String> {
    if config.machine == MachineKind::C64 {
        Some(path.to_string())
    } else {
        None
    }
}

pub fn load_from(path: &Path) -> AppConfig {
    let Ok(text) = fs::read_to_string(path) else {
        return AppConfig::default();
    };
    parse_config(&text)
}

pub fn save_to(path: &Path, config: &AppConfig) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| format!("{}: {error}", parent.display()))?;
    }
    fs::write(path, format_config(config)).map_err(|error| format!("{}: {error}", path.display()))
}

pub fn parse_config(text: &str) -> AppConfig {
    let mut config = AppConfig::default();
    for line in text.lines() {
        apply_line(&mut config, line);
    }
    config
}

pub fn format_config(config: &AppConfig) -> String {
    format!(
        "machine={}\nmodel={}\nspectrum_rom={}\nkernal={}\nbasic={}\nchargen={}\n",
        machine_name(config.machine),
        model_name(config.model_128),
        config.spectrum_rom,
        config.kernal,
        config.basic,
        config.chargen,
    )
}

pub fn config_path() -> PathBuf {
    config_dir().join("config.txt")
}

fn config_dir() -> PathBuf {
    if let Some(path) = std::env::var_os("EIGHTBIT_EMU_CONFIG_DIR") {
        return PathBuf::from(path);
    }
    platform_config_dir()
}

fn platform_config_dir() -> PathBuf {
    #[cfg(target_os = "macos")]
    {
        if let Some(home) = std::env::var_os("HOME") {
            return PathBuf::from(home)
                .join("Library")
                .join("Application Support")
                .join("8bitEmu");
        }
    }
    #[cfg(target_os = "windows")]
    {
        if let Some(base) = std::env::var_os("APPDATA") {
            return PathBuf::from(base).join("8bitEmu");
        }
    }
    if let Some(xdg) = std::env::var_os("XDG_CONFIG_HOME") {
        return PathBuf::from(xdg).join("8bitEmu");
    }
    if let Some(home) = std::env::var_os("HOME") {
        return PathBuf::from(home).join(".config").join("8bitEmu");
    }
    PathBuf::from("8bitEmu")
}

fn apply_line(config: &mut AppConfig, line: &str) {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with('#') {
        return;
    }
    let Some((key, value)) = trimmed.split_once('=') else {
        return;
    };
    match key.trim() {
        "machine" => apply_machine(config, value.trim()),
        "model" => apply_model(config, value.trim()),
        "spectrum_rom" => config.spectrum_rom = value.trim().to_string(),
        "kernal" => config.kernal = value.trim().to_string(),
        "basic" => config.basic = value.trim().to_string(),
        "chargen" => config.chargen = value.trim().to_string(),
        _ => {}
    }
}

fn apply_machine(config: &mut AppConfig, value: &str) {
    match value {
        "spectrum" => config.machine = MachineKind::Spectrum,
        "c64" => config.machine = MachineKind::C64,
        _ => {}
    }
}

fn apply_model(config: &mut AppConfig, value: &str) {
    match value {
        "48" => config.model_128 = false,
        "128" => config.model_128 = true,
        _ => {}
    }
}

fn machine_name(machine: MachineKind) -> &'static str {
    match machine {
        MachineKind::Spectrum => "spectrum",
        MachineKind::C64 => "c64",
    }
}

fn model_name(model_128: bool) -> &'static str {
    if model_128 { "128" } else { "48" }
}
