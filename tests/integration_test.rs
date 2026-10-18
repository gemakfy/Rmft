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
    let entries = mft::mft::parse_mft_file(std::path::Path::new("test.mft"));
    println!("Parsed {} entries from test.mft", entries.len());
    assert_eq!(entries.len(), 66);
}

