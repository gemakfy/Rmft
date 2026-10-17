pub fn read_u16(slice: &[u8], offset: usize) -> u16 {
    let slice: [u8; 2] = _slice[_offset.._offset + 2].try_into().unwrap();

    u16::from_le_bytes(slice)
}

pub fn read_u32(_slice: &[u8], _offset: usize) -> u32 {
    let slice: [u8; 4] = _slice[_offset.._offset + 4].try_into().unwrap();

    u32::from_le_bytes(slice)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_u16() {
        let buffer = [0x00, 0x38, 0x01, 0x00];
        // 0x38, 0x01 в Little-Endian = 0x0138 (312 в dec)
        assert_eq!(read_u16(&buffer, 1), 0x0138);
    }

    #[test]
    fn test_read_u32() {
        let buffer = [0x00, 0x04, 0x00, 0x00];
        // 0x00, 0x04, 0x00, 0x00 в Little-Endian = 1024
        assert_eq!(read_u32(&buffer, 0), 1024);
    }
}
