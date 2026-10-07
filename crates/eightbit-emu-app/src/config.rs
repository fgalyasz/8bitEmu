use crate::launch::{Launch, MachineKind};
use crate::open_file::OpenKind;
use std::fs;
use std::path::{Path, PathBuf};

pub const NAME_SPECTRUM_48: &str = "spectrum-48.rom";
pub const NAME_SPECTRUM_128: &str = "spectrum-128.rom";
pub const NAME_KERNAL: &str = "c64-kernal.rom";
pub const NAME_BASIC: &str = "c64-basic.rom";
pub const NAME_CHARGEN: &str = "c64-chargen.rom";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AppConfig {
    pub machine: MachineKind,
    pub model_128: bool,
    pub rom_folder: Option<String>,
    pub spectrum_rom: Option<String>,
    pub kernal: Option<String>,
    pub basic: Option<String>,
    pub chargen: Option<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            machine: MachineKind::Spectrum,
            model_128: false,
            rom_folder: None,
            spectrum_rom: None,
            kernal: None,
            basic: None,
            chargen: None,
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

    pub fn apply_open_kind(&mut self, kind: OpenKind, len: usize) {
        match kind {
            OpenKind::Prg => self.set_c64(),
            OpenKind::Rom => {
                self.set_spectrum_from_rom_len(len);
            }
            OpenKind::Sna => {
                self.set_spectrum_from_sna_len(len);
            }
            OpenKind::Tape => {
                if self.machine != MachineKind::Spectrum {
                    self.set_spectrum_48();
                }
            }
        }
    }

    pub fn to_launch(&self) -> Result<Launch, String> {
        match self.machine {
            MachineKind::Spectrum => spectrum_launch(self),
            MachineKind::C64 => c64_launch(self),
        }
    }

    fn set_spectrum_from_rom_len(&mut self, len: usize) {
        if len == 32 * 1024 {
            self.set_spectrum_128();
        } else {
            self.set_spectrum_48();
        }
    }

    fn set_spectrum_from_sna_len(&mut self, len: usize) {
        if len == 131_103 {
            self.set_spectrum_128();
        } else {
            self.set_spectrum_48();
        }
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
    let mut lines = vec![
        format!("machine={}", machine_name(config.machine)),
        format!("model={}", model_name(config.model_128)),
    ];
    push_opt(&mut lines, "rom_folder", config.rom_folder.as_deref());
    push_opt(&mut lines, "spectrum_rom", config.spectrum_rom.as_deref());
    push_opt(&mut lines, "kernal", config.kernal.as_deref());
    push_opt(&mut lines, "basic", config.basic.as_deref());
    push_opt(&mut lines, "chargen", config.chargen.as_deref());
    lines.push(String::new());
    lines.join("\n")
}

pub fn config_path() -> PathBuf {
    config_dir().join("config.txt")
}

pub fn discover_rom(name: &str, rom_folder: Option<&str>) -> Option<PathBuf> {
    for root in rom_roots(rom_folder) {
        let path = root.join(name);
        if path.is_file() {
            return Some(path);
        }
    }
    None
}

pub fn resolve_existing(override_path: Option<&str>, name: &str, folder: Option<&str>) -> Option<PathBuf> {
    if let Some(path) = override_path {
        let candidate = PathBuf::from(path);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    discover_rom(name, folder)
}

fn spectrum_launch(config: &AppConfig) -> Result<Launch, String> {
    let name = spectrum_rom_name(config.model_128);
    let rom = resolve_existing(
        config.spectrum_rom.as_deref(),
        name,
        config.rom_folder.as_deref(),
    )
    .ok_or_else(|| missing_rom(name))?;
    Ok(Launch {
        machine: MachineKind::Spectrum,
        model_128: config.model_128,
        rom: Some(rom.to_string_lossy().into_owned()),
        sna: None,
        tap: None,
        tzx: None,
        prg: None,
        kernal: None,
        basic: None,
        chargen: None,
    })
}

fn c64_launch(config: &AppConfig) -> Result<Launch, String> {
    let folder = config.rom_folder.as_deref();
    let kernal = resolve_existing(config.kernal.as_deref(), NAME_KERNAL, folder)
        .ok_or_else(|| missing_rom(NAME_KERNAL))?;
    let basic = resolve_existing(config.basic.as_deref(), NAME_BASIC, folder)
        .ok_or_else(|| missing_rom(NAME_BASIC))?;
    let chargen = resolve_existing(config.chargen.as_deref(), NAME_CHARGEN, folder)
        .ok_or_else(|| missing_rom(NAME_CHARGEN))?;
    Ok(Launch {
        machine: MachineKind::C64,
        model_128: false,
        rom: None,
        sna: None,
        tap: None,
        tzx: None,
        prg: None,
        kernal: Some(kernal.to_string_lossy().into_owned()),
        basic: Some(basic.to_string_lossy().into_owned()),
        chargen: Some(chargen.to_string_lossy().into_owned()),
    })
}

fn spectrum_rom_name(model_128: bool) -> &'static str {
    if model_128 {
        NAME_SPECTRUM_128
    } else {
        NAME_SPECTRUM_48
    }
}

fn missing_rom(name: &str) -> String {
    format!("missing ROM {name} (set Settings → ROM Folder… or place files in roms/)")
}

fn rom_roots(rom_folder: Option<&str>) -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if let Some(folder) = rom_folder {
        roots.push(PathBuf::from(folder));
    }
    roots.push(PathBuf::from("roms"));
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            roots.push(dir.join("roms"));
        }
    }
    roots.push(config_dir().join("roms"));
    roots
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
    let value = value.trim();
    if value.is_empty() {
        return;
    }
    match key.trim() {
        "machine" => apply_machine(config, value),
        "model" => apply_model(config, value),
        "rom_folder" => config.rom_folder = Some(value.to_string()),
        "spectrum_rom" => config.spectrum_rom = Some(value.to_string()),
        "kernal" => config.kernal = Some(value.to_string()),
        "basic" => config.basic = Some(value.to_string()),
        "chargen" => config.chargen = Some(value.to_string()),
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

fn push_opt(lines: &mut Vec<String>, key: &str, value: Option<&str>) {
    if let Some(value) = value {
        lines.push(format!("{key}={value}"));
    }
}
