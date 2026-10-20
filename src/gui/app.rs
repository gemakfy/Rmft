use eframe::egui;

use crate::gui::explorer::render_explorer_screen;
use crate::gui::welcome::render_welcome_screen;
use crate::mft::MftEntry;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortColumn {
    Id,
    Status,
    Created,
    Modified,
    Accessed,
    Size,
    Name,
    ADS,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    Ascending,
    Descending,
}

pub struct MftApp {
    pub title: String,
    pub search_query: String,
    pub deleted_only: bool,
    pub records: Vec<MftEntry>,
    pub selected_record: Option<usize>,
    pub selected_cell: Option<(usize, usize)>,
    pub copy_notification: Option<(String, std::time::Instant)>,
    pub device_path: String,
    pub discovered_devices: Vec<crate::mft::DiscoveredDevice>,
    pub error_message: Option<String>,
    pub filtered_indices: Vec<usize>,
    pub sort_column: Option<SortColumn>,
    pub sort_direction: SortDirection,
    last_search_query: String,
    last_deleted_only: bool,
    last_records_len: usize,
    last_sort_column: Option<SortColumn>,
    last_sort_direction: SortDirection,
}

impl Default for MftApp {
    fn default() -> Self {
        let discovered_devices = crate::mft::detect_devices();
        let default_device = discovered_devices
            .iter()
            .find(|d| d.is_ntfs)
            .map(|d| d.path.clone())
            .unwrap_or_else(|| {
                #[cfg(target_os = "windows")]
                {
                    r"\\.\C:".to_string()
                }
                #[cfg(not(target_os = "windows"))]
                {
                    "/dev/sda2".to_string()
                }
            });

        Self {
            title: "NTFS MFT Explorer".to_string(),
            search_query: String::new(),
            deleted_only: false,
            records: Vec::new(),
            selected_record: None,
            selected_cell: None,
            copy_notification: None,
            device_path: default_device,
            discovered_devices,
            error_message: None,
            sort_column: None,
            sort_direction: SortDirection::Ascending,
            filtered_indices: Vec::new(),
            last_search_query: String::new(),
            last_deleted_only: false,
            last_records_len: 0,
            last_sort_column: None,
            last_sort_direction: SortDirection::Ascending,
        }
    }
}

impl MftApp {
    pub fn update_filtered_indices(&mut self) {
        if self.records.len() == self.last_records_len
            && self.search_query == self.last_search_query
            && self.deleted_only == self.last_deleted_only
            && self.sort_column == self.last_sort_column
            && self.sort_direction == self.last_sort_direction
        {
            return;
        }

        let query = self.search_query.trim().to_lowercase();
        let query_empty = query.is_empty();
        let query_id = query
            .strip_prefix('#')
            .unwrap_or(&query)
            .parse::<u32>()
            .ok();

        self.filtered_indices = self
            .records
            .iter()
            .enumerate()
            .filter(|(_, r)| {
                let is_alive = r.is_in_use();
                if self.deleted_only && is_alive {
                    return false;
                }
                if !query_empty {
                    let matches_id = query_id.is_some_and(|id| r.record_number() == id);
                    if matches_id {
                        return true;
                    }
                    let name = r.name().unwrap_or("[No Name]");
                    let matches_name = if name.is_ascii() {
                        name.as_bytes()
                            .windows(query.len())
                            .any(|w| w.eq_ignore_ascii_case(query.as_bytes()))
                    } else {
                        name.to_lowercase().contains(&query)
                    };
                    if !matches_name {
                        return false;
                    }
                }
                true
            })
            .map(|(i, _)| i)
            .collect();

        if let Some(col) = self.sort_column {
            self.filtered_indices.sort_by(|&a, &b| {
                let ra = &self.records[a];
                let rb = &self.records[b];

                let ord = match col {
                    SortColumn::Id => ra.record_number().cmp(&rb.record_number()),
                    SortColumn::Status => ra.is_in_use().cmp(&rb.is_in_use()),
                    SortColumn::Size => ra.file_size().cmp(&rb.file_size()),
                    SortColumn::Name => {
                        let na = ra.name().unwrap_or("");
                        let nb = rb.name().unwrap_or("");
                        na.to_lowercase().cmp(&nb.to_lowercase())
                    }
                    SortColumn::Created => ra.created_raw().cmp(&rb.created_raw()),
                    SortColumn::Modified => ra.modified_raw().cmp(&rb.modified_raw()),
                    SortColumn::Accessed => ra.accessed_raw().cmp(&rb.accessed_raw()),
                    SortColumn::ADS => {
                        let a_name = ra
                            .alternate_streams
                            .first()
                            .map(|s| s.name.as_str())
                            .unwrap_or("");
                        let b_name = rb
                            .alternate_streams
                            .first()
                            .map(|s| s.name.as_str())
                            .unwrap_or("");
                        a_name.cmp(b_name)
                    }
                };

                if self.sort_direction == SortDirection::Descending {
                    ord.reverse()
                } else {
                    ord
                }
            });
        }

        self.last_search_query = self.search_query.clone();
        self.last_deleted_only = self.deleted_only;
        self.last_records_len = self.records.len();
        self.last_sort_column = self.sort_column;
        self.last_sort_direction = self.sort_direction;
    }

    pub fn toggle_sort(&mut self, column: SortColumn) {
        if self.sort_column == Some(column) {
            self.sort_direction = match self.sort_direction {
                SortDirection::Ascending => SortDirection::Descending,
                SortDirection::Descending => SortDirection::Ascending,
            };
        } else {
            self.sort_column = Some(column);
            self.sort_direction = SortDirection::Ascending;
        }
    }
}

impl eframe::App for MftApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        if self.records.is_empty() {
            render_welcome_screen(self, ui);
        } else {
            render_explorer_screen(self, ui);
        }
    }
}
