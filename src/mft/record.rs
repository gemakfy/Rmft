use smol_str::SmolStr;

use super::attribute::{
    AttributeHeader, AttributeTypeCode, DataAttribute, FileNameAttribute, StandardInfoAttribute,
};
use super::header::{MftHeader, RECORD_SIZE};

#[derive(Debug, Clone)]
pub struct MftEntry {
    pub header: MftHeader,
    pub file_name: Option<FileNameAttribute>,
    pub dos_name: Option<SmolStr>,
    pub extension_name: Option<SmolStr>,
    pub data: Option<DataAttribute>,
    pub standard_info: Option<StandardInfoAttribute>,
    pub alternate_streams: Vec<AlternateDataStream>,
}

#[derive(Debug, Clone)]
pub struct AlternateDataStream {
    pub name: String,
    pub size: u64,
    pub data: Vec<u8>,
}

impl AlternateDataStream {
    pub fn to_safe_string(&self) -> String {
        crate::utils::bytes_to_safe_ascii(&self.data)
    }
}

fn apply_fixup(buffer: &mut [u8; RECORD_SIZE]) {
    let fixup_offset = crate::utils::read_u16(buffer, 4) as usize;
    let fixup_count = crate::utils::read_u16(buffer, 6) as usize;
    if fixup_count > 1 && fixup_offset + fixup_count * 2 <= buffer.len() {
        let update_seq = crate::utils::read_u16(buffer, fixup_offset);
        for i in 1..fixup_count {
            let sector_end = i * 512 - 2;
            if sector_end + 2 <= buffer.len() {
                let actual_seq = crate::utils::read_u16(buffer, sector_end);
                if actual_seq == update_seq {
                    buffer[sector_end] = buffer[fixup_offset + i * 2];
                    buffer[sector_end + 1] = buffer[fixup_offset + i * 2 + 1];
                }
            }
        }
    }
}

impl MftEntry {
    pub fn from_buffer(record: &mut [u8; RECORD_SIZE]) -> Option<Self> {
        apply_fixup(record);

        let header = MftHeader::from_slice(record)?;
        let mut file_name = None;
        let mut dos_name = None;
        let mut data = None;
        let mut standard_info = None;
        let mut alternate_streams = Vec::new();

        let mut offset = header.first_attr_offset as usize;
        while offset + 8 <= record.len() {
            if let Some(attr) = AttributeHeader::from_slice(&record[offset..]) {
                if attr.type_code == AttributeTypeCode::EndOfAttributes || attr.length == 0 {
                    break;
                }

                let attr_slice = &record[offset..];
                match attr.type_code {
                    AttributeTypeCode::FileName => {
                        if let Some(parsed_fn) = FileNameAttribute::from_slice(attr_slice) {
                            if parsed_fn.namespace == 2 {
                                dos_name = Some(parsed_fn.name.clone());
                            }
                            match &file_name {
                                None => file_name = Some(parsed_fn),
                                Some(current) => {
                                    if current.namespace == 2 && parsed_fn.namespace != 2 {
                                        file_name = Some(parsed_fn);
                                    }
                                }
                            }
                        }
                    }
                    AttributeTypeCode::Data => {
                        if attr.name_length == 0 {
                            if data.is_none() {
                                data = DataAttribute::from_slice(attr_slice);
                            }
                        } else {
                            let name_start = attr.name_offset as usize;
                            let name_end = name_start + (attr.name_length as usize) * 2;
                            if attr_slice.len() >= name_end {
                                let name_bytes = &attr_slice[name_start..name_end];
                                let stream_name: String = char::decode_utf16(
                                    name_bytes
                                        .as_chunks::<2>()
                                        .0
                                        .iter()
                                        .map(|c| u16::from_le_bytes([c[0], c[1]])),
                                )
                                .map(|r| r.unwrap_or(char::REPLACEMENT_CHARACTER))
                                .collect();

                                let stream_attr = DataAttribute::from_slice(attr_slice);
                                let (stream_data, stream_size) = match stream_attr {
                                    Some(d) => (d.data, d.real_size),
                                    None => (Vec::new(), 0),
                                };

                                alternate_streams.push(AlternateDataStream {
                                    name: stream_name,
                                    size: stream_size,
                                    data: stream_data,
                                });
                            }
                        }
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
            dos_name,
            extension_name: None,
            data,
            standard_info,
            alternate_streams,
        })
    }

    pub fn record_number(&self) -> u32 {
        self.header.record_number
    }

    pub fn is_in_use(&self) -> bool {
        self.header.is_in_use()
    }

    pub fn is_directory(&self) -> bool {
        self.header.is_directory()
    }

    pub fn name(&self) -> Option<&str> {
        if let Some(ref f) = self.file_name {
            Some(f.name.as_str())
        } else if let Some(ref ext) = self.extension_name {
            Some(ext.as_str())
        } else {
            None
        }
    }

    pub fn dos_name(&self) -> Option<&str> {
        self.dos_name.as_deref()
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

    pub fn created_raw(&self) -> u64 {
        self.standard_info.as_ref().map(|s| s.created).unwrap_or(0)
    }

    pub fn modified_time(&self) -> String {
        self.format_timestamp(|s| s.modified)
    }

    pub fn modified_raw(&self) -> u64 {
        self.standard_info.as_ref().map(|s| s.modified).unwrap_or(0)
    }

    pub fn accessed_time(&self) -> String {
        self.format_timestamp(|s| s.accessed)
    }

    pub fn accessed_raw(&self) -> u64 {
        self.standard_info.as_ref().map(|s| s.accessed).unwrap_or(0)
    }

    pub fn mft_time(&self) -> String {
        self.format_timestamp(|s| s.mft_modified)
    }

    pub fn file_size(&self) -> u64 {
        if let Some(ref data) = self.data {
            data.real_size
        } else {
            self.file_name.as_ref().map(|f| f.real_size).unwrap_or(0)
        }
    }

    pub fn alloc_size(&self) -> u64 {
        if let Some(ref data) = self.data {
            data.allocated_size
        } else {
            self.file_name
                .as_ref()
                .map(|f| f.allocated_size)
                .unwrap_or(0)
        }
    }

    pub fn formatted_size(&self) -> String {
        format_bytes(self.file_size(), self.is_directory())
    }

    pub fn formatted_alloc_size(&self) -> String {
        format_bytes(self.alloc_size(), self.is_directory())
    }

    pub fn ads_summary(&self) -> String {
        if self.alternate_streams.is_empty() {
            "-".to_string()
        } else {
            self.alternate_streams
                .iter()
                .map(|s| format!(":{}", s.name))
                .collect::<Vec<_>>()
                .join(", ")
        }
    }
}

pub fn format_bytes(size: u64, is_dir: bool) -> String {
    if is_dir {
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

pub fn resolve_extension_records(records: &mut [MftEntry]) {
    let mut base_names = std::collections::HashMap::new();

    for r in records.iter() {
        if !r.header.is_extension()
            && let Some(name) = r.name()
        {
            base_names.insert(r.record_number() as u64, SmolStr::new(name));
        }
    }

    for r in records.iter_mut() {
        if r.header.is_extension() && r.file_name.is_none() {
            let base_id = r.header.base_record_number();
            let label = if let Some(base_name) = base_names.get(&base_id) {
                format!("[Extension of #{}: {}]", base_id, base_name)
            } else {
                format!("[Extension of #{}]", base_id)
            };
            r.extension_name = Some(SmolStr::new(label));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mft::attribute::filename::FileNameAttribute;

    #[test]
    fn test_resolve_extension_records() {
        let base_header = MftHeader {
            record_number: 100,
            flags: 1,
            first_attr_offset: 56,
            used_size: 1024,
            base_record: 0,
        };
        let base_entry = MftEntry {
            header: base_header,
            file_name: Some(FileNameAttribute {
                name: SmolStr::new("game.iso"),
                parent_record: 5,
                created: 0,
                modified: 0,
                mft_modified: 0,
                accessed: 0,
                real_size: 10_000_000_000,
                allocated_size: 10_000_000_000,
                namespace: 1,
            }),
            dos_name: None,
            extension_name: None,
            data: None,
            standard_info: None,
            alternate_streams: Vec::new(),
        };

        let ext_header = MftHeader {
            record_number: 200,
            flags: 1,
            first_attr_offset: 56,
            used_size: 1024,
            base_record: 100, // Extension of #100
        };
        let ext_entry = MftEntry {
            header: ext_header,
            file_name: None,
            dos_name: None,
            extension_name: None,
            data: None,
            standard_info: None,
            alternate_streams: Vec::new(),
        };

        let mut records = vec![base_entry, ext_entry];
        assert_eq!(records[1].name(), None);

        resolve_extension_records(&mut records);

        assert_eq!(records[1].name(), Some("[Extension of #100: game.iso]"));
    }
}
