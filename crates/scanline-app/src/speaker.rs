use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, Stream, StreamConfig};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

const QUEUE_LIMIT: usize = 48_000;
const LEAD: usize = 4_096;

struct Pending {
    samples: VecDeque<f32>,
    held: f32,
    live: bool,
}

pub struct Speaker {
    queue: Arc<Mutex<Pending>>,
    _stream: Option<Stream>,
}

impl Speaker {
    pub fn open() -> Self {
        let queue = Arc::new(Mutex::new(Pending {
            samples: VecDeque::new(),
            held: 0.0,
            live: false,
        }));
        let stream = open_stream(queue.clone());
        if stream.is_none() {
            eprintln!("scanline: audio device unavailable");
        }
        Self {
            queue,
            _stream: stream,
        }
    }

    pub fn push(&self, samples: &[f32]) {
        let Ok(mut queue) = self.queue.lock() else {
            return;
        };
        let mut index = 0;
        while index < samples.len() {
            queue.samples.push_back(samples[index]);
            index += 1;
        }
        trim(&mut queue.samples);
    }
}

fn trim(queue: &mut VecDeque<f32>) {
    while queue.len() > QUEUE_LIMIT {
        queue.pop_front();
    }
}

fn open_stream(queue: Arc<Mutex<Pending>>) -> Option<Stream> {
    let device = cpal::default_host().default_output_device()?;
    let supported = device.default_output_config().ok()?;
    let config = supported.config();
    let stream = match supported.sample_format() {
        SampleFormat::F32 => build_f32(&device, &config, queue),
        SampleFormat::I16 => build_i16(&device, &config, queue),
        _ => None,
    }?;
    stream.play().ok()?;
    Some(stream)
}

fn build_f32(
    device: &cpal::Device,
    config: &StreamConfig,
    queue: Arc<Mutex<Pending>>,
) -> Option<Stream> {
    let channels = channel_count(config);
    device
        .build_output_stream(
            config,
            move |data: &mut [f32], _| fill_f32(data, &queue, channels),
            report_error,
            None,
        )
        .ok()
}

fn build_i16(
    device: &cpal::Device,
    config: &StreamConfig,
    queue: Arc<Mutex<Pending>>,
) -> Option<Stream> {
    let channels = channel_count(config);
    device
        .build_output_stream(
            config,
            move |data: &mut [i16], _| fill_i16(data, &queue, channels),
            report_error,
            None,
        )
        .ok()
}

fn fill_f32(data: &mut [f32], queue: &Mutex<Pending>, channels: usize) {
    let Ok(mut pending) = queue.lock() else {
        silence(data);
        return;
    };
    write_f32(data, &mut pending, channels);
}

fn write_f32(data: &mut [f32], pending: &mut Pending, channels: usize) {
    let width = channels.max(1);
    let mut index = 0;
    while index < data.len() {
        let sample = emit(pending);
        let mut channel = 0;
        while channel < width && index < data.len() {
            data[index] = sample;
            index += 1;
            channel += 1;
        }
    }
}

fn emit(pending: &mut Pending) -> f32 {
    if !pending.live && pending.samples.len() < LEAD {
        if let Some(sample) = pending.samples.front() {
            pending.held = *sample;
        }
        return pending.held;
    }
    pending.live = true;
    next_sample(&mut pending.samples, &mut pending.held)
}

pub fn spread(data: &mut [f32], queue: &mut VecDeque<f32>, held: &mut f32, channels: usize) {
    let width = channels.max(1);
    let mut index = 0;
    while index < data.len() {
        let sample = next_sample(queue, held);
        let mut channel = 0;
        while channel < width && index < data.len() {
            data[index] = sample;
            index += 1;
            channel += 1;
        }
    }
}

fn next_sample(queue: &mut VecDeque<f32>, held: &mut f32) -> f32 {
    let Some(sample) = queue.pop_front() else {
        return *held;
    };
    *held = sample;
    sample
}

fn channel_count(config: &StreamConfig) -> usize {
    usize::from(config.channels).max(1)
}

fn fill_i16(data: &mut [i16], queue: &Mutex<Pending>, channels: usize) {
    let Ok(mut pending) = queue.lock() else {
        let mut index = 0;
        while index < data.len() {
            data[index] = 0;
            index += 1;
        }
        return;
    };
    let width = channels.max(1);
    let mut index = 0;
    while index < data.len() {
        let sample = (emit(&mut pending) * 32767.0) as i16;
        let mut channel = 0;
        while channel < width && index < data.len() {
            data[index] = sample;
            index += 1;
            channel += 1;
        }
    }
}

fn silence(data: &mut [f32]) {
    let mut index = 0;
    while index < data.len() {
        data[index] = 0.0;
        index += 1;
    }
}

fn report_error(error: cpal::StreamError) {
    eprintln!("scanline: {error}");
}

#[cfg(test)]
mod tests {
    use super::{emit, Pending, LEAD};
    use std::collections::VecDeque;

    #[test]
    fn playback_waits_until_the_lead_is_queued() {
        let mut pending = Pending {
            samples: VecDeque::new(),
            held: 0.0,
            live: false,
        };
        assert_eq!(emit(&mut pending), 0.0);
        let mut index = 0;
        while index < LEAD - 1 {
            pending.samples.push_back(-0.2);
            index += 1;
        }
        assert_eq!(emit(&mut pending), -0.2);
        assert!(!pending.live);
        assert_eq!(pending.samples.len(), LEAD - 1);
        pending.samples.push_back(0.5);
        assert_eq!(emit(&mut pending), -0.2);
        assert!(pending.live);
        assert_eq!(pending.samples.len(), LEAD - 1);
    }
}
