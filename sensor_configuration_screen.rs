use egui::{Ui, RichText, Color32, Grid, ScrollArea};
use crate::measurements::Value;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

pub struct SensorConfigurationScreen {
    pub calib_a_power: f64, 
    pub calib_b_power: f64,

    pub initial_guess_nr: f64,
    pub tolerance_nr: f64,
    pub max_iterations_nr: u32,
    
    pub baud_rate: u32,

    pub newton_raphson_iter_results: Vec<Value>,
    pub newton_raphson_akar: Option<f64>,
}

impl SensorConfigurationScreen {
    pub fn new() -> Self {
        Self {
            calib_a_power: 0.0001, 
            calib_b_power: 1.05,   

            initial_guess_nr: 1.0, 
            tolerance_nr: 1e-6,    
            max_iterations_nr: 20, 
            
            baud_rate: 9600,

            newton_raphson_iter_results: Vec::new(),
            newton_raphson_akar: None,
        }
    }

    pub fn update_nr_display_data(&mut self, akar: f64, history: Vec<f64>) {
        self.newton_raphson_akar = Some(akar);
        self.newton_raphson_iter_results.clear();
        for (i, val) in history.iter().enumerate() {
            self.newton_raphson_iter_results.push(Value { x: i as f64, y: *val });
        }
    }

    pub fn show(&mut self, ui: &mut Ui) {
        ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(20.0);
                    ui.heading(RichText::new("Konfigurasi Sensor & Numerik").color(Color32::WHITE).strong());
                    ui.add_space(30.0);
                });

                ui.group(|ui| {
                    ui.add_space(5.0);
                    ui.heading(RichText::new("Kalibrasi Sensor Photodiode (Model Power Law)").color(Color32::LIGHT_GREEN).strong());
                    ui.add_space(10.0);

                    Grid::new("calibration_power_law_grid")
                        .num_columns(2)
                        .spacing([40.0, 10.0])
                        .show(ui, |ui_grid| {
                            ui_grid.label(RichText::new("Konstanta A (Power Law):").color(Color32::WHITE));
                            ui_grid.add(egui::DragValue::new(&mut self.calib_a_power)
                                .speed(0.00001) 
                                .fixed_decimals(6)); 
                            ui_grid.end_row();

                            ui_grid.label(RichText::new("Konstanta B (Power Law):").color(Color32::WHITE));
                            ui_grid.add(egui::DragValue::new(&mut self.calib_b_power)
                                .speed(0.01) 
                                .fixed_decimals(2)); 
                            ui_grid.end_row();
                        });
                    ui.add_space(10.0);
                    ui.label(RichText::new("Sesuaikan 'Konstanta A' dan 'Konstanta B' berdasarkan hasil kalibrasi Power Law Anda.").color(Color32::GRAY).italics());
                    ui.label(RichText::new("Ini akan mempengaruhi akurasi hasil Lux dari Newton-Raphson.").color(Color32::RED).italics());
                });

                ui.add_space(30.0);

                ui.group(|ui| {
                    ui.add_space(5.0);
                    ui.heading(RichText::new("Konfigurasi Serial Port").color(Color32::LIGHT_GREEN).strong());
                    ui.add_space(10.0);

                    Grid::new("serial_config_grid")
                        .num_columns(2)
                        .spacing([40.0, 10.0])
                        .show(ui, |ui_grid| {
                            ui_grid.label(RichText::new("Baud Rate:").color(Color32::WHITE));
                            ui_grid.add(egui::DragValue::new(&mut self.baud_rate)
                                .speed(100.0)
                                .suffix(" bps")
                                .range(300..=115200)
                                .fixed_decimals(0));
                            ui_grid.end_row();
                        });
                    ui.add_space(10.0);
                    ui.label(RichText::new("Pastikan Baud Rate di sini sesuai dengan yang diatur pada Arduino.").color(Color32::GRAY).italics());
                    ui.label(RichText::new("Jika mengubah baud rate, aplikasi mungkin perlu di-restart untuk menerapkan perubahan.").color(Color32::RED).italics());
                });

                ui.add_space(30.0);

                ui.group(|ui| {
                    ui.add_space(5.0);
                    ui.heading(RichText::new("Konfigurasi Metode Newton-Raphson").color(Color32::LIGHT_BLUE).strong());
                    ui.add_space(10.0);

                    Grid::new("newton_raphson_grid")
                        .num_columns(2)
                        .spacing([40.0, 10.0])
                        .show(ui, |ui_grid| {
                            ui_grid.label(RichText::new("Tebakan Awal (x₀):").color(Color32::WHITE));
                            ui_grid.add(egui::DragValue::new(&mut self.initial_guess_nr)
                                .speed(0.1));
                            ui_grid.end_row();

                            ui_grid.label(RichText::new("Toleransi Error (ε):").color(Color32::WHITE));
                            ui_grid.add(egui::DragValue::new(&mut self.tolerance_nr)
                                .speed(1e-7)
                                .fixed_decimals(8));
                            ui_grid.end_row();

                            ui_grid.label(RichText::new("Iterasi Maksimum:").color(Color32::WHITE));
                            ui_grid.add(egui::DragValue::new(&mut self.max_iterations_nr)
                                .speed(1.0)
                                .fixed_decimals(0));
                            ui_grid.end_row();
                        });
                    ui.add_space(10.0);
                    ui.label(RichText::new("Parameter ini mempengaruhi akurasi dan kecepatan konvergensi metode Newton-Raphson.").color(Color32::GRAY).italics());
                    ui.label(RichText::new("Grafik hasil perhitungan numerik dapat dilihat di halaman 'Data Graphics'.").color(Color32::GRAY).italics());
                });

                ui.add_space(30.0);
                ui.group(|ui| {
                    ui.add_space(5.0);
                    ui.heading(RichText::new("Riwayat Iterasi Newton-Raphson Terbaru").color(Color32::YELLOW).strong());
                    ui.add_space(10.0);

                    if let Some(akar) = self.newton_raphson_akar {
                        ui.label(RichText::new(format!("Akar Terakhir: {:.8}", akar)).color(Color32::LIGHT_GREEN).strong());
                    } else {
                        ui.label(RichText::new("Belum ada perhitungan Newton-Raphson.").color(Color32::GRAY));
                    }
                    ui.add_space(5.0);

                    if self.newton_raphson_iter_results.is_empty() {
                        ui.label(RichText::new("Riwayat iterasi akan muncul di sini setelah perhitungan pertama.").color(Color32::GRAY).italics());
                    } else {
                        ScrollArea::vertical()
                            .max_height(200.0)
                            .show(ui, |ui_scroll_content| {
                                ui_scroll_content.add_space(5.0);
                                ui_scroll_content.label(RichText::new("Iterasi | X_n").color(Color32::WHITE).strong());
                                ui_scroll_content.separator();
                                for result in &self.newton_raphson_iter_results {
                                    ui_scroll_content.label(format!("{:.0} | {:.8}", result.x, result.y));
                                }
                            });
                    }
                });

                ui.add_space(40.0);
                ui.horizontal(|ui_h| {
                    ui_h.label(RichText::new("💡 Tips:").strong().color(Color32::LIGHT_BLUE));
                    ui_h.label(RichText::new("Nilai toleransi yang lebih kecil menghasilkan akurasi lebih tinggi, tetapi memerlukan lebih banyak iterasi.").color(Color32::WHITE));
                });
                ui.horizontal(|ui_h| {
                    ui_h.label(RichText::new("ℹ️ Info:").strong().color(Color32::GRAY));
                    ui_h.label(RichText::new("Newton-Raphson adalah metode iteratif untuk mencari akar fungsi.").color(Color32::WHITE));
                });
            });
    }
}
