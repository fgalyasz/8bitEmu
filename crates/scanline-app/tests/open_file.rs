use scanline_app::{open_kind, rom_model_128, sna_model_128, OpenKind};

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
fn image_length_selects_the_machine() {
    assert!(rom_model_128(32 * 1024, false));
    assert!(!rom_model_128(16 * 1024, false));
    assert!(rom_model_128(16 * 1024, true));
    assert!(sna_model_128(131_103, false));
    assert!(!sna_model_128(49_179, true));
}
