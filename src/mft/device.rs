//! Block device discovery for Linux and Windows.

#[derive(Debug, Clone)]
pub struct DiscoveredDevice {
    pub path: String,
    pub label: String,
    pub is_ntfs: bool,
}

#[cfg(target_os = "linux")]
pub fn detect_devices() -> Vec<DiscoveredDevice> {
    use std::io::Read;

    let mut devices = Vec::new();
    let content = match std::fs::read_to_string("/proc/partitions") {
        Ok(c) => c,
        Err(_) => return devices,
    };

    for line in content.lines().skip(2) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 4 {
            continue;
        }

        let name = parts[3];
        if name.starts_with("loop") || name.starts_with("zram") {
            continue;
        }

        // Only take partition names (ending with digit)
        if !name.chars().last().map(|c| c.is_ascii_digit()).unwrap_or(false) {
            continue;
        }

        let blocks: u64 = parts[2].parse().unwrap_or(0);
        let bytes = blocks * 1024;
        let dev_path = format!("/dev/{}", name);

        let mut is_ntfs = false;
        if let Ok(mut f) = std::fs::File::open(&dev_path) {
            let mut boot = [0u8; 512];
            if f.read_exact(&mut boot).is_ok() && &boot[3..7] == b"NTFS" {
                is_ntfs = true;
            }
        }

        let size_str = crate::mft::format_bytes(bytes, false);
        let label = if is_ntfs {
            format!("{} (NTFS, {})", name, size_str)
        } else {
            format!("{} ({})", name, size_str)
        };

        devices.push(DiscoveredDevice {
            path: dev_path,
            label,
            is_ntfs,
        });
    }

    // Place verified NTFS partitions at the top
    devices.sort_by_key(|d| !d.is_ntfs);
    devices
}

#[cfg(target_os = "windows")]
pub fn detect_devices() -> Vec<DiscoveredDevice> {
    let mut devices = Vec::new();

    for letter in b'C'..=b'Z' {
        let ch = letter as char;
        let root = format!("{}:\\", ch);
        let path = format!(r"\\.\{}:", ch);

        if std::path::Path::new(&root).exists() {
            devices.push(DiscoveredDevice {
                path,
                label: format!("{}: Drive", ch),
                is_ntfs: true,
            });
        }
    }

    devices
}

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
pub fn detect_devices() -> Vec<DiscoveredDevice> {
    Vec::new()
}
