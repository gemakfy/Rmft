use crate::utils::{read_u16, read_u32, read_u64};

#[derive(Debug, Clone)]
pub struct DataAttribute {
    pub real_size: u64,
    pub allocated_size: u64,
    pub non_resident: bool,
    pub data: Vec<u8>,
}

impl DataAttribute {
    pub fn from_slice(slice: &[u8]) -> Option<Self> {
        if slice.len() < 16 {
            return None;
        }

        let non_resident = slice[8] != 0;

        if !non_resident {
            if slice.len() < 24 {
                return None;
            }

            let data_size = read_u32(slice, 16) as usize;
            let data_offset = read_u16(slice, 20) as usize;

            if slice.len() < data_offset + data_size {
                return None;
            }

            let data = slice[data_offset..data_offset + data_size].to_vec();

            Some(DataAttribute {
                real_size: data_size as u64,
                allocated_size: data_size as u64,
                non_resident: false,
                data,
            })
        } else {
            if slice.len() < 56 {
                return None;
            }

            let allocated_size = read_u64(slice, 40);
            let real_size = read_u64(slice, 48);

            Some(DataAttribute {
                real_size,
                allocated_size,
                non_resident: true,
                data: Vec::new(),
            })
        }
    }

    pub fn to_string_lossy(&self) -> String {
        String::from_utf8_lossy(&self.data).to_string()
    }

    pub fn to_safe_string(&self) -> String {
        crate::utils::bytes_to_safe_ascii(&self.data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resident_data() {
        let mut buffer = vec![0u8; 32];
        buffer[8] = 0; // resident
        buffer[16] = 5; // data_size = 5
        buffer[20] = 24; // data_offset = 24
        buffer[24..29].copy_from_slice(b"hello");

        let data = DataAttribute::from_slice(&buffer).expect("Should parse resident data");
        assert_eq!(data.real_size, 5);
        assert_eq!(data.allocated_size, 5);
        assert!(!data.non_resident);
        assert_eq!(data.data, b"hello");
    }

    #[test]
    fn test_non_resident_data() {
        let mut buffer = vec![0u8; 64];
        buffer[8] = 1; // non-resident
        buffer[40..48].copy_from_slice(&4096u64.to_le_bytes()); // allocated_size
        buffer[48..56].copy_from_slice(&1234u64.to_le_bytes()); // real_size

        let data = DataAttribute::from_slice(&buffer).expect("Should parse non-resident data");
        assert_eq!(data.real_size, 1234);
        assert_eq!(data.allocated_size, 4096);
        assert!(data.non_resident);
        assert!(data.data.is_empty());
    }
}
