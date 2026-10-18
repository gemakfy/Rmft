use crate::utils::{read_u16, read_u64};

#[derive(Debug, Clone)]
pub struct NtfsBootSector {
    pub bytes_per_sector: u16,
    pub sectors_per_cluster: u8,
    pub mft_lcn: u64,
}

impl NtfsBootSector {
    pub fn from_slice(slice: &[u8]) -> Option<Self> {
        if slice.len() < 512 {
            return None;
        }

        if &slice[3..11] != b"NTFS    " {
            return None;
        }

        let bytes_per_sector = read_u16(slice, 11); // 0x0B: bytes per sector
        let sectors_per_cluster = slice[13]; // 0x0D: sectors per cluster
        let mft_lcn = read_u64(slice, 48); // 0x30: starting cluster number of $MFT (LCN)

        Some(NtfsBootSector {
            bytes_per_sector,
            sectors_per_cluster,
            mft_lcn,
        })
    }

    /// Calculates the size of a single cluster in bytes.
    pub fn cluster_size(&self) -> u64 {
        (self.bytes_per_sector as u64) * (self.sectors_per_cluster as u64)
    }

    /// Calculates the byte offset where $MFT begins on the raw partition/disk.
    pub fn mft_byte_offset(&self) -> u64 {
        self.mft_lcn * self.cluster_size()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_boot_sector() {
        let mut sector = [0u8; 512];
        sector[3..11].copy_from_slice(b"NTFS    "); // 0x03: OEM ID
        sector[0x0B] = 0x00; // 0x0B: bytes_per_sector = 512 (0x0200)
        sector[0x0C] = 0x02;
        sector[0x0D] = 8; // 0x0D: sectors_per_cluster = 8
        sector[0x30] = 0x00; // 0x30: mft_lcn = 786432 (0x000C0000)
        sector[0x31] = 0x00;
        sector[0x32] = 0x0C;

        let boot = NtfsBootSector::from_slice(&sector).expect("Valid boot sector");
        assert_eq!(boot.bytes_per_sector, 512);
        assert_eq!(boot.sectors_per_cluster, 8);
        assert_eq!(boot.cluster_size(), 4096);
        assert_eq!(boot.mft_lcn, 786432);
        assert_eq!(boot.mft_byte_offset(), 3_221_225_472);
    }
}
