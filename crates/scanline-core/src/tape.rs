use crate::error::CoreError;

const PILOT: u32 = 2168;
const SYNC1: u32 = 667;
const SYNC2: u32 = 735;
const ZERO: u32 = 855;
const ONE: u32 = 1710;
const SILENCE: u32 = 100 * 3500;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KeyHold {
    pub row: u8,
    pub mask: u8,
}

#[derive(Clone, Copy, Debug)]
struct Edge {
    cycles: u32,
    high: bool,
    stop: bool,
}

#[derive(Debug)]
pub struct Player {
    edges: Vec<Edge>,
    index: usize,
    into: u32,
    stopped: bool,
}

pub struct Recorder {
    level: bool,
    held: u32,
    pulses: Vec<u32>,
    blocks: Vec<Vec<u8>>,
    dirty: bool,
}

struct Scan<'a> {
    bytes: &'a [u8],
    at: usize,
}

pub fn open_tape(bytes: &[u8], model_128: bool) -> Result<Player, CoreError> {
    let edges = if is_tzx(bytes) {
        read_tzx(bytes, model_128)?
    } else {
        read_tap(bytes)?
    };
    Ok(Player {
        edges,
        index: 0,
        into: 0,
        stopped: false,
    })
}

pub fn load_prompt() -> Vec<Vec<KeyHold>> {
    let mut frames = Vec::new();
    warm(&mut frames, 100);
    hold(&mut frames, 6, 0x08);
    hold_pair(&mut frames, 7, 0x02, 5, 0x01);
    hold_pair(&mut frames, 7, 0x02, 5, 0x01);
    hold(&mut frames, 6, 0x01);
    frames
}

impl Player {
    pub fn at_start(&self) -> bool {
        self.index == 0 && self.into == 0 && !self.stopped
    }

    pub fn edge_index(&self) -> usize {
        self.index
    }

    pub fn playing(&self) -> bool {
        !self.stopped && self.index < self.edges.len()
    }

    pub fn ear_high(&self) -> bool {
        if !self.playing() {
            return true;
        }
        self.edges[self.index].high
    }

    pub fn advance(&mut self, mut cycles: u32) {
        if self.stopped {
            return;
        }
        while cycles > 0 && self.index < self.edges.len() {
            if self.edges[self.index].stop {
                self.stopped = true;
                self.index += 1;
                return;
            }
            let left = self.edges[self.index].cycles.saturating_sub(self.into);
            if cycles < left {
                self.into += cycles;
                return;
            }
            cycles -= left;
            self.into = 0;
            self.index += 1;
        }
    }

    pub fn resume(&mut self) {
        self.stopped = false;
    }

    pub fn rewind(&mut self) {
        self.index = 0;
        self.into = 0;
        self.stopped = false;
    }
}

impl Default for Recorder {
    fn default() -> Self {
        Self {
            level: false,
            held: 0,
            pulses: Vec::new(),
            blocks: Vec::new(),
            dirty: false,
        }
    }
}

impl Recorder {
    pub fn advance(&mut self, cycles: u32, level: bool) {
        if level != self.level {
            self.flip(cycles, level);
            return;
        }
        self.held += cycles;
        if !level && self.held >= SILENCE {
            self.split_silence(cycles);
        }
    }

    pub fn take_tap(&mut self) -> Option<Vec<u8>> {
        if !self.dirty {
            return None;
        }
        self.dirty = false;
        self.tap_bytes()
    }

    pub fn tap_bytes(&self) -> Option<Vec<u8>> {
        if self.blocks.is_empty() {
            return None;
        }
        Some(encode_tap(&self.blocks))
    }

    pub fn tzx_bytes(&self) -> Option<Vec<u8>> {
        if self.blocks.is_empty() {
            return None;
        }
        Some(encode_tzx(&self.blocks))
    }

    fn flip(&mut self, cycles: u32, level: bool) {
        if self.held > 0 {
            self.pulses.push(self.held);
        }
        self.held = cycles;
        self.level = level;
    }

    fn split_silence(&mut self, cycles: u32) {
        let pulse = self.held.saturating_sub(cycles);
        if pulse > 0 && pulse < SILENCE {
            self.pulses.push(pulse);
        }
        self.held = 0;
        self.finish();
    }

    fn finish(&mut self) {
        let bytes = decode_standard(&self.pulses);
        self.pulses.clear();
        if bytes.is_empty() {
            return;
        }
        self.blocks.push(bytes);
        self.dirty = true;
    }
}

fn is_tzx(bytes: &[u8]) -> bool {
    bytes.len() >= 8 && &bytes[0..7] == b"ZXTape!"
}

fn read_tap(bytes: &[u8]) -> Result<Vec<Edge>, CoreError> {
    let mut scan = Scan { bytes, at: 0 };
    let mut edges = Vec::new();
    let mut level = false;
    while scan.at < scan.bytes.len() {
        let len = scan.word()? as usize;
        let data = scan.take(len)?;
        append_standard(&mut edges, &data, 1000, &mut level);
    }
    Ok(edges)
}

fn read_tzx(bytes: &[u8], model_128: bool) -> Result<Vec<Edge>, CoreError> {
    if bytes.len() < 10 || bytes[7] != 0x1A {
        return Err(tape_length(bytes.len()));
    }
    if bytes[8] != 1 {
        return Err(CoreError::Unsupported { kind: "tzx", id: bytes[8] });
    }
    let mut scan = Scan { bytes, at: 10 };
    let mut edges = Vec::new();
    let mut level = false;
    let mut loop_from: Option<(usize, u16)> = None;
    while scan.at < scan.bytes.len() {
        let id = scan.byte()?;
        read_block(&mut scan, id, model_128, &mut edges, &mut level, &mut loop_from)?;
    }
    Ok(edges)
}

fn read_block(
    scan: &mut Scan<'_>,
    id: u8,
    model_128: bool,
    edges: &mut Vec<Edge>,
    level: &mut bool,
    loop_from: &mut Option<(usize, u16)>,
) -> Result<(), CoreError> {
    match id {
        0x10 => read_standard(scan, edges, level),
        0x11 => read_turbo(scan, edges, level),
        0x12 => read_tone(scan, edges, level),
        0x13 => read_pulses(scan, edges, level),
        0x14 => read_data(scan, edges, level),
        0x15 => read_direct(scan, edges, level),
        0x20 => read_pause(scan, edges, level),
        0x21 | 0x30 => skip_text(scan),
        0x22 => Ok(()),
        0x24 => read_loop_start(scan, edges, loop_from),
        0x25 => read_loop_end(edges, loop_from),
        0x2A => read_stop_48(scan, edges, model_128),
        0x2B => read_level(scan, edges, level),
        0x31 => skip_message(scan),
        0x32 => skip_word_body(scan),
        0x33 => skip_hardware(scan),
        0x35 => skip_custom(scan),
        0x5A => scan.skip(9),
        other => Err(CoreError::Unsupported { kind: "tzx", id: other }),
    }
}

fn read_standard(scan: &mut Scan<'_>, edges: &mut Vec<Edge>, level: &mut bool) -> Result<(), CoreError> {
    let pause = scan.word()?;
    let len = scan.word()? as usize;
    let data = scan.take(len)?;
    append_standard(edges, &data, pause, level);
    Ok(())
}

fn read_turbo(scan: &mut Scan<'_>, edges: &mut Vec<Edge>, level: &mut bool) -> Result<(), CoreError> {
    let pilot_w = u32::from(scan.word()?);
    let sync1 = u32::from(scan.word()?);
    let sync2 = u32::from(scan.word()?);
    let zero = u32::from(scan.word()?);
    let one = u32::from(scan.word()?);
    let pilot_n = scan.word()?;
    let used = scan.byte()?;
    let pause = scan.word()?;
    let len = scan.u24()? as usize;
    let data = scan.take(len)?;
    append_tone(edges, pilot_w, pilot_n, level);
    push_pulse(edges, sync1, level);
    push_pulse(edges, sync2, level);
    append_bits(edges, &data, used, zero, one, level);
    append_pause(edges, pause, level);
    Ok(())
}

fn read_tone(scan: &mut Scan<'_>, edges: &mut Vec<Edge>, level: &mut bool) -> Result<(), CoreError> {
    let width = u32::from(scan.word()?);
    let count = scan.word()?;
    append_tone(edges, width, count, level);
    Ok(())
}

fn read_pulses(scan: &mut Scan<'_>, edges: &mut Vec<Edge>, level: &mut bool) -> Result<(), CoreError> {
    let count = scan.byte()?;
    let mut index = 0u8;
    while index < count {
        push_pulse(edges, u32::from(scan.word()?), level);
        index += 1;
    }
    Ok(())
}

fn read_data(scan: &mut Scan<'_>, edges: &mut Vec<Edge>, level: &mut bool) -> Result<(), CoreError> {
    let zero = u32::from(scan.word()?);
    let one = u32::from(scan.word()?);
    let used = scan.byte()?;
    let pause = scan.word()?;
    let len = scan.u24()? as usize;
    let data = scan.take(len)?;
    append_bits(edges, &data, used, zero, one, level);
    append_pause(edges, pause, level);
    Ok(())
}

fn read_direct(scan: &mut Scan<'_>, edges: &mut Vec<Edge>, level: &mut bool) -> Result<(), CoreError> {
    let sample = u32::from(scan.word()?);
    let pause = scan.word()?;
    let used = scan.byte()?;
    let len = scan.u24()? as usize;
    let data = scan.take(len)?;
    append_direct(edges, &data, used, sample, level);
    append_pause(edges, pause, level);
    Ok(())
}

fn read_pause(scan: &mut Scan<'_>, edges: &mut Vec<Edge>, level: &mut bool) -> Result<(), CoreError> {
    append_pause(edges, scan.word()?, level);
    Ok(())
}

fn read_loop_start(
    scan: &mut Scan<'_>,
    edges: &[Edge],
    loop_from: &mut Option<(usize, u16)>,
) -> Result<(), CoreError> {
    if loop_from.is_some() {
        return Err(CoreError::Unsupported { kind: "tzx", id: 0x24 });
    }
    *loop_from = Some((edges.len(), scan.word()?));
    Ok(())
}

fn read_loop_end(edges: &mut Vec<Edge>, loop_from: &mut Option<(usize, u16)>) -> Result<(), CoreError> {
    let Some((start, count)) = loop_from.take() else {
        return Ok(());
    };
    let section = edges[start..].to_vec();
    let mut times = 1u16;
    while times < count {
        edges.extend_from_slice(&section);
        times += 1;
    }
    Ok(())
}

fn read_stop_48(scan: &mut Scan<'_>, edges: &mut Vec<Edge>, model_128: bool) -> Result<(), CoreError> {
    scan.skip(4)?;
    if !model_128 {
        edges.push(Edge { cycles: 0, high: false, stop: true });
    }
    Ok(())
}

fn read_level(scan: &mut Scan<'_>, edges: &mut Vec<Edge>, level: &mut bool) -> Result<(), CoreError> {
    let mut rest = scan.dword()? as usize;
    let mut signal = false;
    if rest > 0 {
        signal = scan.byte()? != 0;
        rest -= 1;
    }
    scan.skip(rest)?;
    push_level(edges, 1, signal);
    *level = !signal;
    Ok(())
}

fn skip_text(scan: &mut Scan<'_>) -> Result<(), CoreError> {
    let len = scan.byte()? as usize;
    scan.skip(len)
}

fn skip_message(scan: &mut Scan<'_>) -> Result<(), CoreError> {
    scan.byte()?;
    skip_text(scan)
}

fn skip_word_body(scan: &mut Scan<'_>) -> Result<(), CoreError> {
    let len = scan.word()? as usize;
    scan.skip(len)
}

fn skip_hardware(scan: &mut Scan<'_>) -> Result<(), CoreError> {
    let count = scan.byte()? as usize;
    scan.skip(count * 3)
}

fn skip_custom(scan: &mut Scan<'_>) -> Result<(), CoreError> {
    scan.skip(10)?;
    let len = scan.dword()? as usize;
    scan.skip(len)
}

fn append_standard(edges: &mut Vec<Edge>, data: &[u8], pause: u16, level: &mut bool) {
    let pilot = if data.first().copied().unwrap_or(0) < 128 { 8063 } else { 3223 };
    append_tone(edges, PILOT, pilot, level);
    push_pulse(edges, SYNC1, level);
    push_pulse(edges, SYNC2, level);
    append_bits(edges, data, 8, ZERO, ONE, level);
    append_pause(edges, pause, level);
}

fn append_tone(edges: &mut Vec<Edge>, width: u32, count: u16, level: &mut bool) {
    let mut index = 0u16;
    while index < count {
        push_pulse(edges, width, level);
        index += 1;
    }
}

fn append_bits(edges: &mut Vec<Edge>, data: &[u8], used_last: u8, zero: u32, one: u32, level: &mut bool) {
    let mut index = 0;
    while index < data.len() {
        let bits = bit_count(index + 1 == data.len(), used_last);
        push_byte(edges, data[index], bits, zero, one, level);
        index += 1;
    }
}

fn push_byte(edges: &mut Vec<Edge>, byte: u8, bits: u8, zero: u32, one: u32, level: &mut bool) {
    let mut bit = 0u8;
    while bit < bits {
        let width = if byte & (0x80 >> bit) == 0 { zero } else { one };
        push_pulse(edges, width, level);
        push_pulse(edges, width, level);
        bit += 1;
    }
}

fn append_direct(edges: &mut Vec<Edge>, data: &[u8], used_last: u8, sample: u32, level: &mut bool) {
    let mut index = 0;
    while index < data.len() {
        let bits = bit_count(index + 1 == data.len(), used_last);
        push_samples(edges, data[index], bits, sample, level);
        index += 1;
    }
}

fn push_samples(edges: &mut Vec<Edge>, byte: u8, bits: u8, sample: u32, level: &mut bool) {
    let mut bit = 0u8;
    while bit < bits {
        let high = byte & (0x80 >> bit) != 0;
        push_level(edges, sample, high);
        *level = !high;
        bit += 1;
    }
}

fn bit_count(last: bool, used_last: u8) -> u8 {
    if !last {
        return 8;
    }
    if used_last == 0 { 8 } else { used_last.min(8) }
}

fn append_pause(edges: &mut Vec<Edge>, pause: u16, level: &mut bool) {
    if pause == 0 {
        edges.push(Edge { cycles: 0, high: false, stop: true });
        return;
    }
    edges.push(Edge { cycles: u32::from(pause) * 3500, high: *level, stop: false });
    *level = !*level;
}

fn push_pulse(edges: &mut Vec<Edge>, cycles: u32, level: &mut bool) {
    if cycles == 0 {
        return;
    }
    edges.push(Edge { cycles, high: *level, stop: false });
    *level = !*level;
}

fn push_level(edges: &mut Vec<Edge>, cycles: u32, high: bool) {
    if cycles == 0 {
        return;
    }
    if let Some(last) = edges.last_mut() {
        if !last.stop && last.high == high {
            last.cycles = last.cycles.saturating_add(cycles);
            return;
        }
    }
    edges.push(Edge { cycles, high, stop: false });
}

fn decode_standard(pulses: &[u32]) -> Vec<u8> {
    let mut index = skip_pilot(pulses);
    index = skip_sync(pulses, index);
    let mut bytes = Vec::new();
    let mut acc = 0u8;
    let mut bits = 0u8;
    while index + 1 < pulses.len() {
        let width = pulses[index];
        if width > 3000 {
            break;
        }
        acc = (acc << 1) | u8::from(width >= 1200);
        bits += 1;
        if bits == 8 {
            bytes.push(acc);
            acc = 0;
            bits = 0;
        }
        index += 2;
    }
    bytes
}

fn skip_pilot(pulses: &[u32]) -> usize {
    let mut index = 0;
    while index < pulses.len() && is_pilot(pulses[index]) {
        index += 1;
    }
    index
}

fn skip_sync(pulses: &[u32], mut index: usize) -> usize {
    let mut left = 2;
    while left > 0 && index < pulses.len() && pulses[index] < 1200 {
        index += 1;
        left -= 1;
    }
    index
}

fn is_pilot(width: u32) -> bool {
    (1400..3000).contains(&width)
}

fn encode_tzx(blocks: &[Vec<u8>]) -> Vec<u8> {
    let mut out = b"ZXTape!\x1a\x01\x14".to_vec();
    let mut index = 0;
    while index < blocks.len() {
        push_standard(&mut out, &blocks[index]);
        index += 1;
    }
    out
}

fn push_standard(out: &mut Vec<u8>, data: &[u8]) {
    let len = data.len() as u16;
    out.push(0x10);
    out.extend_from_slice(&1000u16.to_le_bytes());
    out.extend_from_slice(&len.to_le_bytes());
    out.extend_from_slice(data);
}

fn encode_tap(blocks: &[Vec<u8>]) -> Vec<u8> {
    let mut out = Vec::new();
    let mut index = 0;
    while index < blocks.len() {
        let len = blocks[index].len() as u16;
        out.push(len as u8);
        out.push((len >> 8) as u8);
        out.extend_from_slice(&blocks[index]);
        index += 1;
    }
    out
}

fn warm(frames: &mut Vec<Vec<KeyHold>>, count: usize) {
    let mut index = 0;
    while index < count {
        frames.push(Vec::new());
        index += 1;
    }
}

fn hold(frames: &mut Vec<Vec<KeyHold>>, row: u8, mask: u8) {
    press(frames, vec![KeyHold { row, mask }]);
}

fn hold_pair(frames: &mut Vec<Vec<KeyHold>>, row: u8, mask: u8, row2: u8, mask2: u8) {
    press(frames, vec![KeyHold { row, mask }, KeyHold { row: row2, mask: mask2 }]);
}

fn press(frames: &mut Vec<Vec<KeyHold>>, keys: Vec<KeyHold>) {
    let mut down = 0;
    while down < 8 {
        frames.push(keys.clone());
        down += 1;
    }
    let mut up = 0;
    while up < 4 {
        frames.push(Vec::new());
        up += 1;
    }
}

fn tape_length(actual: usize) -> CoreError {
    CoreError::ImageLength { name: "tape", actual }
}

impl Scan<'_> {
    fn byte(&mut self) -> Result<u8, CoreError> {
        let bytes = self.take(1)?;
        Ok(bytes[0])
    }

    fn word(&mut self) -> Result<u16, CoreError> {
        let bytes = self.take(2)?;
        Ok(u16::from(bytes[0]) | (u16::from(bytes[1]) << 8))
    }

    fn u24(&mut self) -> Result<u32, CoreError> {
        let bytes = self.take(3)?;
        Ok(u32::from(bytes[0]) | (u32::from(bytes[1]) << 8) | (u32::from(bytes[2]) << 16))
    }

    fn dword(&mut self) -> Result<u32, CoreError> {
        let bytes = self.take(4)?;
        Ok(u32::from(bytes[0])
            | (u32::from(bytes[1]) << 8)
            | (u32::from(bytes[2]) << 16)
            | (u32::from(bytes[3]) << 24))
    }

    fn take(&mut self, len: usize) -> Result<Vec<u8>, CoreError> {
        let end = self.at.checked_add(len).ok_or_else(|| tape_length(self.bytes.len()))?;
        if end > self.bytes.len() {
            return Err(tape_length(self.bytes.len()));
        }
        let out = self.bytes[self.at..end].to_vec();
        self.at = end;
        Ok(out)
    }

    fn skip(&mut self, len: usize) -> Result<(), CoreError> {
        self.take(len)?;
        Ok(())
    }
}
