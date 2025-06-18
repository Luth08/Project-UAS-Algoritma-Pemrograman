// src/screens/database_screen.rs

use egui::{RichText, Color32, Grid, ScrollArea};
use mongodb::bson::Document;
// Tambahkan import ini untuk menangani Bson::DateTime
use mongodb::bson::Bson;
// Hapus baris ini untuk menghilangkan warning "unused imports" dari chrono
// use chrono::{DateTime, Utc, TimeZone}; // <--- Hapus baris ini!


// Enum untuk memilih jenis data yang ditampilkan
#[derive(PartialEq, Debug, Clone)]
pub enum DatabaseDataType {
    PhotodiodeData,
    NewtonRaphsonResults,
}

pub struct DatabaseScreen {
    pub current_display_type: DatabaseDataType,
}

impl DatabaseScreen {
    pub fn new() -> Self {
        Self {
            current_display_type: DatabaseDataType::PhotodiodeData, // Default menampilkan photodiode Data
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui, data: &Vec<Document>) {
        ui.vertical_centered(|ui| {
            ui.add_space(10.0);
            ui.heading(RichText::new("Data Tersimpan (MongoDB)").color(Color32::WHITE).strong());
            ui.add_space(15.0);
        });

        // Pilihan untuk menampilkan data photodiode atau Newton-Raphson
        ui.horizontal(|ui_h| {
            ui_h.label(RichText::new("Tampilkan Data:").color(Color32::WHITE));
            ui_h.radio_value(&mut self.current_display_type, DatabaseDataType::PhotodiodeData, "Data photodiode");
            ui_h.radio_value(&mut self.current_display_type, DatabaseDataType::NewtonRaphsonResults, "Hasil Newton-Raphson");
        });
        ui.add_space(10.0);

        ScrollArea::vertical()
            .auto_shrink([false; 2])
            .max_height(ui.available_height() - 20.0)
            .show(ui, |ui| {

            ui.add_space(5.0);
            Grid::new("database_header_grid")
                .num_columns(3)
                .spacing([20.0, 8.0])
                .striped(true)
                .show(ui, |ui_grid| {
                    ui_grid.strong(RichText::new("No.").color(Color32::LIGHT_BLUE));
                    match self.current_display_type {
                        DatabaseDataType::PhotodiodeData => ui_grid.strong(RichText::new("Photodiode").color(Color32::LIGHT_BLUE)),
                        DatabaseDataType::NewtonRaphsonResults => ui_grid.strong(RichText::new("Akar (Newton-Raphson)").color(Color32::LIGHT_BLUE)),
                    };
                    ui_grid.strong(RichText::new("Waktu Pengukuran").color(Color32::LIGHT_BLUE));
                    ui_grid.end_row();
                });
            ui.add_space(5.0);


            if data.is_empty() {
                ui.vertical_centered(|ui_centered| {
                    ui_centered.add_space(20.0);
                    ui_centered.label(RichText::new("Belum ada data di database.").color(Color32::GRAY).italics());
                    ui_centered.label(RichText::new("Pastikan sensor terhubung dan pengiriman data ke MongoDB aktif.").color(Color32::GRAY).italics());
                });
            } else {
                Grid::new("database_data_grid")
                    .num_columns(3)
                    .spacing([20.0, 8.0])
                    .striped(true)
                    .show(ui, |ui_grid| {
                        for (i, doc) in data.iter().enumerate() {
                            let doc_index = i + 1;

                            let value_str = match self.current_display_type {
                                DatabaseDataType::PhotodiodeData => {
                                    if let Some(bson_value) = doc.get("photodiode_value") {
                                        match bson_value {
                                            Bson::Double(v) => format!("{:.2}", v),
                                            Bson::Int32(v) => format!("{:.0}", v), // Jika 0 disimpan sebagai Int32
                                            _ => "N/A (Tipe Data Tak Dikenal)".to_string(),
                                        }
                                    } else {
                                        "N/A (Field Tidak Ditemukan)".to_string()
                                    }
                                },
                                DatabaseDataType::NewtonRaphsonResults => doc.get_f64("akar_terakhir")
                                    .map(|v| format!("{:.8}", v))
                                    .unwrap_or_else(|_| "N/A".to_string()),
                            };

                            let timestamp_str = doc.get_datetime("timestamp")
                                .map(|dt| {
                                    // chrono::DateTime<chrono::Utc> sudah otomatis tersedia karena metode to_chrono()
                                    // tidak memerlukan import eksplisit DateTime, Utc, atau TimeZone di sini.
                                    let utc_dt = dt.to_chrono();
                                    utc_dt.format("%Y-%m-%d %H:%M:%S UTC").to_string()
                                })
                                .unwrap_or_else(|_| "N/A".to_string());

                            ui_grid.label(RichText::new(format!("{}", doc_index)).color(Color32::WHITE));
                            ui_grid.label(RichText::new(&value_str).color(Color32::YELLOW).strong());
                            ui_grid.label(RichText::new(&timestamp_str).color(Color32::LIGHT_GREEN));
                            ui_grid.end_row();
                        }
                    });
            }
        });
    }
}