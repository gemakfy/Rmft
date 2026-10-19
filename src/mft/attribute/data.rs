//! $DATA attribute parser (type code 0x80).

use crate::utils::{read_u16, read_u32};

/// Parsed $DATA attribute containing resident file content.
#[derive(Debug, Clone)]
pub struct DataAttribute {
    pub data: Vec<u8>,
}

impl DataAttribute {
    /// Parses a resident $DATA attribute from an attribute buffer slice.
    pub fn from_slice(slice: &[u8]) -> Option<Self> {
        if slice.len() < 24 || slice[8] != 0 {
            // 0x08: non-resident flag (0 = resident)
            return None;
        }

        let data_size = read_u32(slice, 16) as usize; // 0x10: resident payload size
        let data_offset = read_u16(slice, 20) as usize; // 0x14: resident payload offset

        if slice.len() < data_offset + data_size {
            return None;
        }

        let data_slice = &slice[data_offset..data_offset + data_size];
        let data = data_slice.to_vec();

        Some(DataAttribute { data })
    }

    pub fn to_string_lossy(&self) -> String {
        String::from_utf8_lossy(&self.data).to_string()
    }

    pub fn to_safe_string(&self) -> String {
        self.data
            .iter()
            .map(|&b| {
                if b.is_ascii_graphic() || b == b' ' {
                    b as char
                } else {
                    '.'
                }
            })
            .collect()
    }
}
