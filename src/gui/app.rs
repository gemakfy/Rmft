use eframe::egui;

use crate::gui::explorer::render_explorer_screen;
use crate::gui::welcome::render_welcome_screen;
use crate::mft::MftEntry;

pub struct MftApp {
    pub title: String,
    pub search_query: String,
    pub deleted_only: bool,
    pub records: Vec<MftEntry>,
    pub selected_record: Option<usize>,
    pub device_path: String,
    pub error_message: Option<String>,
    pub filtered_indices: Vec<usize>,
    last_search_query: String,
    last_deleted_only: bool,
    last_records_len: usize,
}

impl Default for MftApp {
    fn default() -> Self {
        #[cfg(target_os = "windows")]
        let default_device = r"\\.\C:".to_string();
        #[cfg(not(target_os = "windows"))]
        let default_device = "/dev/sda2".to_string();

        Self {
            title: "NTFS MFT Explorer".to_string(),
            search_query: String::new(),
            deleted_only: false,
            records: Vec::new(),
            selected_record: None,
            device_path: default_device,
            error_message: None,
            filtered_indices: Vec::new(),
            last_search_query: String::new(),
            last_deleted_only: false,
            last_records_len: 0,
        }
    }
}

impl MftApp {
    pub fn update_filtered_indices(&mut self) {
        if self.records.len() == self.last_records_len
            && self.search_query == self.last_search_query
            && self.deleted_only == self.last_deleted_only
        {
            return;
        }

        let query = self.search_query.trim().to_lowercase();
        let query_empty = query.is_empty();

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
                    let name = r.name().unwrap_or("[No Name]");
                    let matches_name = name.to_lowercase().contains(&query);
                    let matches_id = r.record_number().to_string() == query;
                    if !matches_name && !matches_id {
                        return false;
                    }
                }
                true
            })
            .map(|(i, _)| i)
            .collect();

        self.last_search_query = self.search_query.clone();
        self.last_deleted_only = self.deleted_only;
        self.last_records_len = self.records.len();
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
