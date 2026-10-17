//! Представление 1024-байтной записи MFT.
//! Включает в себя заголовок (MftHeader) и сырые байты атрибутов.

use super::header::{MftHeader, RECORD_SIZE};

#[derive(Debug, Clone)]
pub struct MftRecord {
    pub header: MftHeader,
    pub raw: [u8; RECORD_SIZE],
}

impl MftRecord {
    pub fn new(raw: [u8; RECORD_SIZE]) -> Option<Self> {
        let header = MftHeader::from_slice(&raw)?;
        Some(MftRecord { header, raw })
    }

    /// Срез с началом атрибутов (от first_attr_offset до конца записи)
    /// Это пригодится для парсинга атрибутов ($FILE_NAME, $DATA и т.д.)
    pub fn attributes_slice(&self) -> &[u8] {
        let offset = self.header.first_attr_offset as usize;
        &self.raw[offset..]
    }
}
