use muda::accelerator::{Accelerator, Code, CMD_OR_CTRL};
use muda::{Menu, MenuItem, PredefinedMenuItem, Submenu};
use rfd::FileDialog;
use std::path::PathBuf;
use winit::keyboard::{KeyCode, ModifiersState};
use winit::window::Window;

pub const OPEN_ID: &str = "open";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OpenKind {
    Rom,
    Sna,
    Tape,
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

pub fn install_menu(window: &Window) -> Option<Menu> {
    let menu = build_menu().ok()?;
    show_menu(window, &menu);
    Some(menu)
}

pub fn pick_path(window: Option<&Window>) -> Option<PathBuf> {
    let dialog = FileDialog::new()
        .set_title("Open")
        .add_filter("Spectrum", &["rom", "sna", "tap", "tzx"]);
    parented(dialog, window).pick_file()
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

fn build_menu() -> muda::Result<Menu> {
    let open = open_item();
    let file = Submenu::with_items("File", true, &[&open])?;
    menu_with_file(&file)
}

fn open_item() -> MenuItem {
    MenuItem::with_id(OPEN_ID, "Open…", true, Some(open_accelerator()))
}

fn open_accelerator() -> Accelerator {
    Accelerator::new(CMD_OR_CTRL, Code::KeyO)
}

fn menu_with_file(file: &Submenu) -> muda::Result<Menu> {
    #[cfg(target_os = "macos")]
    {
        let app = Submenu::with_items("Scanline", true, &[&PredefinedMenuItem::quit(None)])?;
        return Menu::with_items(&[&app, file]);
    }
    #[cfg(not(target_os = "macos"))]
    Menu::with_items(&[file])
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
