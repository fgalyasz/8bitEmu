use scanline_core::{Look, demo_frame, shade_image};

#[test]
fn gpu_picture_stays_within_one_level_of_the_spec() {
    let frame = demo_frame(0);
    let spec = shade_image(&frame, Look::Sharp).expect("spec");
    let gpu = scanline_app::render_frame(&frame, Look::Sharp).expect("gpu");
    assert_eq!(gpu.len(), spec.len());
    let delta = max_channel_delta(&spec, &gpu);
    assert_eq!(delta, 0, "gpu drifted from shade_image");
}

fn max_channel_delta(spec: &[u8], gpu: &[u8]) -> i16 {
    let mut index = 0;
    let mut max_delta = 0i16;
    while index < spec.len() {
        max_delta = max_delta.max((i16::from(gpu[index]) - i16::from(spec[index])).abs());
        index += 1;
    }
    max_delta
}
