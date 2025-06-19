mod measurements;
mod screens;
mod db;


use eframe::{App, CreationContext, NativeOptions};
use egui::{CentralPanel, Context, ViewportBuilder, TopBottomPanel, SidePanel, Layout, Color32, RichText, Frame, Stroke};


use measurements::{Measurements, Value};
use screens::{
    home_screen::HomeScreen,
    data_graphics_screen::DataGraphicsScreen,
    database_screen::{DatabaseScreen, DatabaseDataType}, 
    sensor_configuration_screen::SensorConfigurationScreen,
};

use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex}; 
use serialport;


#[derive(PartialEq)]
enum AppScreen {
    Home,
    DataGraphics,
    Database,
    SensorConfiguration,
}

pub enum AppEvent {
    NewtonRaphsonCalculated {
        akar: f64,
        history: Vec<f64>,
    },
}

struct MyApp {
    pub measurements: Arc<Mutex<Measurements>>, 
    pub newton_raphson_lux_measurements: Arc<Mutex<Measurements>>, 
    photodiode_data_receiver: mpsc::Receiver<Value>,
    current_photodiode_value: f64,
    current_screen: AppScreen,

    home_screen: HomeScreen,
    data_graphics_screen: DataGraphicsScreen,
    database_screen: DatabaseScreen,
    sensor_configuration_screen: SensorConfigurationScreen,
    
    database_data: Arc<Mutex<Vec<mongodb::bson::Document>>>,

    serial_status_message: String,
    serial_status_receiver: mpsc::Receiver<String>,

    app_event_receiver: mpsc::Receiver<AppEvent>,
    #[allow(dead_code)]
    app_event_sender: mpsc::Sender<AppEvent>,
}



impl MyApp {
    fn new(cc: &CreationContext<'_>) -> Self {
        let mut style = (*cc.egui_ctx.style()).clone();
        style.text_styles.insert(
            egui::TextStyle::Heading,
            egui::FontId::proportional(28.0),
        );
        style.text_styles.insert(
            egui::TextStyle::Body,
            egui::FontId::proportional(16.0),
        );
        cc.egui_ctx.set_style(style);

        let (photodiode_tx, photodiode_rx) = mpsc::channel();
        let (status_tx, status_rx) = mpsc::channel(); 
        let (app_event_tx, app_event_rx) = mpsc::channel(); 
        let start_time = Instant::now();

        let sensor_config_screen = SensorConfigurationScreen::new();
        let initial_baud_rate = sensor_config_screen.baud_rate; 
        
        let status_tx_clone = status_tx.clone();

        let shared_measurements = Arc::new(Mutex::new(Measurements::new())); 
        let shared_newton_raphson_lux_measurements = Arc::new(Mutex::new(Measurements::new()));

        let measurements_for_graphics_screen = Arc::clone(&shared_measurements); 
        let newton_raphson_lux_for_graphics_screen = Arc::clone(&shared_newton_raphson_lux_measurements);

        let app_event_tx_for_graphics = app_event_tx.clone();


        thread::spawn(move || {
            let port_name = "COM4"; 
            let baud_rate = initial_baud_rate; 

            if status_tx_clone.send(format!("Mencoba membuka port: {}...", port_name)).is_err() { return; }

            match serialport::new(port_name, baud_rate)
                .timeout(Duration::from_millis(30))
                .open()
            {
                Ok(mut port) => {
                    if status_tx_clone.send(format!("Terhubung ke: {} ({} bps)", port_name, baud_rate)).is_err() { return; }
                    let mut serial_buf: Vec<u8> = vec![0; 256]; 
                    let mut received_string = String::new();

                    loop {
                        match port.read(serial_buf.as_mut_slice()) {
                            Ok(bytes_read) => {
                                if bytes_read > 0 {
                                    received_string.push_str(&String::from_utf8_lossy(&serial_buf[..bytes_read]));
                                    
                                    if let Some(newline_pos) = received_string.find('\n') {
                                        let line = received_string.drain(..newline_pos + 1).collect::<String>();
                                        let trimmed_line = line.trim();
                                        
                                        let mut photodiode_value: Option<f64> = None;

                                        if let Ok(val) = trimmed_line.parse::<f64>() {
                                            photodiode_value = Some(val);
                                            if status_tx_clone.send(format!("Nilai Photodiode Diterima: {:.2}", val)).is_err() { return; }
                                        } else {
                                            if status_tx_clone.send(format!("Parsing ERROR (Photodiode): '{}'", trimmed_line)).is_err() { return; }
                                        }
                                        
                                        if photodiode_tx.send(Value { x: start_time.elapsed().as_secs_f64(), y: photodiode_value.unwrap_or(0.0) }).is_err() { 
                                            if status_tx_clone.send("Channel photodiode ditutup.".to_string()).is_err() { return; }
                                            break; 
                                        }
                                    }
                                }
                            },
                            Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {  },
                            Err(e) => {
                                if status_tx_clone.send(format!("Serial Read ERROR: {:?}", e)).is_err() { return; }
                                break; 
                            }
                        }
                        thread::sleep(Duration::from_millis(10)); 
                    }
                },
                Err(e) => {
                    if status_tx_clone.send(format!("Gagal membuka port {}: {}", port_name, e)).is_err() { return; }
                    if status_tx_clone.send("Pastikan Arduino IDE Serial Monitor TIDAK terbuka!".to_string()).is_err() { return; }
                }
            }
        });


        let max_data_points = 300; 

        Self {
            measurements: shared_measurements, 
            newton_raphson_lux_measurements: shared_newton_raphson_lux_measurements,
            photodiode_data_receiver: photodiode_rx,
            current_photodiode_value: 0.0, 
            current_screen: AppScreen::Home, 

            home_screen: HomeScreen::new(),
            data_graphics_screen: DataGraphicsScreen::new(
                measurements_for_graphics_screen, 
                newton_raphson_lux_for_graphics_screen,
                max_data_points, 
                app_event_tx_for_graphics
            ), 
            database_screen: DatabaseScreen::new(), 
            sensor_configuration_screen: sensor_config_screen, 
            
            database_data: Arc::new(Mutex::new(Vec::new())),
            serial_status_message: "Menunggu koneksi serial...".to_string(), 
            serial_status_receiver: status_rx,
            app_event_receiver: app_event_rx, 
            app_event_sender: app_event_tx,
        }
    }
}

impl App for MyApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        while let Ok(msg) = self.serial_status_receiver.try_recv() {
            self.serial_status_message = msg;
            ctx.request_repaint(); 
        }

        while let Ok(event) = self.app_event_receiver.try_recv() {
            match event {
                AppEvent::NewtonRaphsonCalculated { akar, history } => {
                    self.sensor_configuration_screen.update_nr_display_data(akar, history.clone()); 
                    
                    let akar_for_db = akar; 
                    
                    std::thread::spawn(move || {
                        let rt = tokio::runtime::Runtime::new().unwrap();
                        rt.block_on(async {
                            match db::connect_db().await {
                                Ok(db) => {
                                    let _ = db::insert_newton_raphson_result(&db, akar_for_db, Vec::new()).await; 
                                    eprintln!("[DB Thread] Berhasil menyimpan hasil Newton-Raphson: {:.8}", akar_for_db);
                                },
                                Err(e) => {
                                    eprintln!("[DB Thread] GAGAL terhubung/menyimpan hasil Newton-Raphson: {:?}", e);
                                }
                            }
                        });
                    });

                    if self.current_screen == AppScreen::Database {
                        self.fetch_database_data(ctx, DatabaseDataType::NewtonRaphsonResults); 
                    }
                },
            }
            ctx.request_repaint(); 
        }


        if self.current_screen == AppScreen::Database && self.database_data.lock().unwrap().is_empty() {
             self.fetch_database_data(ctx, self.database_screen.current_display_type.clone());
        }

        while let Ok(new_value) = self.photodiode_data_receiver.try_recv() {
            self.measurements.lock().unwrap().add_value(new_value); 
            self.current_photodiode_value = new_value.y; 

            const ARDUINO_MAX_VOLTAGE: f64 = 3.3; 
            const ARDUINO_MAX_PHOTODIODE_VALUE: f64 = 1000.0; 

            let scale_factor_to_voltage = ARDUINO_MAX_VOLTAGE / ARDUINO_MAX_PHOTODIODE_VALUE;
            let v_out_terukur = new_value.y * scale_factor_to_voltage; 
            
            let calib_a_power = self.sensor_configuration_screen.calib_a_power;
            let calib_b_power = self.sensor_configuration_screen.calib_b_power;
            
            let f = |lux_estimate: f64| -> f64 {
                if lux_estimate <= 0.0 { 
                    return f64::MAX; 
                }
                calib_a_power * lux_estimate.powf(calib_b_power) - v_out_terukur
            };

            let f_prime = |lux_estimate: f64| -> f64 {
                if lux_estimate <= 0.0 {
                    return f64::MAX; 
                }
                calib_a_power * calib_b_power * lux_estimate.powf(calib_b_power - 1.0)
            };

            let mut x0: f64 = self.sensor_configuration_screen.initial_guess_nr;
            let tolerance: f64 = self.sensor_configuration_screen.tolerance_nr;
            let max_iterations: usize = self.sensor_configuration_screen.max_iterations_nr as usize; 
            
            if x0 <= 0.0 {
                eprintln!("[Newton-Raphson] Tebakan awal Lux ({}) tidak valid. Menggunakan 1.0.", x0);
                x0 = 1.0;
            }

            let (lux_newton_raphson_result, history_newton_raphson_values) =
                Measurements::newton_raphson(f, f_prime, x0, tolerance, max_iterations);

            let final_lux_nr = if lux_newton_raphson_result.is_finite() && lux_newton_raphson_result >= 0.0 {
                lux_newton_raphson_result
            } else {
                eprintln!("[Newton-Raphson] Hasil tidak valid: {}. Menggunakan 0.0 Lux.", lux_newton_raphson_result);
                0.0
            };

            println!("Nilai Photodiode dari Arduino (Scaled 0-1000): {:.2}", new_value.y);
            println!("Tegangan Output Terukur (V_out): {:.4} V", v_out_terukur);
            println!("Lux dari Newton-Raphson (Validasi): {:.2} Lux", final_lux_nr);

            self.newton_raphson_lux_measurements.lock().unwrap().add_value(
                Value { x: new_value.x, y: final_lux_nr }
            );

            let nr_history_for_display: Vec<f64> = history_newton_raphson_values.into_iter().map(|v| v.y).collect();
            
            if self.app_event_sender.send(AppEvent::NewtonRaphsonCalculated { 
                akar: final_lux_nr, 
                history: nr_history_for_display 
            }).is_err() {
                eprintln!("Gagal mengirim AppEvent::NewtonRaphsonCalculated ke SensorConfigurationScreen.");
            }

            let photodiode_value_for_db = new_value.y; 

            std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    if let Ok(db) = db::connect_db().await {
                        let _ = db::insert_photodiode_data(&db, photodiode_value_for_db).await; 
                        let _ = db::insert_newton_raphson_result(&db, final_lux_nr, Vec::new()).await; 
                    } else {
                         eprintln!("[DB Thread] Gagal terhubung ke database untuk menyimpan data photodiode/NR.");
                    }
                });
            });

        }

        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.add_space(10.0);
            ui.vertical_centered(|ui| {
                ui.heading(RichText::new("Monitoring Intesitas Cahaya Pada Green House menggunakan Light Sensor untuk Budidaya tanaman Selada (Lactusa Sativa)").color(Color32::WHITE).strong());
            });
            ui.add_space(10.0);
        });

        SidePanel::left("side_panel")
            .exact_width(180.0)
            .frame(Frame::window(&ctx.style())
                .fill(Color32::from_rgb(200, 180, 255)) 
                .stroke(Stroke::new(1.0, Color32::from_rgb(150, 100, 255)))
                .corner_radius(5.0) 
            )
            .show(ctx, |ui| {
                ui.add_space(20.0);
                ui.vertical_centered(|ui| {
                    ui.heading(RichText::new("Navigasi").color(Color32::from_rgb(80, 0, 150)).strong());
                });
                ui.add_space(20.0);

                ui.vertical(|ui| {
                    if ui.button(RichText::new("🏠 Home").size(18.0).color(Color32::WHITE)).clicked() {
                        self.current_screen = AppScreen::Home;
                    }
                    ui.add_space(10.0);
                    if ui.button(RichText::new("📊 Data Graphics").size(18.0).color(Color32::WHITE)).clicked() {
                        self.current_screen = AppScreen::DataGraphics;
                    }
                    ui.add_space(10.0);
                    if ui.button(RichText::new("🗄️ Database").size(18.0).color(Color32::WHITE)).clicked() {
                        self.current_screen = AppScreen::Database;
                    }
                    ui.add_space(10.0);
                    if ui.button(RichText::new("⚙️ Sensor Configuration").size(18.0).color(Color32::WHITE)).clicked() {
                        self.current_screen = AppScreen::SensorConfiguration;
                    }
                });
            });

        CentralPanel::default()
            .frame(Frame::window(&ctx.style())
                .fill(Color32::from_rgb(30, 30, 40)) 
                .corner_radius(0.0) 
            )
            .show(ctx, |ui| {
                ui.add_space(10.0);
                match self.current_screen {
                    AppScreen::Home => self.home_screen.show(ui, self.current_photodiode_value),
                    AppScreen::DataGraphics => self.data_graphics_screen.show(ui), 
                    AppScreen::Database => {
                        let data = self.database_data.lock().unwrap();
                        self.database_screen.show(ui, &data) 
                    },
                    AppScreen::SensorConfiguration => self.sensor_configuration_screen.show(ui),
                }

                ui.add_space(10.0); 
                ui.with_layout(Layout::bottom_up(egui::Align::LEFT), |ui_bottom| {
                    ui_bottom.label(RichText::new(format!("Status Serial: {}", self.serial_status_message))
                        .color(if self.serial_status_message.contains("ERROR") || self.serial_status_message.contains("Gagal") {
                            Color32::RED
                        } else if self.serial_status_message.contains("Terhubung") || self.serial_status_message.contains("Diterima") { 
                            Color32::LIGHT_GREEN
                        } else {
                            Color32::GRAY
                        })
                        .size(14.0)
                        .italics()
                    );
                    
                    if ui_bottom.button(RichText::new("🗑️ Kosongkan Semua Data Grafis").color(Color32::BLACK).background_color(Color32::RED)).clicked() {
                        self.measurements.lock().unwrap().clear_values(); 
                        self.newton_raphson_lux_measurements.lock().unwrap().clear_values();
                        self.data_graphics_screen.clear_data();
                        self.database_data.lock().unwrap().clear(); 
                        ctx.request_repaint(); 
                    }
                });
            });
    }
}

impl MyApp {
    fn fetch_database_data(&mut self, ctx: &Context, data_type: DatabaseDataType) {
        let database_data_arc = Arc::clone(&self.database_data);
        {
            let mut data = database_data_arc.lock().unwrap();
            data.clear();
        }
        ctx.request_repaint(); 

        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let docs = rt.block_on(async {
                match db::connect_db().await {
                    Ok(db_conn) => {
                        eprintln!("[DB Thread] Berhasil terhubung ke database untuk fetch data.");
                        match data_type {
                            DatabaseDataType::PhotodiodeData => {
                                eprintln!("[DB Thread] Fetching photodiode data...");
                                db::get_all_photodiode_data(&db_conn).await.unwrap_or_else(|e| {
                                    eprintln!("[DB Thread] GAGAL mengambil data photodiode: {:?}", e); 
                                    Vec::new() 
                                })
                            },
                            DatabaseDataType::NewtonRaphsonResults => {
                                eprintln!("[DB Thread] Fetching Newton-Raphson results...");
                                db::get_all_newton_raphson_results(&db_conn).await.unwrap_or_else(|e| {
                                    eprintln!("[DB Thread] GAGAL mengambil hasil Newton-Raphson: {:?}", e); 
                                    Vec::new() 
                                })
                            },
                        }
                    },
                    Err(e) => {
                        eprintln!("[DB Thread] GAGAL terhubung ke database untuk fetch data: {:?}", e); 
                        Vec::new()
                    }
                }
            });
            let mut data = database_data_arc.lock().unwrap();
            *data = docs;
        });
    }
}


fn main() -> eframe::Result<()> {
    let native_options = NativeOptions {
        viewport: ViewportBuilder::default()
            .with_inner_size([900.0, 700.0]) 
            .with_min_inner_size([700.0, 500.0]) 
            .with_title("Monitoring Intesitas Cahaya Pada Green House menggunakan Light Sensor untuk Budidaya tanaman Selada (Lactusa Sativa)"), 
        ..NativeOptions::default() 
    };

    eframe::run_native(
        "Sistem Pemantauan Intensitas Cahaya Tanaman Selada (Lactuca sativa)", 
        native_options,
        Box::new(|cc| {
            Ok(Box::new(MyApp::new(cc)))
        }),
    )
}
