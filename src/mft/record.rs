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

                let attr_slice = &buffer[offset..];
                match attr.type_code {
                    AttributeTypeCode::FileName if file_name.is_none() => {
                        file_name = FileNameAttribute::from_slice(attr_slice);
                    }
                    AttributeTypeCode::Data if data.is_none() => {
                        data = DataAttribute::from_slice(attr_slice);
                    }
                    AttributeTypeCode::StandardInformation if standard_info.is_none() => {
                        standard_info = StandardInfoAttribute::from_slice(attr_slice);
                    }
                    _ => {}
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

    fn format_timestamp(&self, select: impl FnOnce(&StandardInfoAttribute) -> u64) -> String {
        self.standard_info
            .as_ref()
            .map(|s| StandardInfoAttribute::format_filetime(select(s)))
            .unwrap_or_else(|| "N/A".to_string())
    }

    pub fn created_time(&self) -> String {
        self.format_timestamp(|s| s.created)
    }

    pub fn modified_time(&self) -> String {
        self.format_timestamp(|s| s.modified)
    }

    pub fn accessed_time(&self) -> String {
        self.format_timestamp(|s| s.accessed)
    }

    pub fn mft_time(&self) -> String {
        self.format_timestamp(|s| s.mft_modified)
    }

    pub fn file_size(&self) -> u64 {
        self.file_name.as_ref().map(|f| f.real_size).unwrap_or(0)
    }

    pub fn alloc_size(&self) -> u64 {
        self.file_name
            .as_ref()
            .map(|f| f.allocated_size)
            .unwrap_or(0)
    }

    pub fn formatted_size(&self) -> String {
        let size = self.file_size();
        if self.is_directory() {
            return "-".to_string();
        }

        if size >= 1024 * 1024 * 1024 {
            format!("{:.2} GB", size as f64 / (1024.0 * 1024.0 * 1024.0))
        } else if size >= 1024 * 1024 {
            format!("{:.2} MB", size as f64 / (1024.0 * 1024.0))
        } else if size >= 1024 {
            format!("{:.2} KB", size as f64 / 1024.0)
        } else {
            format!("{} B", size)
        }
    }

    pub fn formated_alloc_size(&self) -> String {
        let size = self.alloc_size();
        if self.is_directory() {
            return "-".to_string();
        }

        if size >= 1024 * 1024 * 1024 {
            format!("{:.2} GB", size as f64 / (1024.0 * 1024.0 * 1024.0))
        } else if size >= 1024 * 1024 {
            format!("{:.2} MB", size as f64 / (1024.0 * 1024.0))
        } else if size >= 1024 {
            format!("{:.2} KB", size as f64 / 1024.0)
        } else {
            format!("{} B", size)
        }
    }
}
