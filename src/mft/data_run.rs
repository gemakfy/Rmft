use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DataRun {
    pub lcn: u64,
    pub length: u64,
}

impl fmt::Display for DataRun {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "LCN: {}, Length: {} clusters", self.lcn, self.length)
    }
}

pub fn decode_data_runs(slice: &[u8]) -> Vec<DataRun> {
    let mut runs = Vec::new();
    let mut cursor = 0;
    let mut current_lcn: i64 = 0;

    while cursor < slice.len() {
        let header = slice[cursor];
        if header == 0x00 {
            break; // End of runlist
        }
        cursor += 1;

        let len_field_size = (header & 0x0F) as usize;
        let lcn_field_size = ((header >> 4) & 0x0F) as usize;

        if cursor + len_field_size + lcn_field_size > slice.len() {
            break; // Truncated runlist
        }

        // Read cluster length (unsigned little-endian)
        let mut length = 0u64;
        for i in 0..len_field_size {
            length |= (slice[cursor + i] as u64) << (i * 8);
        }
        cursor += len_field_size;

        // Read LCN delta (signed little-endian with sign extension)
        let mut lcn_delta = 0i64;
        if lcn_field_size > 0 {
            for i in 0..lcn_field_size {
                lcn_delta |= (slice[cursor + i] as i64) << (i * 8);
            }
            // Sign extend if highest bit of last byte is 1
            let last_byte = slice[cursor + lcn_field_size - 1];
            if (last_byte & 0x80) != 0 && lcn_field_size < 8 {
                let shift = lcn_field_size * 8;
                lcn_delta |= !0i64 << shift;
            }
            cursor += lcn_field_size;
        }

        current_lcn += lcn_delta;

        if current_lcn >= 0 && length > 0 {
            runs.push(DataRun {
                lcn: current_lcn as u64,
                length,
            });
        }
    }

    runs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_single_data_run() {
        // header: 0x33 -> 3 bytes length, 3 bytes LCN
        // len: 0x00c820 = 51232
        // lcn: 0x0c0000 = 786432
        let data = [0x33, 0x20, 0xc8, 0x00, 0x00, 0x00, 0x0c, 0x00];
        let runs = decode_data_runs(&data);
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].lcn, 786432);
        assert_eq!(runs[0].length, 51232);
    }

    #[test]
    fn test_decode_negative_lcn_delta() {
        // Run 1: len=10, lcn=1000
        // Run 2: len=5, lcn_delta = -100 -> new lcn = 900
        // 1000 = 0x03E8 (2 bytes: E8 03) -> header 0x21 (len: 1 byte, lcn: 2 bytes)
        // -100 = 0xFF9C (2 bytes: 9C FF) -> header 0x21
        let data = [0x21, 0x0A, 0xE8, 0x03, 0x21, 0x05, 0x9C, 0xFF, 0x00];
        let runs = decode_data_runs(&data);
        assert_eq!(runs.len(), 2);
        assert_eq!(runs[0].lcn, 1000);
        assert_eq!(runs[0].length, 10);
        assert_eq!(runs[1].lcn, 900);
        assert_eq!(runs[1].length, 5);
    }
}
