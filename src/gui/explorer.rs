use eframe::egui;
use egui_extras::{Column, TableBuilder};

use crate::gui::app::MftApp;
use crate::gui::app::{SortColumn, SortDirection};

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
            app.selected_cell = None;
            app.copy_notification = None;
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

        if let Some((msg, time)) = &app.copy_notification
            && time.elapsed().as_secs_f32() < 2.5
        {
            ui.separator();
            ui.label(
                egui::RichText::new(msg)
                    .color(egui::Color32::from_rgb(100, 220, 120))
                    .strong(),
            );
        }

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
        if let Some(dos_name) = record.dos_name() {
            ui.label(format!("DOS Name (8.3): {}", dos_name));
        }
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

        let type_desc = if record.header.is_extension() {
            format!("Extension of #{}", record.header.base_record_number())
        } else if record.is_directory() {
            "Directory".to_string()
        } else {
            "File".to_string()
        };
        ui.label(format!("Type: {}", type_desc));

        ui.separator();
        ui.strong("Sizes:");

        if let Some(ref data) = record.data {
            ui.label(format!(
                "$DATA Real: {}",
                crate::mft::format_bytes(data.real_size, record.is_directory())
            ));
            ui.label(format!(
                "$DATA Alloc: {}",
                crate::mft::format_bytes(data.allocated_size, record.is_directory())
            ));
        } else {
            ui.label("$DATA: None");
        }

        if let Some(ref fn_attr) = record.file_name {
            ui.label(format!(
                "$FN Real: {}",
                crate::mft::format_bytes(fn_attr.real_size, record.is_directory())
            ));
            ui.label(format!(
                "$FN Alloc: {}",
                crate::mft::format_bytes(fn_attr.allocated_size, record.is_directory())
            ));
        }

        if !record.alternate_streams.is_empty() {
            ui.separator();
            ui.strong("Alternate Data Streams (ADS):");
            for stream in &record.alternate_streams {
                ui.label(format!(
                    ":{} ({})",
                    stream.name,
                    crate::mft::format_bytes(stream.size, false)
                ));
                if !stream.data.is_empty() {
                    ui.monospace(stream.to_safe_string());
                }
            }
        }

        ui.separator();
        ui.strong("Timestamps $SI (UTC):");
        ui.label(format!("Created: {}", record.created_time()));
        ui.label(format!("Modified: {}", record.modified_time()));
        ui.label(format!("Accessed: {}", record.accessed_time()));
        ui.label(format!("MFT Modified: {}", record.mft_time()));

        if let Some(ref fn_attr) = record.file_name {
            ui.separator();
            ui.strong("Timestamps $FN (UTC):");
            ui.label(format!("Created: {}", fn_attr.created_time()));
            ui.label(format!("Modified: {}", fn_attr.modified_time()));
            ui.label(format!("Accessed: {}", fn_attr.accessed_time()));
            ui.label(format!("MFT Modified: {}", fn_attr.mft_time()));
        }

        ui.separator();

        if let Some(data_attr) = &record.data {
            if !data_attr.data.is_empty() {
                ui.strong("Resident Data Content:");
                ui.monospace(data_attr.to_safe_string());
            } else if data_attr.non_resident {
                ui.label("Data is non-resident (stored in clusters).");
            }
        } else {
            ui.label("No resident data (file is non-resident or empty).");
        }
    } else {
        ui.label("Click any record in the table to inspect details.");
    }
}

fn render_records_table(app: &mut MftApp, ui: &mut egui::Ui) {
    app.update_filtered_indices();

    // Check Ctrl+C or Event::Copy shortcut for copying selected cell
    let copy_pressed = ui.input(|i| {
        i.events.iter().any(|e| matches!(e, egui::Event::Copy))
            || (i.modifiers.command && i.key_pressed(egui::Key::C))
    });
    if copy_pressed
        && let Some((rec_num, col_idx)) = app.selected_cell
        && let Some(record) = app
            .records
            .iter()
            .find(|r| r.record_number() as usize == rec_num)
    {
        let text = match col_idx {
            0 => format!("#{:04}", record.record_number()),
            1 => {
                if record.is_in_use() {
                    "ALIVE".to_string()
                } else {
                    "DELETED".to_string()
                }
            }
            2 => record.created_time(),
            3 => record.modified_time(),
            4 => record.accessed_time(),
            5 => record.formatted_size(),
            6 => record.name().unwrap_or("[No Name]").to_string(),
            7 => record.ads_summary(),
            _ => String::new(),
        };
        ui.ctx().copy_text(text.clone());
        app.copy_notification = Some((format!("Copied: \"{}\"", text), std::time::Instant::now()));
    }

    let row_height = 22.0;
    let total_rows = app.filtered_indices.len();
    let filtered_indices = &app.filtered_indices;
    let mut sort_clicked: Option<SortColumn> = None;
    let mut new_selected_cell: Option<(usize, usize)> = None;

    TableBuilder::new(ui)
        .striped(true)
        .resizable(true)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(Column::initial(70.0).range(50.0..=120.0)) // ID
        .column(Column::initial(80.0).range(60.0..=120.0)) // Status
        .column(Column::initial(150.0).range(120.0..=220.0)) // Created
        .column(Column::initial(150.0).range(120.0..=220.0)) // Modified
        .column(Column::initial(150.0).range(120.0..=220.0)) // Accessed
        .column(Column::initial(90.0).range(70.0..=160.0)) // Size
        .column(Column::remainder().at_least(160.0)) // Name
        .column(Column::initial(140.0).range(80.0..=260.0)) // ADS
        .header(24.0, |mut header| {
            let mut render_col =
                |header: &mut egui_extras::TableRow<'_, '_>, title: &str, col: SortColumn| {
                    header.col(|ui| {
                        let arrow = if app.sort_column == Some(col) {
                            if app.sort_direction == SortDirection::Ascending {
                                " ^"
                            } else {
                                " v"
                            }
                        } else {
                            ""
                        };
                        if ui.button(format!("{}{}", title, arrow)).clicked() {
                            sort_clicked = Some(col);
                        }
                    });
                };

            render_col(&mut header, "ID", SortColumn::Id);
            render_col(&mut header, "Status", SortColumn::Status);
            render_col(&mut header, "Created", SortColumn::Created);
            render_col(&mut header, "Modified", SortColumn::Modified);
            render_col(&mut header, "Accessed", SortColumn::Accessed);
            render_col(&mut header, "Size", SortColumn::Size);
            render_col(&mut header, "Name", SortColumn::Name);
            render_col(&mut header, "ADS", SortColumn::ADS);
        })
        .body(|body| {
            body.rows(row_height, total_rows, |mut row| {
                let row_idx = row.index();
                let record_idx = filtered_indices[row_idx];
                let record = &app.records[record_idx];
                let rec_num = record.record_number() as usize;

                for col_idx in 0..8 {
                    row.col(|ui| {
                        let is_cell_selected = app.selected_cell == Some((rec_num, col_idx));
                        let cell_size = ui.available_size();
                        let (rect, response) =
                            ui.allocate_exact_size(cell_size, egui::Sense::click());

                        if response.clicked() {
                            new_selected_cell = Some((rec_num, col_idx));
                        }

                        let bg = if is_cell_selected {
                            egui::Color32::from_rgba_unmultiplied(40, 110, 230, 80)
                        } else if response.hovered() {
                            egui::Color32::from_rgba_unmultiplied(255, 255, 255, 15)
                        } else {
                            egui::Color32::TRANSPARENT
                        };

                        let stroke = if is_cell_selected {
                            egui::Stroke::new(2.0, egui::Color32::from_rgb(100, 190, 255))
                        } else {
                            egui::Stroke::NONE
                        };

                        let pixel_rect =
                            egui::Rect::from_min_max(rect.min.round(), rect.max.round());
                        if bg != egui::Color32::TRANSPARENT {
                            ui.painter().rect_filled(pixel_rect, 0.0, bg);
                        }
                        if stroke != egui::Stroke::NONE {
                            ui.painter().rect_stroke(
                                pixel_rect,
                                0.0,
                                stroke,
                                egui::StrokeKind::Inside,
                            );
                        }

                        let content_rect = rect.shrink2(egui::vec2(6.0, 1.0));
                        let text_pos = content_rect.left_center();

                        let (text, is_mono, text_color) = match col_idx {
                            0 => (format!("#{:04}", rec_num), true, ui.visuals().text_color()),
                            1 => {
                                if record.is_in_use() {
                                    (
                                        "ALIVE".to_string(),
                                        false,
                                        egui::Color32::from_rgb(80, 200, 80),
                                    )
                                } else {
                                    (
                                        "DELETED".to_string(),
                                        false,
                                        egui::Color32::from_rgb(240, 80, 80),
                                    )
                                }
                            }
                            2 => (record.created_time(), true, ui.visuals().text_color()),
                            3 => (record.modified_time(), true, ui.visuals().text_color()),
                            4 => (record.accessed_time(), true, ui.visuals().text_color()),
                            5 => (record.formatted_size(), true, ui.visuals().text_color()),
                            6 => (
                                record.name().unwrap_or("[No Name]").to_string(),
                                false,
                                ui.visuals().text_color(),
                            ),
                            7 => (record.ads_summary(), false, ui.visuals().text_color()),
                            _ => (String::new(), false, ui.visuals().text_color()),
                        };

                        let font = if is_mono {
                            egui::FontId::monospace(13.0)
                        } else {
                            egui::FontId::proportional(14.0)
                        };

                        ui.painter().with_clip_rect(content_rect).text(
                            text_pos,
                            egui::Align2::LEFT_CENTER,
                            &text,
                            font,
                            text_color,
                        );
                    });
                }
            });
        });

    if let Some(cell) = new_selected_cell {
        app.selected_cell = Some(cell);
        app.selected_record = Some(cell.0);
    }

    if let Some(col) = sort_clicked {
        app.toggle_sort(col);
    }
}
