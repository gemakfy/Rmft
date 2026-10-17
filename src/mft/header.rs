//! Заголовок записи MFT (первые 48 байт).
//!
//! Карта первых 48 байт (Header):
//! | Смещение | Размер | Тип Rust | Описание |
//! |----------|--------|----------|----------|
//! | 0..4     | 4 B    | &[u8]    | Сигнатура: "FILE" или "BAAD" |
//! | 4..6     | 2 B    | u16      | Fixup offset |
//! | 6..8     | 2 B    | u16      | Fixup count |
//! | 16..18   | 2 B    | u16      | Sequence Number (число переиспользований) |
//! | 20..22   | 2 B    | u16      | Смещение к первому атрибуту (обычно 0x38 = 56) |
//! | 22..24   | 2 B    | u16      | Флаги: 0x01 = ALIVE, 0x02 = DIRECTORY |
//! | 24..28   | 4 B    | u32      | Реальный используемый размер записи |
//! | 44..48   | 4 B    | u32      | Номер записи (Record / Inode Number) |

pub const RECORD_SIZE: usize = 1024;

#[derive(Debug, Clone)]
pub struct MftHeader {
    pub record_number: u32,
    pub flags: u16,
    pub first_attr_offset: u16,
    pub used_size: u32,
}

impl MftHeader {
    /// Принимает срез (slice) записи ровно на 1024 байта
    pub fn from_slice(record: &[u8]) -> Option<Self> {
        // 1. Проверяем сигнатуру "FILE"
        if record.len() < 48 || &record[0..4] != b"FILE" {
            return None;
        }

        // 2. Читаем числа Little-Endian.
        // Задание: когда реализуешь utils::read_u16 и utils::read_u32,
        // замени эти строки на вызовы функций из crate::utils!
        let first_attr_offset = read_u16(record, 20);
        let flags = read_u16(record, 22);
        let used_size = read_u32(record, 24);
        let record_number = read_u32(record, 44);

        Some(MftHeader {
            record_number,
            flags,
            first_attr_offset,
            used_size,
        })
    }

    /// Проверяет, активен ли файл (не удален)
    pub fn is_in_use(&self) -> bool {
        (self.flags & 0x01) != 0
    }

    /// Проверяет, является ли запись каталогом
    pub fn is_directory(&self) -> bool {
        (self.flags & 0x02) != 0
    }
}
