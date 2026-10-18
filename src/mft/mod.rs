//! MFT parsing module.

pub mod attribute;
pub mod boot_sector;
pub mod header;
pub mod record;

pub use header::{MftHeader, RECORD_SIZE};
pub use record::MftEntry;

use std::fs::File;
use std::io::Read;
use std::path::Path;

pub fn parse_mft_file(path: &Path) -> Vec<MftEntry> {
    let mut records = Vec::new();

    if let Ok(mut file) = File::open(path) {
        let mut buffer = [0u8; RECORD_SIZE];

        while file.read_exact(&mut buffer).is_ok() {
            if let Some(entry) = MftEntry::from_buffer(&buffer) {
                records.push(entry);
            }
        }
    }

    records
}
