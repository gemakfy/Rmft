use std::env;
use std::fs::File;
use std::io::{self, Read};

use mft::mft::MftEntry;
use mft::mft::attribute::StandardInfoAttribute;
use mft::mft::header::RECORD_SIZE;

fn main() -> io::Result<()> {
    let path = env::args().nth(1).unwrap_or_else(|| "test.mft".to_string());
    println!("Parsing MFT file: {}\n", path);

    let mut file = match File::open(&path) {
        Ok(f) => f,
        Err(err) => {
            eprintln!("Error opening file '{}': {}", path, err);
            return Err(err);
        }
    };

    let mut buffer = [0u8; RECORD_SIZE];
    let mut total_records = 0;
    let mut alive_records = 0;
    let mut deleted_records = 0;

    while file.read_exact(&mut buffer).is_ok() {
        if let Some(entry) = MftEntry::from_buffer(&buffer) {
            total_records += 1;
            if entry.is_in_use() {
                alive_records += 1;
            } else {
                deleted_records += 1;
            }

            let status = if entry.is_in_use() {
                "ALIVE"
            } else {
                "DELETED"
            };
            let kind = if entry.is_directory() { "DIR " } else { "FILE" };

            println!(
                "Record #{:<4} | {:<7} | {}",
                entry.record_number(),
                status,
                kind
            );

            if let Some(file_name) = &entry.file_name {
                println!(
                    "    └─ File name: \"{}\" (Parent record: {})",
                    file_name.name, file_name.parent_record
                );
            }

            if let Some(data) = &entry.data {
                let text = data.to_string_lossy();
                println!("    └─ Content: {:?}", text.trim());
            }

            if let Some(standart_info) = &entry.standard_info {
                let created = StandardInfoAttribute::format_filetime(standart_info.created);
                let modified = StandardInfoAttribute::format_filetime(standart_info.modified);
                let accessed = StandardInfoAttribute::format_filetime(standart_info.accessed);
                let mft_modified =
                    StandardInfoAttribute::format_filetime(standart_info.mft_modified);
                println!("    └─ Created:  {}", created);
                println!("    └─ Modified: {}", modified);
                println!("    └─ Accessed: {}", accessed);
                println!("    └─ MFT modified: {}", mft_modified);
            }
        }
    }

    println!(
        "\nTotal valid records: {} (Alive: {}, Deleted: {})",
        total_records, alive_records, deleted_records
    );

    Ok(())
}
