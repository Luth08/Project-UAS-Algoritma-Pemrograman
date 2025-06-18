// src/screens/home_screen.rs

use egui::{Ui, RichText, Color32};

pub struct HomeScreen {
    // button_clicked_count: i32, // Hapus ini karena counter dihapus
}

impl HomeScreen {
    pub fn new() -> Self {
        Self {
            // button_clicked_count: 0, // Hapus ini
        }
    }

    pub fn show(&mut self, ui: &mut Ui, current_photodiode_value: f64) {
        ui.vertical_centered(|ui| {
            ui.add_space(20.0);
            ui.heading(RichText::new("Selamat Datang di Sistem Pemantauan Tanaman Selada Greenhouse")
                .color(Color32::LIGHT_GREEN)
                .strong()
                .size(24.0));

            ui.add_space(30.0);

            ui.group(|ui| {
                ui.add_space(10.0);
                ui.vertical_centered(|ui| {
                    ui.heading(RichText::new("Status Intensitas Cahaya").color(Color32::WHITE).strong());
                    ui.add_space(15.0);

                    // Logika warna dan status berdasarkan nilai photodiode
                    // Asumsi: photodiode rendah = Terang, photodiode tinggi = Gelap
                    let photodiode_text_color = if current_photodiode_value < 300.0 {
                        Color32::YELLOW // Sangat Terang
                    } else if current_photodiode_value < 600.0 {
                        Color32::LIGHT_GREEN // Normal
                    } else {
                        Color32::from_rgb(100, 100, 200) // Gelap (biru gelap)
                    };

                    let light_status_icon = if current_photodiode_value < 300.0 {
                        "☀️" // Sangat Terang
                    } else if current_photodiode_value < 600.0 {
                        "☁️" // Normal
                    } else {
                        "🌙" // Gelap
                    };

                    let light_status_text = if current_photodiode_value < 300.0 {
                        "Sangat Terang"
                    } else if current_photodiode_value < 600.0 {
                        "Normal"
                    } else {
                        "Gelap"
                    };

                    ui.label(RichText::new(format!("Nilai photodiode Saat Ini: {:.2}", current_photodiode_value))
                        .color(photodiode_text_color)
                        .size(40.0)
                        .strong());
                    ui.add_space(10.0);
                    ui.label(RichText::new(format!("{} {}", light_status_icon, light_status_text))
                        .color(photodiode_text_color)
                        .size(22.0)
                        .italics());
                });
                ui.add_space(10.0);
            });

            ui.add_space(40.0);

            ui.label(RichText::new("Gunakan navigasi di sisi kiri untuk memilih modul pemantauan.")
                .color(Color32::WHITE)
                .size(18.0));
            ui.add_space(20.0);

            // ui.button() dan ui.label() terkait counter dihapus di sini

            ui.add_space(40.0);
            ui.horizontal(|ui| {
                ui.label(RichText::new("💡 Tips:").strong().color(Color32::YELLOW));
                ui.label(RichText::new("Intensitas cahaya adalah faktor kunci untuk pertumbuhan selada Greenhouse.").color(Color32::WHITE));
            });
            ui.horizontal(|ui| {
                ui.label(RichText::new("ℹ️ Info:").strong().color(Color32::WHITE));
                ui.label(RichText::new("Rentang nilai photodiode 0-1023 (0=Terang, 1023=Gelap)").color(Color32::WHITE));
            });
            ui.horizontal(|ui| {
                ui.label(RichText::new("⚠️ Catatan:").strong().color(Color32::RED));
                ui.label(RichText::new("Asumsi: Nilai photodiode rendah = Terang, Nilai photodiode tinggi = Gelap").color(Color32::WHITE));
            });
        });
    }
}