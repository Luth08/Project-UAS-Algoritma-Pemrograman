// src/db.rs
#![allow(unused_imports)] // Mengizinkan import yang tidak digunakan untuk kompatibilitas

use mongodb::{Client, options::ClientOptions, Database, bson::doc};
use std::error::Error;
use mongodb::bson::Document;
use futures::stream::TryStreamExt;
use mongodb::bson::DateTime; // Diperlukan untuk DateTime::now()
use chrono::Utc; // Diperlukan untuk konversi to_chrono() di database_screen.rs
// HAPUS BARIS INI: use mongodb::bson::datetime::DateTimeExt; 

pub async fn connect_db() -> Result<Database, Box<dyn Error>> {
    let client_uri = "mongodb://localhost:27017";
    let options = ClientOptions::parse(client_uri).await?;
    let client = Client::with_options(options)?;
    Ok(client.database("amitdb"))
}

pub async fn insert_photodiode_data(db: &Database, photodiode_value: f64) -> mongodb::error::Result<()> {
    let collection = db.collection("photodiode_data");
    let doc = doc! { 
        "photodiode_value": photodiode_value, 
        "timestamp": mongodb::bson::DateTime::now()
    };
    collection.insert_one(doc).await?;
    Ok(())
}

pub async fn get_all_photodiode_data(db: &Database) -> mongodb::error::Result<Vec<Document>> {
    let collection = db.collection::<Document>("photodiode_data");
    let mut cursor = collection.find(doc! {}).await?;
    let mut results = Vec::new();
    while let Some(doc) = cursor.try_next().await? {
        results.push(doc);
    }
    Ok(results)
}

pub async fn insert_newton_raphson_result(db: &Database, akar: f64, iterations_history: Vec<f64>) -> mongodb::error::Result<()> {
    let collection = db.collection("newton_raphson_results");
    let doc = doc! {
        "akar_terakhir": akar,
        "riwayat_iterasi": iterations_history,
        "timestamp": mongodb::bson::DateTime::now()
    };
    collection.insert_one(doc).await?;
    Ok(())
}

pub async fn get_all_newton_raphson_results(db: &Database) -> mongodb::error::Result<Vec<Document>> {
    let collection = db.collection::<Document>("newton_raphson_results");
    let mut cursor = collection.find(doc! {}).await?;
    let mut results = Vec::new();
    while let Some(doc) = cursor.try_next().await? {
        results.push(doc);
    }
    Ok(results)
}