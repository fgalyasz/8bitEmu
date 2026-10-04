use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, Stream, StreamConfig};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

const QUEUE_LIMIT: usize = 48_000;

pub struct Speaker {
    queue: Arc<Mutex<VecDeque<f32>>>,
    _stream: Option<Stream>,
}

impl Speaker {
    pub fn open() -> Self {
        let queue = Arc::new(Mutex::new(VecDeque::new()));
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
            queue.push_back(samples[index]);
            index += 1;
        }
        trim(&mut queue);
    }
}

fn trim(queue: &mut VecDeque<f32>) {
    while queue.len() > QUEUE_LIMIT {
        queue.pop_front();
    }
}

fn open_stream(queue: Arc<Mutex<VecDeque<f32>>>) -> Option<Stream> {
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
    queue: Arc<Mutex<VecDeque<f32>>>,
) -> Option<Stream> {
    device
        .build_output_stream(
            config,
            move |data: &mut [f32], _| fill_f32(data, &queue),
            report_error,
            None,
        )
        .ok()
}

fn build_i16(
    device: &cpal::Device,
    config: &StreamConfig,
    queue: Arc<Mutex<VecDeque<f32>>>,
) -> Option<Stream> {
    device
        .build_output_stream(
            config,
            move |data: &mut [i16], _| fill_i16(data, &queue),
            report_error,
            None,
        )
        .ok()
}

fn fill_f32(data: &mut [f32], queue: &Mutex<VecDeque<f32>>) {
    let Ok(mut queue) = queue.lock() else {
        silence(data);
        return;
    };
    let mut index = 0;
    while index < data.len() {
        data[index] = pop(&mut queue);
        index += 1;
    }
}

fn fill_i16(data: &mut [i16], queue: &Mutex<VecDeque<f32>>) {
    let Ok(mut queue) = queue.lock() else {
        let mut index = 0;
        while index < data.len() {
            data[index] = 0;
            index += 1;
        }
        return;
    };
    let mut index = 0;
    while index < data.len() {
        data[index] = (pop(&mut queue) * 32767.0) as i16;
        index += 1;
    }
}

fn silence(data: &mut [f32]) {
    let mut index = 0;
    while index < data.len() {
        data[index] = 0.0;
        index += 1;
    }
}

fn pop(queue: &mut VecDeque<f32>) -> f32 {
    queue.pop_front().unwrap_or(0.0)
}

fn report_error(error: cpal::StreamError) {
    eprintln!("scanline: {error}");
}
