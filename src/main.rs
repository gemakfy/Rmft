use std::env;
use std::fs::File;
use std::io::{self, Read};

use mft::mft::header::{MftHeader, RECORD_SIZE};

fn main() -> io::Result<()> {
    // Принимаем путь из аргументов командной строки (например, cargo run -- test.mft)
    // Либо по умолчанию открываем test.mft
    let path = env::args().nth(1).unwrap_or_else(|| "test.mft".to_string());
    println!("Парсинг файла MFT: {}\n", path);

    let mut file = match File::open(&path) {
        Ok(f) => f,
        Err(err) => {
            eprintln!("Ошибка: не удалось открыть файл '{}': {}", path, err);
            eprintln!("Подсказка: создай test.mft с помощью скрипта scripts/create_test_mft.sh");
            return Err(err);
        }
    };

    let mut buffer = [0u8; RECORD_SIZE];
    let mut total_records = 0;
    let mut alive_records = 0;
    let mut deleted_records = 0;

    // Читаем файл кусками ровно по 1024 байта
    while file.read_exact(&mut buffer).is_ok() {
        if let Some(header) = MftHeader::from_slice(&buffer) {
            total_records += 1;
            if header.is_in_use() {
                alive_records += 1;
            } else {
                deleted_records += 1;
            }

            let status = if header.is_in_use() { "ALIVE" } else { "DELETED" };
            let kind = if header.is_directory() { "DIR " } else { "FILE" };

            println!(
                "Запись #{:<4} | {} | {} | Первый атрибут на байте: {}",
                header.record_number, status, kind, header.first_attr_offset
            );
        }
    }

    println!("\nВсего валидных записей: {} (Живых: {}, Удаленных: {})", total_records, alive_records, deleted_records);
    Ok(())
}
