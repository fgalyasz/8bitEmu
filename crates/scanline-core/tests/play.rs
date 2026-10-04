use scanline_core::{
    aspect_fit, load_prompt, open_tape, percent_size, place_percent, CoreError, Machine, Memory,
    Ports, PresentPace, Presenter, Recorder,
};

#[test]
fn a_tap_pilot_flips_after_2168_t_states() {
    let bytes = tap_block(&[0x00, 0x01]);
    let mut player = open_tape(&bytes, false).expect("tap");
    let first = player.ear_high();
    player.advance(2167);
    assert_eq!(player.ear_high(), first);
    player.advance(1);
    assert_ne!(player.ear_high(), first);
    let mut tzx = b"ZXTape!\x1a\x01\x14".to_vec();
    tzx.push(0x10);
    tzx.extend_from_slice(&1000u16.to_le_bytes());
    tzx.extend_from_slice(&2u16.to_le_bytes());
    tzx.extend_from_slice(&[0x00, 0x01]);
    let mut other = open_tape(&tzx, false).expect("tzx");
    let other_first = other.ear_high();
    other.advance(2168);
    assert_ne!(other.ear_high(), other_first);
}

#[test]
fn an_unknown_tzx_block_names_its_id() {
    let mut bytes = b"ZXTape!\x1a\x01\x14".to_vec();
    bytes.push(0x19);
    let error = open_tape(&bytes, false).expect_err("block");
    assert!(error.to_string().contains("0x19"));
    assert_eq!(error, CoreError::Unsupported { kind: "tzx", id: 0x19 });
    let short = open_tape(&[0x02, 0x00], false).expect_err("short");
    assert!(short.to_string().contains("tape"));
    assert!(short.to_string().contains("2"));
}

#[test]
fn the_load_prompt_quotes_with_symbol_shift() {
    let frames = load_prompt();
    assert_eq!(frames[50].len(), 1);
    assert_eq!(frames[50][0].row, 6);
    assert_eq!(frames[50][0].mask, 0x02);
    let quote = frames.iter().find(|frame| frame.len() == 2).expect("quote");
    assert_eq!(quote[0].row, 7);
    assert_eq!(quote[0].mask, 0x02);
    assert_eq!(quote[1].row, 5);
    assert_eq!(quote[1].mask, 0x01);
}

#[test]
fn a_snapshot_round_trips_the_register_border_and_ram() {
    let mut crafted = vec![0u8; 49179];
    crafted[19] = 0x04;
    crafted[22] = 0x42;
    crafted[23] = 0x00;
    crafted[24] = 0x80;
    crafted[26] = 4;
    crafted[27 + 0x4000] = 0x34;
    crafted[27 + 0x4001] = 0x12;
    crafted[27 + 0x4002] = 0x5A;
    let mut machine = Machine::new();
    machine.load_sna(&crafted).expect("load");
    let image = machine.snapshot().expect("save");
    assert_eq!(image[19], 0x04);
    assert_eq!(image[22], 0x42);
    let mut again = Machine::new();
    again.load_sna(&image).expect("reload");
    assert_eq!(again.border(), 4);
    assert_eq!(again.read(0x8000), 0x34);
    assert_eq!(again.read(0x8002), 0x5A);
}

#[test]
fn standard_speed_pulses_decode_to_the_recorded_byte() {
    let mut recorder = Recorder::default();
    let mut widths = vec![2168u32; 20];
    widths.extend_from_slice(&[667, 735]);
    push_bit(&mut widths, false);
    push_bit(&mut widths, true);
    push_bit(&mut widths, false);
    push_bit(&mut widths, false);
    push_bit(&mut widths, false);
    push_bit(&mut widths, false);
    push_bit(&mut widths, true);
    push_bit(&mut widths, false);
    let mut high = true;
    let mut index = 0;
    while index < widths.len() {
        recorder.advance(widths[index], high);
        high = !high;
        index += 1;
    }
    recorder.advance(350_000, false);
    let saved = recorder.take_tap().expect("tap");
    assert_eq!(saved, vec![1, 0, 0x42]);
}

#[test]
fn kempston_is_idle_until_a_direction_or_fire_is_held() {
    let mut ports = Ports::default();
    assert_eq!(ports.input(0x001F), 0x00);
    ports.set_stick(0x01, true);
    assert_eq!(ports.input(0x001F), 0x01);
    ports.set_stick(0x10, true);
    assert_eq!(ports.input(0x001F) & 0x10, 0x10);
    assert_eq!(ports.input(0x00FE), 0xFF);
}

#[test]
fn percent_and_fit_keep_the_frame_aspect() {
    assert_eq!(percent_size(288, 224, 200), (576, 448));
    assert_eq!(percent_size(288, 224, 125), (360, 280));
    let fitted = aspect_fit(288, 224, 1000, 224);
    assert_eq!(fitted.height, 224);
    assert_eq!(fitted.width, 288);
    let narrow = aspect_fit(288, 224, 100, 1000);
    assert_eq!(narrow.width, 100);
    assert_eq!(aspect_fit(0, 0, 0, 0).width, 1);
    let placed = place_percent(288, 224, 800, 600, 200);
    assert_eq!(placed.width, 576);
    let shrunk = place_percent(288, 224, 100, 100, 200);
    assert!(shrunk.width <= 100);
}

#[test]
fn supported_tzx_blocks_become_edges() {
    let mut player = open_tape(&sample_tzx(), false).expect("tzx");
    player.advance(1_000_000);
    player.advance(1);
    assert!(player.ear_high());
    player.resume();
    player.rewind();
    assert!(player.playing());
    let nested = open_tape(&nested_loop(), false).expect_err("nest");
    assert_eq!(nested, CoreError::Unsupported { kind: "tzx", id: 0x24 });
    let version = open_tape(b"ZXTape!\x1a\x02\x00", false).expect_err("version");
    assert_eq!(version, CoreError::Unsupported { kind: "tzx", id: 0x02 });
    open_tape(&stop_block(), true).expect("128");
}

#[test]
fn a_saved_128k_image_keeps_the_paging_byte() {
    let mut image = vec![0u8; 131_103];
    image[131_101] = 0x3B;
    let mut machine = Machine::new();
    machine.load_sna(&image).expect("load");
    let saved = machine.snapshot().expect("save");
    assert_eq!(saved.len(), 131_103);
    assert_eq!(saved[131_101], 0x3B);
    let memory = Memory::new();
    let mut short = [0u8; 4];
    memory.copy_bank(9, &mut short);
    assert_eq!(short, [0, 0, 0, 0]);
}

#[test]
fn a_booted_machine_hears_the_tape_and_can_save() {
    let mut machine = Machine::new();
    let rom = vec![0x76u8; 16384];
    machine.load_rom(&rom, false).expect("rom");
    machine.load_tape(&tap_block(&[0x00, 0x00])).expect("tape");
    machine.frame().expect("frame");
    machine.resume_tape();
    machine.set_stick(0x01, true);
    machine.set_stick(0x01, false);
    machine.reset();
    assert!(machine.take_tap().is_none());
    machine.load_sna(&vec![0u8; 49179]).expect("low stack");
    let error = machine.snapshot().expect_err("stack");
    assert!(error.to_string().contains("stack"));
    let mut ports = Ports::default();
    ports.tape_on = true;
    ports.ear_high = true;
    assert_eq!(ports.input(0x00FE) & 0x40, 0x40);
    ports.ear_high = false;
    assert_eq!(ports.input(0x00FE) & 0x40, 0);
}

#[test]
fn the_prompt_runs_once_after_a_tape_is_inserted() {
    let mut presenter = Presenter::new(PresentPace::Fixed60Hz);
    let rom = vec![0x76u8; 16384];
    presenter.load_rom(&rom, false).expect("rom");
    presenter.load_tape(&tap_block(&[0x00, 0x00])).expect("tape");
    let mut tick = 0u64;
    while tick < 120 {
        presenter.on_display_tick().expect("tick");
        tick += 1;
    }
    presenter.resume_tape();
    presenter.set_stick(0x10, true);
    let _ = presenter.snapshot();
    assert!(presenter.take_tap().is_none());
    presenter.reset();
}

#[test]
fn silence_and_a_wide_pulse_end_the_recording() {
    let mut recorder = Recorder::default();
    recorder.advance(350_000, false);
    assert!(recorder.take_tap().is_none());
    recorder.advance(2168, true);
    recorder.advance(2168, false);
    recorder.advance(667, true);
    recorder.advance(735, false);
    recorder.advance(4000, true);
    recorder.advance(350_000, false);
    recorder.advance(1, false);
    assert!(recorder.take_tap().is_none());
}

fn tap_block(data: &[u8]) -> Vec<u8> {
    let mut bytes = Vec::new();
    let len = data.len() as u16;
    bytes.push(len as u8);
    bytes.push((len >> 8) as u8);
    bytes.extend_from_slice(data);
    bytes
}

fn push_bit(widths: &mut Vec<u32>, one: bool) {
    let width = if one { 1710 } else { 855 };
    widths.push(width);
    widths.push(width);
}

fn sample_tzx() -> Vec<u8> {
    let mut bytes = header();
    bytes.push(0x11);
    push_words(&mut bytes, &[1, 0, 1, 1, 2, 1]);
    bytes.push(0);
    push_word(&mut bytes, 1);
    push_u24(&mut bytes, 2);
    bytes.extend_from_slice(&[0x80, 0x01]);
    bytes.push(0x12);
    push_word(&mut bytes, 1);
    push_word(&mut bytes, 1);
    bytes.push(0x13);
    bytes.push(1);
    push_word(&mut bytes, 1);
    bytes.push(0x14);
    push_word(&mut bytes, 1);
    push_word(&mut bytes, 2);
    bytes.push(1);
    push_word(&mut bytes, 1);
    push_u24(&mut bytes, 1);
    bytes.push(0x80);
    bytes.push(0x15);
    push_word(&mut bytes, 2);
    push_word(&mut bytes, 1);
    bytes.push(8);
    push_u24(&mut bytes, 1);
    bytes.push(0xFF);
    bytes.push(0x20);
    push_word(&mut bytes, 0);
    bytes.extend_from_slice(&[0x21, 1, b'A', 0x30, 0, 0x22]);
    bytes.push(0x24);
    push_word(&mut bytes, 2);
    bytes.push(0x12);
    push_word(&mut bytes, 1);
    push_word(&mut bytes, 1);
    bytes.push(0x25);
    bytes.push(0x2A);
    bytes.extend_from_slice(&[0, 0, 0, 0]);
    bytes.push(0x2B);
    push_dword(&mut bytes, 1);
    bytes.push(1);
    bytes.push(0x2B);
    push_dword(&mut bytes, 0);
    bytes.extend_from_slice(&[0x31, 1, 1, b'B']);
    bytes.push(0x32);
    push_word(&mut bytes, 1);
    bytes.push(0);
    bytes.push(0x33);
    bytes.push(1);
    bytes.extend_from_slice(&[0, 0, 0]);
    bytes.push(0x35);
    bytes.extend_from_slice(&[0; 10]);
    push_dword(&mut bytes, 1);
    bytes.push(0);
    bytes.push(0x5A);
    bytes.extend_from_slice(&[0; 9]);
    bytes.push(0x10);
    push_word(&mut bytes, 0);
    push_word(&mut bytes, 1);
    bytes.push(0x00);
    bytes
}

fn nested_loop() -> Vec<u8> {
    let mut bytes = header();
    bytes.push(0x24);
    push_word(&mut bytes, 2);
    bytes.push(0x24);
    push_word(&mut bytes, 2);
    bytes
}

fn stop_block() -> Vec<u8> {
    let mut bytes = header();
    bytes.push(0x2A);
    bytes.extend_from_slice(&[0, 0, 0, 0]);
    bytes
}

fn header() -> Vec<u8> {
    b"ZXTape!\x1a\x01\x00".to_vec()
}

fn push_words(bytes: &mut Vec<u8>, values: &[u16]) {
    let mut index = 0;
    while index < values.len() {
        push_word(bytes, values[index]);
        index += 1;
    }
}

fn push_word(bytes: &mut Vec<u8>, value: u16) {
    bytes.push(value as u8);
    bytes.push((value >> 8) as u8);
}

fn push_u24(bytes: &mut Vec<u8>, value: u32) {
    bytes.push(value as u8);
    bytes.push((value >> 8) as u8);
    bytes.push((value >> 16) as u8);
}

fn push_dword(bytes: &mut Vec<u8>, value: u32) {
    push_u24(bytes, value);
    bytes.push((value >> 24) as u8);
}
