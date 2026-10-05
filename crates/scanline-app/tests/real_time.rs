use scanline_app::{due_ticks, spread};
use std::collections::VecDeque;
use std::time::Duration;

#[test]
fn lateness_runs_whole_steps_up_to_the_cap() {
    let frame = Duration::from_nanos(1_000_000_000 / 60);
    assert_eq!(due_ticks(frame - Duration::from_nanos(1), frame, 4), 0);
    assert_eq!(due_ticks(frame, frame, 4), 1);
    assert_eq!(due_ticks(frame * 5, frame, 4), 4);
    assert_eq!(due_ticks(frame, Duration::ZERO, 4), 0);
}

#[test]
fn one_sample_fills_every_channel() {
    let mut queue = VecDeque::from([0.5, -0.5]);
    let mut held = 0.0;
    let mut data = [0.0; 4];
    spread(&mut data, &mut queue, &mut held, 2);
    assert_eq!(data, [0.5, 0.5, -0.5, -0.5]);
    let mut rest = [0.0; 2];
    spread(&mut rest, &mut queue, &mut held, 2);
    assert_eq!(rest, [-0.5, -0.5]);
}