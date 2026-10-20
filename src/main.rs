#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use mft::gui::MftApp;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();

    eframe::run_native(
        "NTFS MFT Explorer",
        options,
        Box::new(|_cc| Ok(Box::new(MftApp::default()))),
    )
}
