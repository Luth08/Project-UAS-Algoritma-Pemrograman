// src/measurements.rs

#[derive(Clone)]
pub struct Measurements {
    pub values: Vec<Value>,
    pub max_data_points: usize,
}

// HAPUS STRUKTUR INI, KARENA SUDAH ADA DI data_graphics_screen.rs
// pub struct DataGraphicsScreen {
//     pub measurements: Measurements,
//     pub newtonraphson_results: Measurements, // Tambahkan ini
//     pub newtonraphson_akar: f64,             // Jika ingin simpan akar
// }

#[derive(Clone, Copy)]
pub struct Value {
    pub x: f64,
    pub y: f64,
}

impl Measurements {
    pub fn new() -> Self {
        Self {
            values: Vec::new(),
            max_data_points: 300,
        }
    }

    // Fungsi Newton-Raphson yang akan mengembalikan akar dan riwayat iterasi
    pub fn newton_raphson<F, Fp>(f: F, fp: Fp, x0: f64, tol: f64, max_iter: usize) -> (f64, Vec<Value>)
    where
        F: Fn(f64) -> f64,
        Fp: Fn(f64) -> f64,
    {
        let mut x = x0;
        let mut results_history = Vec::new(); // Menyimpan setiap nilai x pada setiap iterasi
        results_history.push(Value { x: 0.0, y: x0 }); // Tambahkan tebakan awal sebagai iterasi 0

        for i in 0..max_iter {
            let fx = f(x);
            let fpx = fp(x);

            // Hindari pembagian dengan nol atau sangat mendekati nol
            if fpx.abs() < 1e-12 {
                println!("Newton-Raphson: Turunan mendekati nol pada iterasi {}. Menghentikan.", i);
                break;
            }

            let x_new = x - fx / fpx;
            results_history.push(Value { x: (i + 1) as f64, y: x_new }); // Iterasi ke-i+1

            if (x_new - x).abs() < tol {
                println!("Newton-Raphson: Konvergensi tercapai pada iterasi {}. x = {:.8}", i + 1, x_new);
                x = x_new;
                break;
            }
            x = x_new;

            if i == max_iter - 1 {
                println!("Newton-Raphson: Iterasi maksimum tercapai. Hasil terakhir: x = {:.8}", x);
            }
        }
        (x, results_history)
    }
    
    pub fn add_value(&mut self, value: Value) {
        self.values.push(value);
        if self.values.len() > self.max_data_points {
            self.remove_oldest_value();
        }
    }

    pub fn clear_values(&mut self) {
        self.values.clear();
    }

    fn remove_oldest_value(&mut self) {
        if !self.values.is_empty() {
            self.values.remove(0);
        }
    }

    pub fn set_max_data_points(&mut self, max_points: usize) {
        self.max_data_points = max_points;
        while self.values.len() > self.max_data_points {
            self.remove_oldest_value();
        }
    }
}