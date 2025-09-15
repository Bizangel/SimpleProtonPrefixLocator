use serde::Serialize;
use std::collections::HashMap;
use std::io::{self, Cursor, Read};

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum VdfValue {
    Map(VdfMap),
    String(String),
    Number(u32),
}

impl VdfValue {
    pub fn as_map(&self) -> Option<&VdfMap> {
        match self {
            VdfValue::Map(m) => Some(m),
            _ => None,
        }
    }

    pub fn into_map(self) -> Option<VdfMap> {
        match self {
            VdfValue::Map(m) => Some(m),
            _ => None,
        }
    }

    pub fn as_map_mut(&mut self) -> Option<&mut VdfMap> {
        match self {
            VdfValue::Map(m) => Some(m),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&String> {
        match self {
            VdfValue::String(m) => Some(m),
            _ => None,
        }
    }

    pub fn copy_as_str(&self) -> Option<String> {
        match self {
            VdfValue::String(m) => Some(m.clone()),
            VdfValue::Number(n) => Some(n.to_string()),
            _ => None,
        }
    }

    pub fn as_number(&self) -> Option<&u32> {
        match self {
            VdfValue::Number(m) => Some(m),
            _ => None,
        }
    }
}

pub type VdfMap = HashMap<String, VdfValue>;

#[derive(Debug)]
enum VdfMapItemType {
    Map = 0x00,
    String = 0x01,
    Number = 0x02,
    MapEnd = 0x08,
}

fn read_cstring(cursor: &mut Cursor<Vec<u8>>) -> io::Result<String> {
    let mut buf = Vec::new();
    let mut byte = [0u8; 1];
    loop {
        cursor.read_exact(&mut byte)?;
        if byte[0] == 0 {
            break;
        }
        buf.push(byte[0]);
    }
    Ok(String::from_utf8_lossy(&buf).to_string())
}

fn next_map_item(cursor: &mut Cursor<Vec<u8>>) -> io::Result<Option<(String, VdfValue)>> {
    let mut type_byte = [0u8; 1];
    if cursor.read_exact(&mut type_byte).is_err() {
        return Ok(None);
    }
    match type_byte[0] {
        x if x == VdfMapItemType::MapEnd as u8 => Ok(None),
        x if x == VdfMapItemType::Map as u8 => {
            let name = read_cstring(cursor)?.to_lowercase();
            let value = next_map(cursor)?;
            Ok(Some((name, VdfValue::Map(value))))
        }
        x if x == VdfMapItemType::String as u8 => {
            let name = read_cstring(cursor)?.to_lowercase();
            let value = read_cstring(cursor)?;
            Ok(Some((name, VdfValue::String(value))))
        }
        x if x == VdfMapItemType::Number as u8 => {
            let name = read_cstring(cursor)?.to_lowercase();
            let mut buf = [0u8; 4];
            cursor.read_exact(&mut buf)?;
            let value = u32::from_le_bytes(buf);
            Ok(Some((name, VdfValue::Number(value))))
        }
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Unknown type byte",
        )),
    }
}

fn next_map(cursor: &mut Cursor<Vec<u8>>) -> io::Result<VdfMap> {
    let mut map = VdfMap::new();
    loop {
        match next_map_item(cursor)? {
            Some((name, value)) => {
                map.insert(name, value);
            }
            None => break,
        }
    }
    Ok(map)
}

pub fn read_vdf(data: Vec<u8>) -> io::Result<VdfMap> {
    let mut cursor = Cursor::new(data);
    next_map(&mut cursor)
}
