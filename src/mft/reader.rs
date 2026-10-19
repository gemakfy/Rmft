use std::io::{BufReader, Read, Seek, SeekFrom};

use crate::mft::boot_sector::NtfsBootSector;
use crate::mft::data_run::{DataRun, decode_data_runs};
use crate::mft::header::RECORD_SIZE;
use crate::mft::{File, MftEntry, Path};
use crate::utils::{read_u16, read_u32};

pub fn read_mft_from_device(device_path: &Path) -> std::io::Result<Vec<MftEntry>> {
    let mut file = File::open(device_path)?;
    let mut boot_buffer = [0u8; 512];
    file.read_exact(&mut boot_buffer)?;

    let boot = NtfsBootSector::from_slice(&boot_buffer).ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Not a valid NTFS partition",
        )
    })?;

    let mft_offset = boot.mft_byte_offset();
    let cluster_size = boot.cluster_size();

    // Read Record 0 $MFT to find data runs
    file.seek(SeekFrom::Start(mft_offset))?;
    let mut rec0_buf = [0u8; RECORD_SIZE];
    file.read_exact(&mut rec0_buf)?;

    let runs = extract_mft_data_runs(&rec0_buf);

    let mut records = Vec::new();
    let mut buffer = [0u8; RECORD_SIZE];
    let mut reader = BufReader::with_capacity(1024 * 1024, file);

    if !runs.is_empty() {
        // Read through all fragmented extents of the MFT
        for run in runs {
            let run_byte_offset = run.lcn * cluster_size;
            let records_in_run = (run.length * cluster_size) / RECORD_SIZE as u64;

            if reader.seek(SeekFrom::Start(run_byte_offset)).is_err() {
                break;
            }

            for _ in 0..records_in_run {
                if reader.read_exact(&mut buffer).is_err() {
                    break;
                }
                if let Some(entry) = MftEntry::from_buffer(&buffer) {
                    records.push(entry);
                }
            }
        }
    } else {
        // Fallback: sequential read with streak cutoff
        reader.seek(SeekFrom::Start(mft_offset))?;
        let mut empty_streak = 0;

        while reader.read_exact(&mut buffer).is_ok() {
            if let Some(entry) = MftEntry::from_buffer(&buffer) {
                records.push(entry);
                empty_streak = 0;
            } else {
                empty_streak += 1;
                if empty_streak > 1000 {
                    break;
                }
            }
        }
    }

    Ok(records)
}

fn extract_mft_data_runs(rec0: &[u8; RECORD_SIZE]) -> Vec<DataRun> {
    if &rec0[0..4] != b"FILE" {
        return Vec::new();
    }

    let first_attr_offset = read_u16(rec0, 20) as usize;
    let mut offset = first_attr_offset;

    while offset + 8 <= rec0.len() {
        let type_code = read_u32(rec0, offset);
        let attr_len = read_u32(rec0, offset + 4) as usize;

        if type_code == 0xFFFF_FFFF || attr_len == 0 || offset + attr_len > rec0.len() {
            break;
        }

        // 0x80 is the $DATA attribute
        if type_code == 0x80 {
            let is_non_resident = rec0[offset + 8] != 0;
            if is_non_resident && attr_len >= 0x40 {
                let run_offset = read_u16(rec0, offset + 32) as usize;
                if run_offset < attr_len {
                    let run_slice = &rec0[offset + run_offset..offset + attr_len];
                    return decode_data_runs(run_slice);
                }
            }
            break;
        }

        offset += attr_len;
    }

    Vec::new()
}
