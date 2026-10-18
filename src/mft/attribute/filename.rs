//! $FILE_NAME attribute parser (type code 0x30).

use crate::utils::{read_u16, read_u64};

pub const FILE_NAME_DATA_OFFSET: usize = 66;

/// Parsed $FILE_NAME attribute containing the name and parent record number.
#[derive(Debug, Clone)]
pub struct FileNameAttribute {
    pub name: String,
    pub parent_record: u64,
}

impl FileNameAttribute {
    /// Parses a $FILE_NAME attribute from an attribute buffer slice.
    pub fn from_slice(slice: &[u8]) -> Option<Self> {
        if slice.len() < 22 {
            return None;
        }

        let content_offset = read_u16(slice, 20) as usize;
        if slice.len() < content_offset + FILE_NAME_DATA_OFFSET {
            return None;
        }

        let content = &slice[content_offset..];
        let parent_record = read_u64(&content[..8], 0);
        let name_len = content[64] as usize;

        if content.len() < FILE_NAME_DATA_OFFSET + name_len * 2 {
            return None;
        }

        let name_bytes = &content[FILE_NAME_DATA_OFFSET..];
        let u16_chars: Vec<u16> = name_bytes[..name_len * 2]
            .chunks_exact(2)
            .map(|c| u16::from_le_bytes([c[0], c[1]]))
            .collect();

        let name = String::from_utf16_lossy(&u16_chars);

        Some(FileNameAttribute {
            name,
            parent_record,
        })
    }
}
