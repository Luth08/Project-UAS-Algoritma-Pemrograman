// src/screens/data_graphics_screen.rs

use egui::{Color32, RichText, Ui}; // <--- Hapus ScrollArea dari sini
use egui_plot::{Line, Plot, PlotPoints, Legend}; // <--- LineShape akan diimpor secara spesifik atau digunakan dengan path penuh
use crate::measurements::Measurements; // <--- Import Measurements dan Value di sini
// Hapus baris ini: use measurements::{Value}; // Redundan
// Hapus baris ini: use std::sync::mpsc;
// Hapus baris ini: use crate::AppEvent;
use std::sync::{Arc, Mutex}; // Ini tetap dibutuhkan

// Tambahkan import LineShape secara spesifik:


pub struct DataGraphicsScreen {
    pub measurements: Arc<Mutex<Measurements>>, // Ini untuk nilai photodiode mentah/scaled
    pub newton_raphson_lux_measurements: Arc<Mutex<Measurements>>, // Ini untuk hasil Lux dari NR
}

impl DataGraphicsScreen {
    pub fn new(
        measurements: Arc<Mutex<Measurements>>,
        newton_raphson_lux_measurements: Arc<Mutex<Measurements>>, 
        max_data_points: usize,
        _app_event_sender: std::sync::mpsc::Sender<crate::AppEvent>,
    ) -> Self {
        measurements.lock().unwrap().set_max_data_points(max_data_points);
        newton_raphson_lux_measurements.lock().unwrap().set_max_data_points(max_data_points);

        Self {
            measurements: measurements,
            newton_raphson_lux_measurements: newton_raphson_lux_measurements,
        }
    }

    pub fn show(&mut self, ui: &mut Ui) {
        let measurements_guard = self.measurements.lock().unwrap();
        let newton_raphson_lux_guard = self.newton_raphson_lux_measurements.lock().unwrap();

        egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show(ui, |ui_scroll_content| {
                ui_scroll_content.add_space(5.0);
                ui_scroll_content.heading(RichText::new("Analisis Data Sensor dan Numerik").color(Color32::WHITE).strong());
                ui_scroll_content.add_space(15.0);

                // --- Bagian Grafik Data Sensor PhotoDioda (nilai scaled) ---
                ui_scroll_content.group(|ui| {
                    ui.add_space(5.0);
                    ui.heading(RichText::new("Grafik Nilai Sensor PhotoDioda").color(Color32::LIGHT_GREEN).strong());
                    ui.add_space(10.0);

                    let photodiode_data: PlotPoints = measurements_guard.values.iter().map(|v| [v.x, v.y]).collect();
                    let line = Line::new("Nilai Terukur Sensor", photodiode_data).color(Color32::from_rgb(100, 200, 255)).width(2.0);

                    Plot::new("photodiode_Tegangan_plot")
                        .width(ui.available_width())
                        .height(280.0) 
                        .view_aspect(2.0)
                        .include_y(0.0)
                        .auto_bounds([true, true]) 
                        .set_margin_fraction([0.1, 0.1].into())
                        .show_background(true)
                        .label_formatter(|name, value| {
                            if !name.is_empty() {
                                format!("{}: {:.2}", name, value.y)
                            } else {
                                "".to_owned()
                            }
                        })
                        .legend(Legend::default())
                        .show(ui, |plot_ui| {
                            plot_ui.line(line);
                        });

                    if measurements_guard.values.is_empty() {
                        ui.label(RichText::new("Menunggu data photodiode dari sensor...").color(Color32::GRAY).italics());
                    } else {
                        ui.label(format!("Nilai Sensor Terbaru: {:.2}", measurements_guard.values.last().unwrap().y));
                    }
                    ui.add_space(5.0);
                });

                ui_scroll_content.add_space(20.0);

                // --- Bagian Grafik Perhitungan Newton-Raphson (Lux) ---
                ui_scroll_content.group(|ui| {
                    ui.add_space(5.0);
                    ui.heading(RichText::new("Grafik Lux Hasil Newton-Raphson").color(Color32::LIGHT_BLUE).strong());
                    ui.add_space(10.0);

                    let newton_raphson_lux_data: PlotPoints = newton_raphson_lux_guard.values.iter().map(|v| [v.x, v.y]).collect();
                    let line_nr = Line::new("Lux Hasil NR", newton_raphson_lux_data)
                                    .color(Color32::from_rgb(255, 100, 100))
                                    .width(2.0);

                    Plot::new("newton_raphson_lux_plot")
                        .width(ui.available_width())
                        .height(280.0) 
                        .view_aspect(2.0)
                        .include_y(0.0) 
                        .auto_bounds([true, true]) 
                        .show_background(true)
                        .legend(Legend::default())
                        .label_formatter(|name, value| {
                            if !name.is_empty() {
                                format!("{}: Waktu {}, Lux = {:.2}", name, value.x, value.y)
                            } else {
                                "".to_owned()
                            }
                        })
                        .show(ui, |plot_ui| {
                            plot_ui.line(line_nr);
                        });

                    if newton_raphson_lux_guard.values.is_empty() {
                        ui.label(RichText::new("Menunggu perhitungan Newton-Raphson Lux...").color(Color32::GRAY).italics());
                    } else {
                        ui.label(format!("Lux Newton-Raphson Terbaru: {:.2}", newton_raphson_lux_guard.values.last().unwrap().y));
                    }
                    ui.add_space(5.0);
                });
            }); 
        }

        pub fn clear_data(&mut self) {
            self.measurements.lock().unwrap().clear_values();
            self.newton_raphson_lux_measurements.lock().unwrap().clear_values();
            println!("DataGraphicsScreen: Data cleared.");
        }
    }