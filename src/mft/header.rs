//! MFT record header (first 48 bytes of a record).
//!
//! Offset map:
//! | Offset  | Size | Type   | Description |
//! |---------|------|--------|-------------|
//! | 0..4    | 4 B  | &[u8]  | Signature: "FILE" or "BAAD" |
//! | 4..6    | 2 B  | u16    | Fixup array offset |
//! | 6..8    | 2 B  | u16    | Fixup count |
//! | 16..18  | 2 B  | u16    | Sequence number |
//! | 20..22  | 2 B  | u16    | First attribute offset (typically 0x38 = 56) |
//! | 22..24  | 2 B  | u16    | Flags: 0x01 = IN_USE, 0x02 = DIRECTORY |
//! | 24..28  | 4 B  | u32    | Used size of the record |
//! | 44..48  | 4 B  | u32    | Record / Inode number |

use crate::utils::{read_u16, read_u32, read_u64};

pub const RECORD_SIZE: usize = 1024;
pub const SIGNATURE_FILE: &[u8; 4] = b"FILE";

pub const FLAG_IN_USE: u16 = 0x0001;
pub const FLAG_DIRECTORY: u16 = 0x0002;

#[derive(Debug, Clone)]
pub struct MftHeader {
    pub record_number: u32,
    pub flags: u16,
    pub first_attr_offset: u16,
    pub used_size: u32,
    pub base_record: u64,
}

impl MftHeader {
    pub fn from_slice(record: &[u8]) -> Option<Self> {
        if record.len() < 48 || &record[0..4] != SIGNATURE_FILE {
            return None;
        }

        let first_attr_offset = read_u16(record, 20);
        let flags = read_u16(record, 22);
        let used_size = read_u32(record, 24);
        let base_record = read_u64(record, 32);
        let record_number = read_u32(record, 44);

        Some(MftHeader {
            record_number,
            flags,
            first_attr_offset,
            used_size,
            base_record,
        })
    }

    pub fn is_in_use(&self) -> bool {
        (self.flags & FLAG_IN_USE) != 0
    }

    pub fn is_directory(&self) -> bool {
        (self.flags & FLAG_DIRECTORY) != 0
    }

    pub fn is_extension(&self) -> bool {
        self.base_record != 0
    }

    pub fn base_record_number(&self) -> u64 {
        self.base_record & 0x0000_FFFF_FFFF_FFFF
    }
}
