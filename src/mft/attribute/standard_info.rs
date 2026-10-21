use crate::utils::{read_u16, read_u64};
use chrono::DateTime;

pub const FILETIME_TO_UNIX_OFFSET: u64 = 116_444_736_000_000_000;

#[derive(Debug, Clone)]
pub struct StandardInfoAttribute {
    pub created: u64,
    pub modified: u64,
    pub mft_modified: u64,
    pub accessed: u64,
}

impl StandardInfoAttribute {
    pub fn from_slice(slice: &[u8]) -> Option<Self> {
        if slice.len() < 8 {
            return None;
        }

        let content_offset = read_u16(slice, 20) as usize; // 0x14: offset to attribute content
        if slice.len() < content_offset + 32 {
            return None;
        }

        let content = &slice[content_offset..];

        let created = read_u64(content, 0); // 0x00: file creation time (FILETIME)
        let modified = read_u64(content, 8); // 0x08: last modification time (FILETIME)
        let mft_modified = read_u64(content, 16); // 0x10: MFT record modification time (FILETIME)
        let accessed = read_u64(content, 24); // 0x18: last access time (FILETIME)

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standard_info_timestamp_parsing() {
        // FILETIME for 2026-10-17 21:31:23 UTC: 134_000_000_000_000_000 approx
        let unix_time = 1_700_000_000u64;
        let filetime = unix_time * 10_000_000 + FILETIME_TO_UNIX_OFFSET;
        let formatted = StandardInfoAttribute::format_filetime(filetime);
        assert!(!formatted.is_empty());
        assert_ne!(formatted, "N/A");
    }
}
