use crate::utils::{read_u16, read_u64};
use smol_str::SmolStr;

pub const FILE_NAME_DATA_OFFSET: usize = 66;

#[derive(Debug, Clone)]
pub struct FileNameAttribute {
    pub name: SmolStr,
    pub parent_record: u64,
    pub created: u64,
    pub modified: u64,
    pub mft_modified: u64,
    pub accessed: u64,
    pub real_size: u64,
    pub allocated_size: u64,
    pub namespace: u8,
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
        let name_len = content[64] as usize;
        let namespace = content[65];

        let parent_record = read_u64(content, 0);
        let created = read_u64(content, 8);
        let modified = read_u64(content, 16);
        let mft_modified = read_u64(content, 24);
        let accessed = read_u64(content, 32);
        let allocated_size = read_u64(content, 40);
        let real_size = read_u64(content, 48);

        if content.len() < FILE_NAME_DATA_OFFSET + name_len * 2 {
            return None;
        }

        let name_bytes = &content[FILE_NAME_DATA_OFFSET..];

        let name_iter = name_bytes[..name_len * 2]
            .chunks_exact(2)
            .map(|c| u16::from_le_bytes([c[0], c[1]]));

        let name: SmolStr = char::decode_utf16(name_iter)
            .map(|r| r.unwrap_or(char::REPLACEMENT_CHARACTER))
            .collect();

        Some(FileNameAttribute {
            name,
            parent_record,
            created,
            modified,
            mft_modified,
            accessed,
            real_size,
            allocated_size,
            namespace,
        })
    }

    pub fn created_time(&self) -> String {
        super::standard_info::StandardInfoAttribute::format_filetime(self.created)
    }

    pub fn modified_time(&self) -> String {
        super::standard_info::StandardInfoAttribute::format_filetime(self.modified)
    }

    pub fn accessed_time(&self) -> String {
        super::standard_info::StandardInfoAttribute::format_filetime(self.accessed)
    }

    pub fn mft_time(&self) -> String {
        super::standard_info::StandardInfoAttribute::format_filetime(self.mft_modified)
    }
}
