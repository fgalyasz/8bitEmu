use eightbit_emu_core::{
    Attribute, BORDER, Blend, CONTENT_HEIGHT, CONTENT_WIDTH, Content, CoreError, Look, Mailbox,
    PaletteKind, PresentPace, Presenter, Rgb, Sprite, TRANSPARENT, TemporalHistory,
    attribute_index, c64_pal_clock, centered_viewport, compose, demo_frame, demo_sprite_x,
    frames_from_samples, integer_scale, linear_unorm16, mix_half, mix_quarter, shade_image,
    source_index, spectrum_clock,
};

#[test]
fn clocks_are_exact_and_distinct() {
    let spectrum = spectrum_clock();
    let c64 = c64_pal_clock();
    assert_eq!((spectrum.numerator, spectrum.denominator), (15_625, 312));
    assert_eq!((c64.numerator, c64.denominator), (13_684, 273));
    assert_ne!(spectrum, c64);
}

#[test]
fn audio_clock_counts_whole_frames_only() {
    let clock = spectrum_clock();
    assert_eq!(frames_from_samples(958, 48_000, clock).unwrap(), 0);
    assert_eq!(frames_from_samples(959, 48_000, clock).unwrap(), 1);
    let c64 = frames_from_samples(957, 48_000, c64_pal_clock()).unwrap();
    assert_eq!(c64, 0);
    assert_eq!(
        frames_from_samples(958, 48_000, c64_pal_clock()).unwrap(),
        1
    );
}

#[test]
fn audio_clock_rejects_a_zero_sample_rate() {
    let error = frames_from_samples(1, 0, spectrum_clock()).unwrap_err();
    assert_eq!(error, CoreError::SampleRate);
    assert!(error.to_string().contains("sample rate"));
}

#[test]
fn scale_fits_an_integer_and_rejects_empty_destinations() {
    assert_eq!(integer_scale(10, 10, 25, 40), Some(2));
    assert_eq!(integer_scale(100, 100, 50, 80), Some(1));
    assert_eq!(integer_scale(0, 10, 10, 10), None);
    assert_eq!(integer_scale(10, 10, 0, 10), None);
}

#[test]
fn viewport_centers_the_scaled_frame() {
    let view = centered_viewport(10, 8, 30, 20, 2);
    assert_eq!((view.x, view.y, view.width, view.height), (5, 2, 20, 16));
}

#[test]
fn cadence_repeats_one_source_frame_every_six_refreshes() {
    let pace = PresentPace::Fixed60Hz;
    assert_eq!(cadence_prefix(pace), vec![0, 0, 1, 2, 3, 4, 5]);
    assert_eq!(source_index(4, PresentPace::VariableRefresh), 4);
}

#[test]
fn mailbox_keeps_only_the_latest_frame() {
    let mut mailbox = Mailbox::default();
    mailbox.publish(1);
    mailbox.publish(2);
    assert_eq!(mailbox.latest(), Some(2));
    assert_eq!(mailbox.latest(), None);
}

#[test]
fn palette_bytes_match_the_published_tables() {
    let ula = PaletteKind::Ula.colors();
    let c64 = PaletteKind::C64.colors();
    assert_eq!(
        ula[2],
        Rgb {
            r: 0xD7,
            g: 0,
            b: 0
        }
    );
    assert_eq!(
        ula[15],
        Rgb {
            r: 255,
            g: 255,
            b: 255
        }
    );
    assert_eq!(
        c64[2],
        Rgb {
            r: 0x96,
            g: 0x28,
            b: 0x2E
        }
    );
    assert_eq!(linear_unorm16(0), 0);
    assert_eq!(linear_unorm16(255), 65_535);
    assert_eq!(linear_unorm16(215), 44_534);
}

#[test]
fn mixes_happen_in_linear_light() {
    let black = Rgb { r: 0, g: 0, b: 0 };
    let white = Rgb {
        r: 255,
        g: 255,
        b: 255,
    };
    let red = Rgb {
        r: 0xD7,
        g: 0,
        b: 0,
    };
    assert_eq!(mix_half(black, white).r, 188);
    assert_eq!(mix_quarter(red, white).r, 226);
    assert_ne!(mix_half(black, white).r, 128);
}

#[test]
fn look_digits_select_the_three_modes() {
    assert_eq!(Look::from_digit(1), Some(Look::Sharp));
    assert_eq!(Look::from_digit(2), Some(Look::SoftEdge));
    assert_eq!(Look::from_digit(3), Some(Look::Temporal));
    assert_eq!(Look::from_digit(9), None);
    assert_eq!(Look::Sharp.title(), "8bitEmu — Sharp");
    assert_eq!(Look::Temporal.shader_id(), 2);
}

#[test]
fn attribute_flash_swaps_ink_and_paper() {
    let attr = Attribute {
        ink: 2,
        paper: 1,
        bright: true,
        flash: true,
    };
    assert_eq!(attribute_index(attr, false, false), 9);
    assert_eq!(attribute_index(attr, false, true), 10);
    assert_eq!(attribute_index(attr, true, true), 9);
}

#[test]
fn compose_rejects_bad_planes() {
    let mut content = tiny_content();
    content.bitmap.clear();
    assert!(matches!(
        compose(&content),
        Err(CoreError::BitmapLength { .. })
    ));
    let mut wide = tiny_content();
    wide.width = 7;
    let text = compose(&wide).unwrap_err().to_string();
    assert!(text.contains("7"));
    content.bitmap.clear();
    let bitmap = compose(&content).unwrap_err().to_string();
    assert!(bitmap.contains("bitmap length"));
}

#[test]
fn compose_rejects_bad_sprites_and_border_indexes() {
    let mut content = tiny_content();
    content.sprites[0].pixels.pop();
    let pixels = compose(&content).unwrap_err().to_string();
    assert!(pixels.contains("sprite pixels"));
    let mut border = tiny_content();
    border.border[0] = 16;
    assert!(matches!(
        compose(&border),
        Err(CoreError::IndexRange { index: 16 })
    ));
}

#[test]
fn border_scanline_and_bitmap_msb_survive_compose() {
    let frame = compose(&tiny_content()).unwrap();
    let origin = content_origin(&frame);
    assert_eq!(frame.index[0], 2);
    assert_eq!(frame.luma[origin], 1);
    assert_eq!(frame.index[origin], 2);
}

#[test]
fn black_sprite_is_visible_and_transparent_is_not() {
    let frame = compose(&tiny_content()).unwrap();
    let origin = content_origin(&frame);
    assert_eq!(frame.sprite_on[origin + 1], 1);
    assert_eq!(frame.sprite[origin + 1], 0);
    assert_eq!(frame.sprite_on[origin], 0);
}

#[test]
fn sharp_pixels_stay_on_the_palette_bytes() {
    let frame = compose(&tiny_content()).unwrap();
    let image = shade_image(&frame, Look::Sharp).unwrap();
    let offset = content_origin(&frame) * 4;
    assert_eq!(&image[offset..offset + 4], &[0xD7, 0, 0, 255]);
}

#[test]
fn soft_edge_pulls_a_luma_step_toward_its_neighbor() {
    let mut content = tiny_content();
    content.sprites.clear();
    let frame = compose(&content).unwrap();
    let sharp = shade_image(&frame, Look::Sharp).unwrap();
    let soft = shade_image(&frame, Look::SoftEdge).unwrap();
    let offset = content_origin(&frame) * 4;
    let red = Rgb {
        r: 0xD7,
        g: 0,
        b: 0,
    };
    let black = Rgb { r: 0, g: 0, b: 0 };
    assert_ne!(soft[offset], sharp[offset]);
    assert_eq!(soft[offset], mix_quarter(red, black).r);
}

#[test]
fn temporal_blends_flicker_and_leaves_motion_alone() {
    let mut history = TemporalHistory::default();
    let width = 4;
    history.decide(&[1, 0, 1, 0], width).unwrap();
    history.decide(&[6, 0, 1, 0], width).unwrap();
    let flicker = history.decide(&[1, 0, 1, 0], width).unwrap();
    assert_eq!(flicker.blend[0], Blend::Previous);
    assert_eq!(flicker.previous[0], 6);
    let mut motion = TemporalHistory::default();
    motion.decide(&[1, 0, 0, 0], width).unwrap();
    motion.decide(&[2, 0, 0, 0], width).unwrap();
    let moved = motion.decide(&[3, 0, 0, 0], width).unwrap();
    assert_eq!(moved.blend[0], Blend::Current);
}

#[test]
fn temporal_blends_a_stable_checker_inside_the_row() {
    let mut history = TemporalHistory::default();
    let row = [7, 0, 7, 0];
    history.decide(&row, 4).unwrap();
    history.decide(&row, 4).unwrap();
    let decision = history.decide(&row, 4).unwrap();
    assert_eq!(decision.blend[0], Blend::Neighbor);
    assert_eq!(decision.blend[3], Blend::Current);
}

#[test]
fn temporal_rejects_a_short_history_buffer() {
    let mut history = TemporalHistory::default();
    history.decide(&[1, 2], 2).unwrap();
    let error = history.decide(&[1], 1).unwrap_err();
    assert_eq!(
        error,
        CoreError::HistoryLength {
            expected: 1,
            actual: 2
        }
    );
}

#[test]
fn shaded_temporal_pixel_is_the_linear_mix() {
    let mut frame = compose(&block_content()).unwrap();
    frame.blend[0] = Blend::Previous as u8;
    frame.previous[0] = 6;
    frame.index[0] = 1;
    let image = shade_image(&frame, Look::Temporal).unwrap();
    let expected = mix_half(PaletteKind::Ula.colors()[1], PaletteKind::Ula.colors()[6]);
    assert_eq!(&image[0..3], &[expected.r, expected.g, expected.b]);
}

#[test]
fn demo_pattern_moves_the_sprite_and_keeps_the_border_stripe() {
    let first = demo_frame(0);
    let second = demo_frame(1);
    assert_eq!(demo_sprite_x(0), 0);
    assert_eq!(demo_sprite_x(1), 2);
    assert_eq!(first.index[4 * usize::from(first.width)], 2);
    assert_ne!(sprite_column(&first), sprite_column(&second));
    assert_eq!(first.width, CONTENT_WIDTH + BORDER * 2);
    assert_eq!(first.height, CONTENT_HEIGHT + BORDER * 2);
}

#[test]
fn presenter_holds_the_source_frame_across_the_repeated_refresh() {
    let mut presenter = Presenter::new(PresentPace::Fixed60Hz);
    let first = cycling_ink(presenter.on_display_tick().unwrap());
    let repeated = cycling_ink(presenter.on_display_tick().unwrap());
    let advanced = cycling_ink(presenter.on_display_tick().unwrap());
    assert_eq!(first, 1);
    assert_eq!(first, repeated);
    assert_eq!(advanced, 2);
    presenter.set_look(Look::SoftEdge);
    assert_eq!(presenter.look(), Look::SoftEdge);
}

#[test]
fn error_text_names_the_failing_plane() {
    assert!(empty_text().contains("non-zero"));
    assert!(align_text().contains("width 4"));
    assert!(CoreError::EmptyFrame.to_string().contains("presenter"));
    assert!(
        CoreError::IndexRange { index: 19 }
            .to_string()
            .contains("19")
    );
    assert!(history_text().contains("history length"));
}

#[test]
fn compose_reports_attribute_border_and_sprite_luma() {
    assert!(matches!(
        compose(&cleared_attributes()),
        Err(CoreError::AttributeLength { .. })
    ));
    assert!(matches!(
        compose(&short_border()),
        Err(CoreError::BorderLength { .. })
    ));
    assert!(matches!(
        compose(&short_luma()),
        Err(CoreError::SpriteLuma { .. })
    ));
    assert!(attribute_text().contains("attribute length"));
    assert!(
        compose(&short_border())
            .unwrap_err()
            .to_string()
            .contains("border length")
    );
    assert!(
        compose(&short_luma())
            .unwrap_err()
            .to_string()
            .contains("sprite luma")
    );
    assert_eq!(Look::Temporal.title(), "8bitEmu — Temporal color");
}

#[test]
fn clipped_sprite_and_bad_shade_inputs_fail_clearly() {
    let clipped = compose(&offscreen_sprite()).unwrap();
    assert!(no_sprite(&clipped));
    let mut frame = compose(&block_content()).unwrap();
    frame.luma.pop();
    assert!(
        shade_image(&frame, Look::Sharp)
            .unwrap_err()
            .to_string()
            .contains("luma")
    );
    frame = compose(&block_content()).unwrap();
    frame.index[0] = 16;
    assert!(shade_image(&frame, Look::Sharp).is_err());
}

#[test]
fn temporal_neighbor_blend_and_sprite_luma_are_shaded() {
    let mut frame = compose(&block_content()).unwrap();
    frame.blend[0] = Blend::Neighbor as u8;
    frame.index[0] = 1;
    frame.index[1] = 6;
    let image = shade_image(&frame, Look::Temporal).unwrap();
    let expected = mix_half(PaletteKind::Ula.colors()[1], PaletteKind::Ula.colors()[6]);
    assert_eq!(image[0], expected.r);
    frame.sprite_on[0] = 1;
    frame.sprite[0] = 15;
    frame.sprite_luma[0] = 1;
    frame.luma[0] = 0;
    let soft = shade_image(&frame, Look::SoftEdge).unwrap();
    assert_eq!(soft[0], 255);
    assert_eq!(Look::SoftEdge.title(), "8bitEmu — Soft edge");
    assert_eq!(Look::Sharp.shader_id(), 0);
    assert_eq!(Look::SoftEdge.shader_id(), 1);
}
#[test]
fn empty_content_is_rejected() {
    let mut content = tiny_content();
    content.width = 0;
    content.height = 0;
    assert_eq!(compose(&content).unwrap_err(), CoreError::EmptySize);
}

fn empty_text() -> String {
    let mut content = tiny_content();
    content.width = 0;
    content.height = 0;
    compose(&content).unwrap_err().to_string()
}

fn align_text() -> String {
    let mut content = tiny_content();
    content.width = 4;
    compose(&content).unwrap_err().to_string()
}

fn history_text() -> String {
    let mut history = TemporalHistory::default();
    history.decide(&[1, 2], 2).unwrap();
    history.decide(&[1], 1).unwrap_err().to_string()
}

fn cleared_attributes() -> Content {
    let mut content = block_content();
    content.attributes.clear();
    content
}

fn short_border() -> Content {
    let mut content = block_content();
    content.border.clear();
    content
}

fn short_luma() -> Content {
    let mut content = tiny_content();
    content.sprites[0].luma.pop();
    content
}

fn attribute_text() -> String {
    compose(&cleared_attributes()).unwrap_err().to_string()
}

fn offscreen_sprite() -> Content {
    let mut content = block_content();
    content.sprites.push(Sprite {
        x: -40,
        y: 0,
        width: 1,
        height: 1,
        pixels: vec![1],
        luma: vec![1],
    });
    content
}

fn no_sprite(frame: &eightbit_emu_core::Frame) -> bool {
    let mut index = 0;
    while index < frame.sprite_on.len() {
        if frame.sprite_on[index] != 0 {
            return false;
        }
        index += 1;
    }
    true
}

fn tiny_content() -> Content {
    Content {
        width: 8,
        height: 1,
        bitmap: vec![0b1000_0000],
        attributes: vec![Attribute {
            ink: 2,
            paper: 0,
            bright: false,
            flash: false,
        }],
        flash_on: false,
        border_px: BORDER,
        border: tiny_border(),
        sprites: vec![tiny_sprite()],
        palette: PaletteKind::Ula,
    }
}

fn tiny_border() -> Vec<u8> {
    let mut border = vec![0; usize::from(1 + BORDER * 2)];
    border[0] = 2;
    border
}

fn tiny_sprite() -> Sprite {
    let mut pixels = vec![TRANSPARENT; 4];
    pixels[1] = 0;
    Sprite {
        x: 0,
        y: 0,
        width: 4,
        height: 1,
        pixels,
        luma: vec![1; 4],
    }
}

fn block_content() -> Content {
    Content {
        width: 8,
        height: 1,
        bitmap: vec![0xff],
        attributes: vec![Attribute {
            ink: 1,
            paper: 0,
            bright: false,
            flash: false,
        }],
        flash_on: false,
        border_px: 0,
        border: vec![0],
        sprites: Vec::new(),
        palette: PaletteKind::Ula,
    }
}

fn content_origin(frame: &eightbit_emu_core::Frame) -> usize {
    usize::from(BORDER) * usize::from(frame.width) + usize::from(BORDER)
}

fn cadence_prefix(pace: PresentPace) -> Vec<u64> {
    let mut got = Vec::new();
    let mut tick = 0u64;
    while tick < 7 {
        got.push(source_index(tick, pace));
        tick += 1;
    }
    got
}

fn cycling_ink(frame: &eightbit_emu_core::Frame) -> u8 {
    let x = usize::from(BORDER);
    let y = usize::from(BORDER);
    frame.index[y * usize::from(frame.width) + x]
}

fn sprite_column(frame: &eightbit_emu_core::Frame) -> usize {
    let mut index = 0;
    while index < frame.sprite_on.len() {
        if frame.sprite_on[index] != 0 {
            return index % usize::from(frame.width);
        }
        index += 1;
    }
    0
}
