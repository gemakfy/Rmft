use crate::utils::{read_u16, read_u32};

#[derive(Debug, Clone)]
pub struct AttributeHeader {
    pub type_code: AttributeTypeCode,
    pub length: u32,
    pub non_resident: bool,
    pub name_length: u8,
    pub name_offset: u16,
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
                name_length: 0,
                name_offset: 0,
            });
        }

        if slice.len() < 12 {
            return None;
        }

        let name_length = slice[9];
        let name_offset = read_u16(slice, 10);
        let length = read_u32(slice, 4);
        let non_resident = slice[8] != 0;
        let type_code = AttributeTypeCode::from(raw_type);

        Some(AttributeHeader {
            type_code,
            length,
            non_resident,
            name_length,
            name_offset,
        })
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
        let fake_bytes = [
            0x10, 0x00, 0x00, 0x00, 0x60, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        let header = AttributeHeader::from_slice(&fake_bytes).expect("Valid attribute header");
        assert_eq!(header.type_code, AttributeTypeCode::StandardInformation);
        assert_eq!(header.length, 96);
        assert!(!header.non_resident);
    }
}
