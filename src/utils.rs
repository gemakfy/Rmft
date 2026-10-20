//! Byte reading and conversion utilities for NTFS data structures.

pub fn read_u16(slice: &[u8], offset: usize) -> u16 {
    let bytes: [u8; 2] = slice[offset..offset + 2].try_into().unwrap();
    u16::from_le_bytes(bytes)
}

pub fn read_u32(slice: &[u8], offset: usize) -> u32 {
    let bytes: [u8; 4] = slice[offset..offset + 4].try_into().unwrap();
    u32::from_le_bytes(bytes)
}

pub fn read_u64(slice: &[u8], offset: usize) -> u64 {
    let bytes: [u8; 8] = slice[offset..offset + 8].try_into().unwrap();
    u64::from_le_bytes(bytes)
}

/// Converts raw binary bytes into a safe ASCII string, replacing control and null bytes with '.'.
pub fn bytes_to_safe_ascii(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|&b| {
            if b.is_ascii_graphic() || b == b' ' {
                b as char
            } else {
                '.'
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_u16() {
        let buffer = [0x00, 0x38, 0x01, 0x00];
        assert_eq!(read_u16(&buffer, 1), 0x0138);
    }

    #[test]
    fn test_read_u32() {
        let buffer = [0x00, 0x04, 0x00, 0x00];
        assert_eq!(read_u32(&buffer, 0), 1024);
    }

    #[test]
    fn test_bytes_to_safe_ascii() {
        let raw = b"Hello\x00World\x1F!";
        assert_eq!(bytes_to_safe_ascii(raw), "Hello.World.!");
    }
}
