use scanline_app::{
    ensure_extension, open_kind, picture_size, rom_model_128, save_kind, save_shortcut,
    sna_model_128, OpenKind, PictureSize, SaveKind,
};

#[test]
fn extensions_select_the_loader() {
    let game = "Bosconian '87 (Europe).tzx";
    assert_eq!(open_kind(game), Some(OpenKind::Tape));
    assert_eq!(open_kind("GAME.TAP"), Some(OpenKind::Tape));
    assert_eq!(open_kind("48.rom"), Some(OpenKind::Rom));
    assert_eq!(open_kind("snap.sna"), Some(OpenKind::Sna));
    assert_eq!(open_kind("notes.txt"), None);
    assert_eq!(open_kind("noext"), None);
}

#[test]
fn menu_ids_select_the_size_and_the_save() {
    assert_eq!(picture_size("size-200"), Some(PictureSize::Percent(200)));
    assert_eq!(picture_size("size-125"), Some(PictureSize::Percent(125)));
    assert_eq!(picture_size("size-fit"), Some(PictureSize::Fit));
    assert_eq!(picture_size("size-fit").unwrap().percent(), None);
    assert_eq!(picture_size("open"), None);
    assert_eq!(save_kind("save-sna"), Some(SaveKind::Sna));
    assert_eq!(save_kind("save-tap"), Some(SaveKind::Tap));
    assert_eq!(save_kind("save-tzx"), Some(SaveKind::Tzx));
    assert_eq!(save_kind("open"), None);
    assert_eq!(ensure_extension("notes", "sna"), "notes.sna");
    assert_eq!(ensure_extension("game.TAP", "tap"), "game.TAP");
    assert_eq!(save_shortcut(true, false, 's'), Some(SaveKind::Sna));
    assert_eq!(save_shortcut(true, true, 't'), Some(SaveKind::Tap));
    assert_eq!(save_shortcut(true, true, 'z'), Some(SaveKind::Tzx));
    assert_eq!(save_shortcut(false, false, 's'), None);
}

#[test]
fn image_length_selects_the_machine() {
    assert!(rom_model_128(32 * 1024, false));
    assert!(!rom_model_128(16 * 1024, false));
    assert!(rom_model_128(16 * 1024, true));
    assert!(sna_model_128(131_103, false));
    assert!(!sna_model_128(49_179, true));
}
