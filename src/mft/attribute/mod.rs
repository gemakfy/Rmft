//! NTFS attributes module.
//!
//! Every file record in MFT stores its data (timestamps, names, content)
//! as a sequence of attributes starting at `first_attr_offset`.
//! The chain of attributes terminates with marker `0xFFFFFFFF`.

use crate::utils::{read_u16, read_u32, read_u64};
use chrono::DateTime;

pub const FILE_NAME_DATA_OFFSET: usize = 66;
pub const FILETIME_TO_UNIX_OFFSET: u64 = 116_444_736_000_000_000;

#[derive(Debug, Clone)]
pub struct AttributeHeader {
    pub type_code: AttributeTypeCode,
    pub length: u32,
    pub non_resident: bool,
}

#[derive(Debug, Clone)]
pub struct FileNameAttribute {
    pub name: String,
    pub parent_record: u64,
}

#[derive(Debug, Clone)]
pub struct DataAttribute {
    pub data: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct StandardInfoAttribute {
    pub created: u64,
    pub modified: u64,
    pub mft_modified: u64,
    pub accessed: u64,
}

impl DataAttribute {
    /// Returns the data as a UTF-8 lossy string.
    pub fn to_string_lossy(&self) -> String {
        String::from_utf8_lossy(&self.data).to_string()
    }
}

impl AttributeHeader {
    pub fn from_slice(slice: &[u8]) -> Option<Self> {
        if slice.len() < 8 {
            return None;
        }

        let raw_type = read_u32(slice, 0);
        if raw_type == 0xFFFF_FFFF {
            return Some(AttributeHeader {
                type_code: AttributeTypeCode::EndOfAttributes,
                length: 0,
                non_resident: false,
            });
        }

        if slice.len() < 9 {
            return None;
        }

        let length = read_u32(slice, 4);
        let non_resident = slice[8] != 0;
        let type_code = AttributeTypeCode::from(raw_type);

        Some(AttributeHeader {
            type_code,
            length,
            non_resident,
        })
    }
}

impl FileNameAttribute {
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
        let mut u16_chars = Vec::with_capacity(name_len);

        for i in 0..name_len {
            let symbol = read_u16(name_bytes, i * 2);
            u16_chars.push(symbol);
        }

        let name = String::from_utf16_lossy(&u16_chars);

        Some(FileNameAttribute {
            name,
            parent_record,
        })
    }
}

impl DataAttribute {
    pub fn from_slice(slice: &[u8]) -> Option<Self> {
        if slice.len() < 24 || slice[8] != 0 {
            return None;
        }

        let data_size = read_u32(slice, 16) as usize;
        let data_offset = read_u16(slice, 20) as usize;

        if slice.len() < data_offset + data_size {
            return None;
        }

        let data_slice = &slice[data_offset..data_offset + data_size];
        let data = data_slice.to_vec();

        Some(DataAttribute { data })
    }
}

impl StandardInfoAttribute {
    pub fn from_slice(slice: &[u8]) -> Option<Self> {
        if slice.len() < 8 {
            return None;
        }

        let content_offset = read_u16(slice, 20) as usize;
        if slice.len() < content_offset + 32 {
            return None;
        }

        let content = &slice[content_offset..];

        let created = read_u64(content, 0);
        let modified = read_u64(content, 8);
        let mft_modified = read_u64(content, 16);
        let accessed = read_u64(content, 24);

        Some(StandardInfoAttribute {
            created,
            modified,
            mft_modified,
            accessed,
        })
    }

    pub fn format_filetime(filetime: u64) -> String {
        let unix_secs = filetime.saturating_sub(FILETIME_TO_UNIX_OFFSET) / 10_000_000;

        DateTime::from_timestamp(unix_secs as i64, 0)
            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
            .unwrap_or_else(|| "N/A".to_string())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttributeTypeCode {
    StandardInformation,
    AttributeList,
    FileName,
    ObjectId,
    SecurityDescriptor,
    VolumeName,
    VolumeInformation,
    Data,
    IndexRoot,
    IndexAllocation,
    Bitmap,
    ReparsePoint,
    EndOfAttributes,
    Unknown(u32),
}

impl From<u32> for AttributeTypeCode {
    fn from(val: u32) -> Self {
        match val {
            0x10 => AttributeTypeCode::StandardInformation,
            0x20 => AttributeTypeCode::AttributeList,
            0x30 => AttributeTypeCode::FileName,
            0x40 => AttributeTypeCode::ObjectId,
            0x50 => AttributeTypeCode::SecurityDescriptor,
            0x60 => AttributeTypeCode::VolumeName,
            0x70 => AttributeTypeCode::VolumeInformation,
            0x80 => AttributeTypeCode::Data,
            0x90 => AttributeTypeCode::IndexRoot,
            0xA0 => AttributeTypeCode::IndexAllocation,
            0xB0 => AttributeTypeCode::Bitmap,
            0xC0 => AttributeTypeCode::ReparsePoint,
            0xFFFF_FFFF => AttributeTypeCode::EndOfAttributes,
            other => AttributeTypeCode::Unknown(other),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_attribute_header() {
        let fake_bytes = [0x10, 0x00, 0x00, 0x00, 0x60, 0x00, 0x00, 0x00, 0x00];
        let header = AttributeHeader::from_slice(&fake_bytes).expect("Valid attribute header");
        assert_eq!(header.type_code, AttributeTypeCode::StandardInformation);
        assert_eq!(header.length, 96);
        assert!(!header.non_resident);
    }
}
