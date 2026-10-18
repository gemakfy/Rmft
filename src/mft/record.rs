//! High-level representation of an MFT record and its parsed attributes.

use super::attribute::{
    AttributeHeader, AttributeTypeCode, DataAttribute, FileNameAttribute, StandardInfoAttribute,
};
use super::header::{MftHeader, RECORD_SIZE};

/// A parsed MFT file record containing its header and attributes.
#[derive(Debug, Clone)]
pub struct MftEntry {
    pub header: MftHeader,
    pub file_name: Option<FileNameAttribute>,
    pub data: Option<DataAttribute>,
    pub standard_info: Option<StandardInfoAttribute>,
}

impl MftEntry {
    /// Parses an MFT entry from a 1024-byte record buffer.
    pub fn from_buffer(buffer: &[u8; RECORD_SIZE]) -> Option<Self> {
        let header = MftHeader::from_slice(buffer)?;
        let mut file_name = None;
        let mut data = None;
        let mut standard_info = None;

        let mut offset = header.first_attr_offset as usize;
        while offset + 8 <= buffer.len() {
            if let Some(attr) = AttributeHeader::from_slice(&buffer[offset..]) {
                if attr.type_code == AttributeTypeCode::EndOfAttributes || attr.length == 0 {
                    break;
                }

                if attr.type_code == AttributeTypeCode::FileName && file_name.is_none() {
                    file_name = FileNameAttribute::from_slice(&buffer[offset..]);
                } else if attr.type_code == AttributeTypeCode::Data && data.is_none() {
                    data = DataAttribute::from_slice(&buffer[offset..]);
                } else if attr.type_code == AttributeTypeCode::StandardInformation
                    && standard_info.is_none()
                {
                    standard_info = StandardInfoAttribute::from_slice(&buffer[offset..]);
                }

                offset += attr.length as usize;
            } else {
                break;
            }
        }

        Some(MftEntry {
            header,
            file_name,
            data,
            standard_info,
        })
    }

    /// Returns the record number (inode).
    pub fn record_number(&self) -> u32 {
        self.header.record_number
    }

    /// Returns true if the record is currently in use (not deleted).
    pub fn is_in_use(&self) -> bool {
        self.header.is_in_use()
    }

    /// Returns true if the record is a directory.
    pub fn is_directory(&self) -> bool {
        self.header.is_directory()
    }

    /// Returns the file name if present.
    pub fn name(&self) -> Option<&str> {
        self.file_name.as_ref().map(|f| f.name.as_str())
    }
}
