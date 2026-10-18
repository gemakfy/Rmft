use eframe::egui;

use crate::mft::{parse_mft_file, MftEntry};

/// State of the MFT Explorer GUI application.
pub struct MftApp {
    pub title: String,
    pub search_query: String,
    pub deleted_only: bool,
    pub records: Vec<MftEntry>,
    pub selected_record: Option<usize>,
}

impl Default for MftApp {
    fn default() -> Self {
        Self {
            title: "NTFS MFT Explorer".to_string(),
            search_query: String::new(),
            deleted_only: false,
            records: Vec::new(),
            selected_record: None,
        }
    }
}

impl MftApp {
    /// Renders the top toolbar: file picker, search input, deleted-only filter, and record count.
    fn render_top_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading(&self.title);
        ui.separator();

        ui.horizontal(|ui| {
            if ui.button("Open MFT Dump").clicked()
                && let Some(path) = rfd::FileDialog::new().pick_file()
            {
                self.records = parse_mft_file(&path);
            }

            ui.separator();

            ui.label("Search:");
            ui.text_edit_singleline(&mut self.search_query);
            ui.checkbox(&mut self.deleted_only, "Deleted only");

            ui.separator();
            ui.label(format!("Loaded: {} records", self.records.len()));
        });
    }

    /// Renders the left side panel displaying all metadata and resident content for the selected record.
    fn render_details_panel(&self, ui: &mut egui::Ui) {
        ui.heading("Record Details");
        ui.separator();

        if let Some(selected_id) = self.selected_record
            && let Some(record) = self
                .records
                .iter()
                .find(|r| r.record_number() as usize == selected_id)
        {
            ui.label(format!("Name: {}", record.name().unwrap_or("[No Name]")));
            ui.label(format!("Record ID: #{}", record.record_number()));
            ui.label(format!("Created: {}", record.created_time()));
            ui.label(format!("Modified: {}", record.modified_time()));
            ui.label(format!("Accessed: {}", record.accessed_time()));
            ui.label(format!("MFT modified: {}", record.mft_time()));

            ui.separator();

            if let Some(data_attr) = &record.data {
                ui.strong("Resident Data Content:");
                let text = String::from_utf8_lossy(&data_attr.data);
                ui.monospace(text);
            } else {
                ui.label("No resident data (file is empty or non-resident).");
            }
        } else {
            ui.label("Click any record in the table to inspect it.");
        }
    }

    /// Renders the central scrollable table containing the list of parsed MFT records.
    fn render_records_table(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::both().auto_shrink(false).show(ui, |ui| {
            egui::Grid::new("mft_records_grid")
                .striped(true)
                .spacing([15.0, 4.0])
                .show(ui, |ui| {
                    // Table header
                    ui.strong("ID");
                    ui.strong("Status");
                    ui.strong("Created");
                    ui.strong("Modified");
                    ui.strong("MFT Mod");
                    ui.strong("Accessed");
                    ui.strong("Name");
                    ui.end_row();

                    // Render matching rows
                    for record in &self.records {
                        let name = record.name().unwrap_or("[No Name]");
                        let is_alive = record.is_in_use();

                        // Filter by deleted flag
                        if self.deleted_only && is_alive {
                            continue;
                        }

                        // Filter by search query (name or ID)
                        if !self.search_query.is_empty() {
                            let query = self.search_query.to_lowercase();
                            let matches_name = name.to_lowercase().contains(&query);
                            let matches_id =
                                record.record_number().to_string() == self.search_query.trim();
                            if !matches_name && !matches_id {
                                continue;
                            }
                        }

                        let status_text = if is_alive {
                            egui::RichText::new("ALIVE").color(egui::Color32::from_rgb(80, 200, 80))
                        } else {
                            egui::RichText::new("DELETED").color(egui::Color32::from_rgb(240, 80, 80))
                        };

                        let is_selected =
                            self.selected_record == Some(record.record_number() as usize);
                        if ui
                            .selectable_label(
                                is_selected,
                                format!("#{:04}", record.record_number()),
                            )
                            .clicked()
                        {
                            self.selected_record = Some(record.record_number() as usize);
                        }

                        ui.label(status_text);
                        ui.monospace(record.created_time());
                        ui.monospace(record.modified_time());
                        ui.monospace(record.mft_time());
                        ui.monospace(record.accessed_time());
                        ui.label(name);
                        ui.end_row();
                    }
                });
        });
    }
}

impl eframe::App for MftApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.render_top_panel(ui);
        ui.separator();

        egui::Panel::left("details_panel")
            .default_size(280.0)
            .show(ui, |ui| {
                self.render_details_panel(ui);
            });

        egui::CentralPanel::default().show(ui, |ui| {
            self.render_records_table(ui);
        });
    }
}
