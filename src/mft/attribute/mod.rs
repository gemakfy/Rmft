//! Модуль для парсинга атрибутов MFT (задел на будущее).
//!
//! В NTFS данные файла хранятся в цепочке атрибутов (от first_attr_offset).
//! Список атрибутов всегда завершается маркером 0xFFFFFFFF.
//!
//! Основные коды типов атрибутов:
//! - 0x10 = $STANDARD_INFORMATION (даты MACB, флаги read-only/hidden)
//! - 0x20 = $ATTRIBUTE_LIST (если атрибуты не влезли в одну запись)
//! - 0x30 = $FILE_NAME (имя файла UTF-16, ссылка на родительскую папку)
//! - 0x80 = $DATA (содержимое файла: текст, исполняемый код и т.д.)
//! - 0xFFFFFFFF = маркер конца атрибутов

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
