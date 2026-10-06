use crate::error::CoreError;

use super::mem::Map;

const BASIC_START: u16 = 0x0801;
const VARTAB: u16 = 0x002D;
const ARYTAB: u16 = 0x002F;
const STREND: u16 = 0x0031;
const LOAD_END: u16 = 0x00AE;
const NDX: u16 = 0x00C6;
const KEYD: u16 = 0x0277;

pub fn parse(bytes: &[u8]) -> Result<(u16, &[u8]), CoreError> {
    if bytes.len() < 3 {
        return Err(CoreError::ImageLength {
            name: "prg",
            actual: bytes.len(),
        });
    }
    let address = u16::from_le_bytes([bytes[0], bytes[1]]);
    Ok((address, &bytes[2..]))
}

pub fn write_payload(map: &mut Map, address: u16, payload: &[u8]) -> Result<(), CoreError> {
    payload_end(address, payload.len())?;
    copy_bytes(map, address, payload);
    Ok(())
}

fn copy_bytes(map: &mut Map, address: u16, payload: &[u8]) {
    let mut index = 0usize;
    while index < payload.len() {
        map.write_ram(address.wrapping_add(index as u16), payload[index]);
        index += 1;
    }
}

pub fn link_basic(map: &mut Map, end: u16) {
    write_word(map, VARTAB, end);
    write_word(map, ARYTAB, end);
    write_word(map, STREND, end);
    write_word(map, LOAD_END, end);
}

pub fn queue_run(map: &mut Map) {
    map.write_ram(KEYD, b'R');
    map.write_ram(KEYD + 1, b'U');
    map.write_ram(KEYD + 2, b'N');
    map.write_ram(KEYD + 3, 0x0D);
    map.write_ram(NDX, 4);
}

pub fn is_basic_load(address: u16) -> bool {
    address == BASIC_START
}

pub fn payload_end(address: u16, length: usize) -> Result<u16, CoreError> {
    let start = usize::from(address);
    let end = start.checked_add(length).ok_or(CoreError::Unsupported {
        kind: "prg",
        id: 1,
    })?;
    if end > 0x10000 {
        return Err(CoreError::Unsupported {
            kind: "prg",
            id: 1,
        });
    }
    Ok(end as u16)
}

fn write_word(map: &mut Map, address: u16, value: u16) {
    let bytes = value.to_le_bytes();
    map.write_ram(address, bytes[0]);
    map.write_ram(address + 1, bytes[1]);
}
