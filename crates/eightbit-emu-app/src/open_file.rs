use crate::config::AppConfig;
use muda::accelerator::{Accelerator, Code, Modifiers, CMD_OR_CTRL};
use muda::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem, Submenu};
use rfd::FileDialog;
use std::path::PathBuf;
use winit::keyboard::{KeyCode, ModifiersState};
use winit::window::Window;

pub const OPEN_ID: &str = "open";
pub const SAVE_SNA: &str = "save-sna";
pub const SAVE_TAP: &str = "save-tap";
pub const SAVE_TZX: &str = "save-tzx";
pub const SIZE_FIT: &str = "size-fit";
pub const LOADING_SOUND: &str = "loading-sound";
pub const MACHINE_48: &str = "machine-48";
pub const MACHINE_128: &str = "machine-128";
pub const MACHINE_C64: &str = "machine-c64";
pub const MACHINE_START: &str = "machine-start";
pub const SET_SPECTRUM_ROM: &str = "set-spectrum-rom";
pub const SET_KERNAL: &str = "set-kernal";
pub const SET_BASIC: &str = "set-basic";
pub const SET_CHARGEN: &str = "set-chargen";

pub struct MenuBar {
    pub menu: Menu,
    pub loading_sound: CheckMenuItem,
    pub machine_48: CheckMenuItem,
    pub machine_128: CheckMenuItem,
    pub machine_c64: CheckMenuItem,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SettingsPath {
    SpectrumRom,
    Kernal,
    Basic,
    Chargen,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PictureSize {
    Fit,
    Percent(u32),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SaveKind {
    Sna,
    Tap,
    Tzx,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OpenKind {
    Rom,
    Sna,
    Tape,
    Prg,
}

pub fn open_kind(path: &str) -> Option<OpenKind> {
    let ext = extension(path)?;
    kind_for(ext)
}

pub fn rom_model_128(len: usize, current: bool) -> bool {
    len == 32 * 1024 || current
}

pub fn sna_model_128(len: usize, current: bool) -> bool {
    if len == 131_103 {
        return true;
    }
    if len == 49_179 {
        return false;
    }
    current
}

pub fn command_open(state: ModifiersState, code: KeyCode) -> bool {
    code == KeyCode::KeyO && command_held(state)
}

pub fn command_start(state: ModifiersState, code: KeyCode) -> bool {
    code == KeyCode::KeyR && command_held(state)
}

pub fn picture_size(id: &str) -> Option<PictureSize> {
    match id {
        SIZE_FIT => Some(PictureSize::Fit),
        "size-125" => Some(PictureSize::Percent(125)),
        "size-150" => Some(PictureSize::Percent(150)),
        "size-175" => Some(PictureSize::Percent(175)),
        "size-200" => Some(PictureSize::Percent(200)),
        _ => None,
    }
}

pub fn save_kind(id: &str) -> Option<SaveKind> {
    match id {
        SAVE_SNA => Some(SaveKind::Sna),
        SAVE_TAP => Some(SaveKind::Tap),
        SAVE_TZX => Some(SaveKind::Tzx),
        _ => None,
    }
}

pub fn save_shortcut(command: bool, shift: bool, letter: char) -> Option<SaveKind> {
    if !command {
        return None;
    }
    shortcut_letter(shift, letter)
}

pub fn ensure_extension(path: &str, ext: &str) -> String {
    if extension(path).is_some_and(|found| found.eq_ignore_ascii_case(ext)) {
        return path.to_string();
    }
    format!("{path}.{ext}")
}

pub fn command_save(state: ModifiersState, code: KeyCode) -> Option<SaveKind> {
    let letter = save_letter(code)?;
    save_shortcut(command_held(state), state.shift_key(), letter)
}

pub fn pick_save(window: Option<&Window>, kind: SaveKind) -> Option<PathBuf> {
    let dialog = FileDialog::new()
        .set_title(kind.label())
        .set_file_name(kind.file_name())
        .add_filter(kind.label(), &[kind.extension()]);
    parented(dialog, window).save_file()
}

pub fn install_menu(window: &Window, config: &AppConfig) -> Option<MenuBar> {
    let bar = build_menu(config).ok()?;
    show_menu(window, &bar.menu);
    Some(bar)
}

pub fn pick_path(window: Option<&Window>) -> Option<PathBuf> {
    let dialog = FileDialog::new()
        .set_title("Open")
        .add_filter("Spectrum", &["rom", "sna", "tap", "tzx"])
        .add_filter("Commodore 64", &["prg"]);
    parented(dialog, window).pick_file()
}

pub fn pick_rom_path(window: Option<&Window>, kind: SettingsPath) -> Option<PathBuf> {
    let dialog = FileDialog::new()
        .set_title(kind.label())
        .add_filter("ROM", &["rom", "bin"]);
    parented(dialog, window).pick_file()
}

pub fn settings_path(id: &str) -> Option<SettingsPath> {
    match id {
        SET_SPECTRUM_ROM => Some(SettingsPath::SpectrumRom),
        SET_KERNAL => Some(SettingsPath::Kernal),
        SET_BASIC => Some(SettingsPath::Basic),
        SET_CHARGEN => Some(SettingsPath::Chargen),
        _ => None,
    }
}

pub fn machine_choice(id: &str) -> Option<MachineChoice> {
    match id {
        MACHINE_48 => Some(MachineChoice::Spectrum48),
        MACHINE_128 => Some(MachineChoice::Spectrum128),
        MACHINE_C64 => Some(MachineChoice::C64),
        _ => None,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MachineChoice {
    Spectrum48,
    Spectrum128,
    C64,
}

fn kind_for(ext: &str) -> Option<OpenKind> {
    if ext.eq_ignore_ascii_case("rom") {
        return Some(OpenKind::Rom);
    }
    if ext.eq_ignore_ascii_case("sna") {
        return Some(OpenKind::Sna);
    }
    if ext.eq_ignore_ascii_case("tap") || ext.eq_ignore_ascii_case("tzx") {
        return Some(OpenKind::Tape);
    }
    if ext.eq_ignore_ascii_case("prg") {
        return Some(OpenKind::Prg);
    }
    None
}

fn extension(path: &str) -> Option<&str> {
    let name = path.rsplit(['/', '\\']).next().unwrap_or(path);
    let (_, ext) = name.rsplit_once('.')?;
    if ext.is_empty() { None } else { Some(ext) }
}

fn command_held(state: ModifiersState) -> bool {
    #[cfg(target_os = "macos")]
    {
        return state.super_key();
    }
    #[cfg(not(target_os = "macos"))]
    {
        return state.control_key();
    }
}

fn build_menu(config: &AppConfig) -> muda::Result<MenuBar> {
    let file = file_menu()?;
    let (machine, machine_48, machine_128, machine_c64) = machine_menu(config)?;
    let settings = settings_menu()?;
    let view = view_menu()?;
    let (sound, loading_sound) = sound_menu()?;
    let menu = menu_bar(&file, &machine, &settings, &view, &sound)?;
    Ok(MenuBar {
        menu,
        loading_sound,
        machine_48,
        machine_128,
        machine_c64,
    })
}

fn machine_menu(
    config: &AppConfig,
) -> muda::Result<(Submenu, CheckMenuItem, CheckMenuItem, CheckMenuItem)> {
    let machine_48 = CheckMenuItem::with_id(MACHINE_48, "Spectrum 48K", true, config.is_spectrum_48(), None);
    let machine_128 =
        CheckMenuItem::with_id(MACHINE_128, "Spectrum 128K", true, config.is_spectrum_128(), None);
    let machine_c64 = CheckMenuItem::with_id(MACHINE_C64, "Commodore 64", true, config.is_c64(), None);
    let start = MenuItem::with_id(MACHINE_START, "Start", true, Some(start_accelerator()));
    let separator = PredefinedMenuItem::separator();
    let menu = Submenu::with_items(
        "Machine",
        true,
        &[&machine_48, &machine_128, &machine_c64, &separator, &start],
    )?;
    Ok((menu, machine_48, machine_128, machine_c64))
}

fn settings_menu() -> muda::Result<Submenu> {
    let spectrum = MenuItem::with_id(SET_SPECTRUM_ROM, "Spectrum ROM…", true, None);
    let kernal = MenuItem::with_id(SET_KERNAL, "C64 Kernal…", true, None);
    let basic = MenuItem::with_id(SET_BASIC, "C64 BASIC…", true, None);
    let chargen = MenuItem::with_id(SET_CHARGEN, "C64 Chargen…", true, None);
    Submenu::with_items("Settings", true, &[&spectrum, &kernal, &basic, &chargen])
}

fn start_accelerator() -> Accelerator {
    Accelerator::new(CMD_OR_CTRL, Code::KeyR)
}

fn file_menu() -> muda::Result<Submenu> {
    let open = open_item();
    let separator = PredefinedMenuItem::separator();
    let sna = save_item(SaveKind::Sna);
    let tap = save_item(SaveKind::Tap);
    let tzx = save_item(SaveKind::Tzx);
    Submenu::with_items("File", true, &[&open, &separator, &sna, &tap, &tzx])
}

fn view_menu() -> muda::Result<Submenu> {
    let fit = MenuItem::with_id(SIZE_FIT, "Fit", true, None);
    let separator = PredefinedMenuItem::separator();
    let p125 = percent_item(125, Code::F5);
    let p150 = percent_item(150, Code::F6);
    let p175 = percent_item(175, Code::F7);
    let p200 = percent_item(200, Code::F8);
    Submenu::with_items("View", true, &[&fit, &separator, &p125, &p150, &p175, &p200])
}

fn save_item(kind: SaveKind) -> MenuItem {
    MenuItem::with_id(kind.id(), kind.label(), true, Some(kind.accelerator()))
}

fn percent_item(percent: u32, code: Code) -> MenuItem {
    let id = format!("size-{percent}");
    let label = format!("{percent}%");
    let shortcut = Accelerator::new(Modifiers::empty(), code);
    MenuItem::with_id(id, label, true, Some(shortcut))
}

fn open_item() -> MenuItem {
    MenuItem::with_id(OPEN_ID, "Open…", true, Some(open_accelerator()))
}

fn open_accelerator() -> Accelerator {
    Accelerator::new(CMD_OR_CTRL, Code::KeyO)
}

fn sound_menu() -> muda::Result<(Submenu, CheckMenuItem)> {
    let loading_sound = loading_sound_item();
    let sound = Submenu::with_items("Sound", true, &[&loading_sound])?;
    Ok((sound, loading_sound))
}

fn loading_sound_item() -> CheckMenuItem {
    CheckMenuItem::with_id(LOADING_SOUND, "Loading sound", true, false, Some(sound_key()))
}

fn sound_key() -> Accelerator {
    Accelerator::new(Modifiers::empty(), Code::F4)
}

fn menu_bar(
    file: &Submenu,
    machine: &Submenu,
    settings: &Submenu,
    view: &Submenu,
    sound: &Submenu,
) -> muda::Result<Menu> {
    #[cfg(target_os = "macos")]
    {
        let app = Submenu::with_items("8bitEmu", true, &[&PredefinedMenuItem::quit(None)])?;
        return Menu::with_items(&[&app, file, machine, settings, view, sound]);
    }
    #[cfg(not(target_os = "macos"))]
    Menu::with_items(&[file, machine, settings, view, sound])
}

fn show_menu(window: &Window, menu: &Menu) {
    #[cfg(target_os = "macos")]
    {
        let _ = window;
        show_mac(menu);
    }
    #[cfg(target_os = "windows")]
    show_win(window, menu);
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        let _ = (window, menu);
    }
}

#[cfg(target_os = "macos")]
fn show_mac(menu: &Menu) {
    menu.init_for_nsapp();
}

#[cfg(target_os = "windows")]
fn show_win(window: &Window, menu: &Menu) {
    let Ok(handle) = window.window_handle() else {
        return;
    };
    let winit::raw_window_handle::RawWindowHandle::Win32(win) = handle.as_raw() else {
        return;
    };
    unsafe {
        let _ = menu.init_for_hwnd(win.hwnd.get());
    }
}

fn parented(dialog: FileDialog, window: Option<&Window>) -> FileDialog {
    let Some(window) = window else {
        return dialog;
    };
    dialog.set_parent(window)
}

impl PictureSize {
    pub fn percent(self) -> Option<u32> {
        match self {
            Self::Fit => None,
            Self::Percent(percent) => Some(percent),
        }
    }
}

impl SettingsPath {
    pub fn label(self) -> &'static str {
        match self {
            Self::SpectrumRom => "Spectrum ROM",
            Self::Kernal => "C64 Kernal",
            Self::Basic => "C64 BASIC",
            Self::Chargen => "C64 Chargen",
        }
    }
}

impl MenuBar {
    pub fn sync_machine(&self, config: &AppConfig) {
        self.machine_48.set_checked(config.is_spectrum_48());
        self.machine_128.set_checked(config.is_spectrum_128());
        self.machine_c64.set_checked(config.is_c64());
    }
}

impl SaveKind {
    pub fn id(self) -> &'static str {
        match self {
            Self::Sna => SAVE_SNA,
            Self::Tap => SAVE_TAP,
            Self::Tzx => SAVE_TZX,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Sna => "Save Snapshot…",
            Self::Tap => "Save Tape as TAP…",
            Self::Tzx => "Save Tape as TZX…",
        }
    }

    pub fn file_name(self) -> &'static str {
        match self {
            Self::Sna => "8bitemu.sna",
            Self::Tap => "8bitemu.tap",
            Self::Tzx => "8bitemu.tzx",
        }
    }

    pub fn extension(self) -> &'static str {
        match self {
            Self::Sna => "sna",
            Self::Tap => "tap",
            Self::Tzx => "tzx",
        }
    }

    fn accelerator(self) -> Accelerator {
        let mods = save_modifiers(self);
        Accelerator::new(mods, save_code(self))
    }
}

fn save_modifiers(kind: SaveKind) -> Modifiers {
    if kind == SaveKind::Sna {
        return CMD_OR_CTRL;
    }
    CMD_OR_CTRL | Modifiers::SHIFT
}

fn save_code(kind: SaveKind) -> Code {
    match kind {
        SaveKind::Sna => Code::KeyS,
        SaveKind::Tap => Code::KeyT,
        SaveKind::Tzx => Code::KeyZ,
    }
}

fn shortcut_letter(shift: bool, letter: char) -> Option<SaveKind> {
    if !shift && letter == 's' {
        return Some(SaveKind::Sna);
    }
    if shift && letter == 't' {
        return Some(SaveKind::Tap);
    }
    if shift && letter == 'z' {
        return Some(SaveKind::Tzx);
    }
    None
}

fn save_letter(code: KeyCode) -> Option<char> {
    match code {
        KeyCode::KeyS => Some('s'),
        KeyCode::KeyT => Some('t'),
        KeyCode::KeyZ => Some('z'),
        _ => None,
    }
}
