use eframe::egui;

use crate::gui::app::MftApp;

pub fn render_explorer_screen(app: &mut MftApp, ui: &mut egui::Ui) {
    egui::Panel::top("explorer_top_toolbar").show(ui, |ui| {
        render_top_toolbar(app, ui);
    });

    egui::Panel::left("explorer_details_panel")
        .default_size(300.0)
        .show(ui, |ui| {
            render_details_panel(app, ui);
        });

    render_records_table(app, ui);
}

fn render_top_toolbar(app: &mut MftApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        if ui.button("< Back").clicked() {
            app.records.clear();
            app.selected_record = None;
            app.error_message = None;
        }

        ui.separator();
        ui.label(egui::RichText::new(format!("Loaded: {} records", app.records.len())).strong());

        ui.separator();
        ui.label("Search:");
        ui.add(
            egui::TextEdit::singleline(&mut app.search_query).hint_text("Filter by name or #ID..."),
        );

        ui.checkbox(&mut app.deleted_only, "Deleted only");

        if let Some(err) = &app.error_message {
            ui.separator();
            ui.label(egui::RichText::new(err).color(egui::Color32::from_rgb(240, 80, 80)));
        }
    });
}

fn render_details_panel(app: &MftApp, ui: &mut egui::Ui) {
    ui.heading("Record Details");
    ui.separator();

    if let Some(selected_id) = app.selected_record
        && let Some(record) = app
            .records
            .iter()
            .find(|r| r.record_number() as usize == selected_id)
    {
        ui.label(format!("Name: {}", record.name().unwrap_or("[No Name]")));
        ui.label(format!("Record ID: #{}", record.record_number()));

        let status = if record.is_in_use() {
            egui::RichText::new("ALIVE").color(egui::Color32::from_rgb(80, 200, 80))
        } else {
            egui::RichText::new("DELETED").color(egui::Color32::from_rgb(240, 80, 80))
        };
        ui.horizontal(|ui| {
            ui.label("Status:");
            ui.label(status);
        });

        ui.label(format!(
            "Type: {}",
            if record.is_directory() {
                "Directory"
            } else {
                "File"
            }
        ));

        ui.separator();
        ui.strong("Timestamps $SI (UTC):");
        ui.label(format!("Created: {}", record.created_time()));
        ui.label(format!("Modified: {}", record.modified_time()));
        ui.label(format!("Accessed: {}", record.accessed_time()));
        ui.label(format!("MFT Modified: {}", record.mft_time()));
        ui.label(format!("File Size: {}", record.formatted_size()));
        ui.label(format!("Allocated Size: {}", record.formated_alloc_size()));

        ui.separator();

        if let Some(data_attr) = &record.data {
            ui.strong("Resident Data Content:");
            ui.monospace(data_attr.to_safe_string());
        } else {
            ui.label("No resident data (file is non-resident or empty).");
        }
    } else {
        ui.label("Click any record in the table to inspect details.");
    }
}

fn render_records_table(app: &mut MftApp, ui: &mut egui::Ui) {
    app.update_filtered_indices();

    let row_height = 20.0;
    let total_rows = app.filtered_indices.len();
    let filtered_indices = &app.filtered_indices;

    egui::ScrollArea::both().auto_shrink(false).show_rows(
        ui,
        row_height,
        total_rows,
        |ui, row_range| {
            egui::Grid::new("mft_records_grid")
                .striped(true)
                .spacing([15.0, 4.0])
                .show(ui, |ui| {
                    ui.strong("ID");
                    ui.strong("Status");
                    ui.strong("Created");
                    ui.strong("Modified");
                    ui.strong("MFT Mod");
                    ui.strong("Accessed");
                    ui.strong("Size");
                    ui.strong("Name");
                    ui.end_row();

                    for &idx in &filtered_indices[row_range] {
                        let record = &app.records[idx];
                        let name = record.name().unwrap_or("[No Name]");
                        let is_alive = record.is_in_use();

                        let status_text = if is_alive {
                            egui::RichText::new("ALIVE").color(egui::Color32::from_rgb(80, 200, 80))
                        } else {
                            egui::RichText::new("DELETED")
                                .color(egui::Color32::from_rgb(240, 80, 80))
                        };

                        let is_selected =
                            app.selected_record == Some(record.record_number() as usize);
                        if ui
                            .selectable_label(
                                is_selected,
                                format!("#{:04}", record.record_number()),
                            )
                            .clicked()
                        {
                            app.selected_record = Some(record.record_number() as usize);
                        }

                        ui.label(status_text);
                        ui.monospace(record.created_time());
                        ui.monospace(record.modified_time());
                        ui.monospace(record.mft_time());
                        ui.monospace(record.accessed_time());
                        ui.monospace(record.formatted_size());
                        ui.label(name);
                        ui.end_row();
                    }
                });
        },
    );
}
