use eframe::egui;

use crate::gui::app::MftApp;
use crate::mft::{parse_mft_file, read_mft_from_device};

pub fn render_welcome_screen(app: &mut MftApp, ui: &mut egui::Ui) {
    ui.vertical_centered(|ui| {
        ui.add_space(20.0);
        ui.heading(egui::RichText::new("NTFS MFT Explorer").size(26.0).strong());
        ui.label(
            egui::RichText::new("Choose a data source to begin forensic file system analysis")
                .color(egui::Color32::from_gray(160)),
        );
        ui.add_space(15.0);
    });

    if let Some(err) = &app.error_message {
        ui.vertical_centered(|ui| {
            ui.label(egui::RichText::new(err).color(egui::Color32::from_rgb(240, 80, 80)));
            ui.add_space(10.0);
        });
    }

    let available_width = ui.available_width();
    let available_height = ui.available_height() - 20.0;
    let col_width = (available_width - 30.0) / 2.0;

    ui.horizontal(|ui| {
        // Left Half: Offline MFT Dump
        ui.allocate_ui_with_layout(
            egui::vec2(col_width, available_height),
            egui::Layout::top_down(egui::Align::Center),
            |ui| {
                ui.add_space(available_height * 0.28);
                ui.heading(egui::RichText::new("Offline MFT Dump").size(24.0).strong());
                ui.add_space(10.0);
                ui.label(
                    egui::RichText::new(
                        "Analyze an extracted $MFT file or raw partition dump (.bin, .mft)",
                    )
                    .color(egui::Color32::from_gray(160)),
                );
                ui.add_space(28.0);

                let btn = egui::Button::new(egui::RichText::new("Browse Dump File...").size(16.0))
                    .min_size(egui::vec2(240.0, 48.0));

                if ui.add(btn).clicked()
                    && let Some(path) = rfd::FileDialog::new().pick_file()
                {
                    app.records = parse_mft_file(&path);
                    app.error_message = None;
                }
            },
        );

        ui.add_sized(
            [1.0, available_height],
            egui::Separator::default().vertical(),
        );

        // Right Half: Live Disk / Partition
        ui.allocate_ui_with_layout(
            egui::vec2(col_width, available_height),
            egui::Layout::top_down(egui::Align::Center),
            |ui| {
                ui.add_space(available_height * 0.22);
                ui.heading(
                    egui::RichText::new("Live Disk / Partition")
                        .size(24.0)
                        .strong(),
                );
                ui.add_space(10.0);
                ui.label(
                    egui::RichText::new(
                        "Read directly from a live block device or drive partition",
                    )
                    .color(egui::Color32::from_gray(160)),
                );
                ui.add_space(24.0);

                ui.label(egui::RichText::new("Device Path:").strong());
                ui.add_space(6.0);
                ui.add(
                    egui::TextEdit::singleline(&mut app.device_path)
                        .hint_text("/dev/sda2 or \\\\.\\C:")
                        .desired_width(260.0),
                );
                ui.add_space(24.0);

                let btn = egui::Button::new(egui::RichText::new("Read Partition").size(16.0))
                    .min_size(egui::vec2(240.0, 48.0));

                if ui.add(btn).clicked() {
                    match read_mft_from_device(std::path::Path::new(&app.device_path)) {
                        Ok(records) => {
                            app.records = records;
                            app.error_message = None;
                        }
                        Err(err) => {
                            app.error_message = Some(format!("Error: {}", err));
                        }
                    }
                }
            },
        );
    });
}
