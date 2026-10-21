use mft::mft::header::{MftHeader, RECORD_SIZE};

#[test]
fn test_mft_header_parsing_on_dummy_record() {
    let mut record = [0u8; RECORD_SIZE];
    record[0..4].copy_from_slice(b"FILE");
    // first_attr_offset = 56 (0x38)
    record[20] = 0x38;
    record[21] = 0x00;
    // flags = 1 (ALIVE FILE)
    record[22] = 0x01;
    record[23] = 0x00;
    // used_size = 1024 (0x0400)
    record[24] = 0x00;
    record[25] = 0x04;
    record[26] = 0x00;
    record[27] = 0x00;
    // record_number = 100
    record[44] = 100;

    let header = MftHeader::from_slice(&record).expect("Header should parse successfully");
    assert_eq!(header.record_number, 100);
    assert_eq!(header.first_attr_offset, 56);
    assert!(header.is_in_use());
    assert!(!header.is_directory());
}

#[test]
fn test_parse_real_test_mft() {
    let test_mft_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("test.mft");
    let entries = mft::mft::parse_mft_file(&test_mft_path);
    assert_eq!(entries.len(), 66);

    // Verify Record 0 is $MFT
    let rec0 = &entries[0];
    assert_eq!(rec0.record_number(), 0);
    assert!(rec0.is_in_use());
    assert_eq!(rec0.name(), Some("$MFT"));

    // Verify Record 3 is $Volume
    let rec3 = &entries[3];
    assert_eq!(rec3.record_number(), 3);
    assert_eq!(rec3.name(), Some("$Volume"));
}

/// Optional hardware test: only runs if a live NTFS device is specified via `MFT_TEST_DEVICE`
/// or opportunistically if /dev/sda2 exists and is readable as NTFS.
/// Ignored by default during standard CI/reproducible test runs.
#[test]
#[ignore = "requires physical NTFS partition and root permissions; run with -- --ignored"]
fn test_read_mft_from_live_device() {
    let device_str = std::env::var("MFT_TEST_DEVICE").unwrap_or_else(|_| "/dev/sda2".to_string());
    let path = std::path::Path::new(&device_str);

    if !path.exists() {
        println!("Device {:?} does not exist, skipping live test.", path);
        return;
    }

    match mft::mft::read_mft_from_device(path) {
        Ok(entries) => {
            println!("Successfully read {} records from {:?}!", entries.len(), path);
            assert!(!entries.is_empty(), "MFT must contain at least one record");
            assert_eq!(entries[0].record_number(), 0);
            assert_eq!(entries[0].name(), Some("$MFT"));
        }
        Err(err) => {
            println!("Could not read {:?} ({}), skipping.", path, err);
        }
    }
}
