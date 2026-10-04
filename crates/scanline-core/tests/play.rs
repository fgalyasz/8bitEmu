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
    assert!(frames[..100].iter().all(|frame| frame.is_empty()));
    let first = frames.iter().find(|frame| !frame.is_empty()).expect("key");
    assert_eq!(first.len(), 1);
    assert_eq!(first[0].row, 6);
    assert_eq!(first[0].mask, 0x08);
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
    assert_eq!(recorder.tap_bytes().as_ref(), Some(&saved));
    let tzx = recorder.tzx_bytes().expect("tzx");
    assert_eq!(&tzx[..10], b"ZXTape!\x1a\x01\x14");
    open_tape(&tzx, false).expect("tzx");
    assert!(Recorder::default().tap_bytes().is_none());
    assert!(Recorder::default().tzx_bytes().is_none());
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
    assert!(presenter.is_booted());
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

#[test]
fn the_tape_waits_until_the_ear_is_polled() {
    let mut parked = Machine::new();
    parked.load_rom(&vec![0x76; 16384], false).expect("halt");
    parked.load_tape(&tap_block(&[0x00, 0x00])).expect("tape");
    let mut frame = 0;
    while frame < 3 {
        parked.frame().expect("frame");
        frame += 1;
    }
    assert!(parked.tape_at_start());
    let mut polling = polling_rom();
    polling.load_tape(&tap_block(&[0x00, 0x00])).expect("tape");
    polling.frame().expect("arm");
    assert!(polling.tape_at_start());
    polling.frame().expect("play");
    assert!(!polling.tape_at_start());
    let mut waiting = counted_then_halt();
    waiting.load_tape(&tap_block(&[0x00, 0x00])).expect("tape");
    waiting.frame().expect("arm");
    assert_eq!(waiting.tape_edge(), 0);
    waiting.frame().expect("play");
    let moved = waiting.tape_edge();
    assert!(moved > 0);
    waiting.frame().expect("keep");
    assert!(waiting.tape_edge() > moved);
}

#[test]
fn loading_sound_off_budgets_turbo_only_while_loading() {
    let mut parked = Machine::new();
    parked.set_loading_sound(false);
    parked.load_rom(&vec![0x76; 16384], false).expect("halt");
    parked.load_tape(&tap_block(&[0x00, 0x00])).expect("tape");
    parked.frame().expect("frame");
    assert!(parked.hears_loading());
    assert_eq!(parked.turbo_budget(), 0);
    let mut polling = polling_rom();
    polling.load_tape(&tap_block(&[0x00, 0x00])).expect("tape");
    polling.frame().expect("arm");
    assert_eq!(polling.turbo_budget(), 0);
    assert!(polling.hears_loading());
    polling.set_loading_sound(false);
    assert_eq!(polling.turbo_budget(), 1);
    assert!(!polling.hears_loading());
}

#[test]
fn the_loading_beep_follows_the_ear_when_sound_is_on() {
    let heard = beep_after_start(true);
    assert!(heard.iter().any(|sample| *sample > 0.0));
    assert!(heard.iter().any(|sample| *sample < -0.2));
    let quiet = beep_after_start(false);
    let first = quiet[0];
    assert!(quiet.iter().all(|sample| *sample == first));
}

fn beep_after_start(sound: bool) -> Vec<f32> {
    let mut machine = polling_rom();
    machine.set_loading_sound(sound);
    machine.load_tape(&tap_block(&[0x00, 0x00])).expect("tape");
    machine.frame().expect("arm");
    let _ = machine.take_audio();
    machine.frame().expect("play");
    machine.take_audio()
}

#[test]
fn a_quiet_load_runs_ahead_after_the_prompt() {
    let mut heard = polling_presenter(true);
    let mut quiet = polling_presenter(false);
    let mut tick = 0;
    while tick < 148 {
        heard.on_display_tick().expect("tick");
        quiet.on_display_tick().expect("tick");
        tick += 1;
    }
    assert_eq!(heard.tape_edge(), quiet.tape_edge());
    assert!(quiet.rush().expect("rush").is_none());
    heard.on_display_tick().expect("step");
    quiet.on_display_tick().expect("step");
    assert_eq!(heard.tape_edge(), quiet.tape_edge());
    assert!(quiet.rush().expect("ahead").is_some());
    assert!(quiet.tape_edge() > heard.tape_edge());
    assert!(heard.rush().expect("heard").is_none());
}

fn polling_presenter(sound: bool) -> Presenter {
    let mut presenter = Presenter::new(PresentPace::VariableRefresh);
    presenter.load_rom(&polling_bytes(), false).expect("rom");
    presenter.load_tape(&tap_block(&[0x00, 0x00])).expect("tape");
    presenter.set_loading_sound(sound);
    presenter
}

fn polling_bytes() -> Vec<u8> {
    let mut bytes = vec![0x00; 16384];
    bytes[0] = 0x3E;
    bytes[1] = 0xFE;
    bytes[2] = 0xDB;
    bytes[3] = 0xFE;
    bytes[4] = 0x18;
    bytes[5] = 0xFA;
    bytes
}

#[test]
fn the_pause_ends_the_last_data_pulse() {
    let lead = 8063 * 2168 + 667 + 735 + 16 * 855;
    let mut player = open_tape(&tap_block(&[0x00]), false).expect("tap");
    player.advance(lead - 1);
    let before = player.ear_high();
    player.advance(1);
    assert_ne!(player.ear_high(), before);
}

fn counted_then_halt() -> Machine {
    let mut bytes = vec![0x00; 16384];
    bytes[0] = 0x3E;
    bytes[1] = 0xFE;
    bytes[2] = 0x06;
    bytes[3] = 0x00;
    bytes[4] = 0xDB;
    bytes[5] = 0xFE;
    bytes[6] = 0x10;
    bytes[7] = 0xFC;
    bytes[8] = 0x76;
    let mut machine = Machine::new();
    machine.load_rom(&bytes, false).expect("rom");
    machine
}

fn polling_rom() -> Machine {
    let mut machine = Machine::new();
    machine.load_rom(&polling_bytes(), false).expect("rom");
    machine
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
