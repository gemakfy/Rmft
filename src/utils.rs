//! Byte reading utilities for Little-Endian data slices.

/// Reads a 2-byte Little-Endian unsigned integer (u16) from `slice` at `offset`.
pub fn read_u16(slice: &[u8], offset: usize) -> u16 {
    let bytes: [u8; 2] = slice[offset..offset + 2].try_into().unwrap();
    u16::from_le_bytes(bytes)
}

/// Reads a 4-byte Little-Endian unsigned integer (u32) from `slice` at `offset`.
pub fn read_u32(slice: &[u8], offset: usize) -> u32 {
    let bytes: [u8; 4] = slice[offset..offset + 4].try_into().unwrap();
    u32::from_le_bytes(bytes)
}

pub fn read_u64(slice: &[u8], offset: usize) -> u64 {
    let bytes: [u8; 8] = slice[offset..offset + 8].try_into().unwrap();
    u64::from_le_bytes(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_u16() {
        let buffer = [0x00, 0x38, 0x01, 0x00];
        // 0x38, 0x01 in Little-Endian = 0x0138 (312 decimal)
        assert_eq!(read_u16(&buffer, 1), 0x0138);
    }

    #[test]
    fn test_read_u32() {
        let buffer = [0x00, 0x04, 0x00, 0x00];
        // 0x00, 0x04, 0x00, 0x00 in Little-Endian = 1024 decimal
        assert_eq!(read_u32(&buffer, 0), 1024);
    }
}
