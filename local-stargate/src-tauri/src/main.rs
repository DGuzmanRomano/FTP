// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use axum::{
    body::Bytes,
    http::{header, HeaderMap, HeaderName, Method, StatusCode},
    routing::post,
    Router,
};
use std::net::SocketAddr;
use std::path::Path;
use tauri::AppHandle;
use tauri_plugin_dialog::{DialogExt, FilePath}; // Keep this import
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::sync::oneshot;
use tower_http::cors::{Any, CorsLayer};

const PORT: u16 = 3033;

#[tauri::command]
async fn open_file_dialog(app_handle: AppHandle) -> Result<String, String> {
    let (tx, rx) = oneshot::channel();
    app_handle
        .dialog()
        .file()
        .pick_file(move |file_path_option: Option<FilePath>| {
            if let Some(path) = file_path_option {
                // --- THIS LINE IS FIXED ---
                // The correct way to convert the FilePath to a string.
                tx.send(Ok(path.to_string())).unwrap();
            } else {
                tx.send(Err("No file was selected.".into())).unwrap();
            }
        });

    rx.await.unwrap_or(Err("Dialog operation failed.".into()))
}

#[tauri::command]
async fn send_file(file_path: String, target_ip: String) -> Result<(), String> {
    println!("[CLIENT] Preparing to send: {} to {}", file_path, target_ip);

    let path = Path::new(&file_path);
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("unknown_file");

    let file_bytes = tokio::fs::read(&file_path)
        .await
        .map_err(|e| format!("Failed to read file: {}", e))?;

    let client = reqwest::Client::new();
    let url = format!("http://{}:{}/upload", target_ip, PORT);

    println!("[CLIENT] Sending POST request to {}", url);

    let response = client
        .post(&url)
        .header("X-File-Name", file_name)
        .body(file_bytes)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    if response.status().is_success() {
        println!("[CLIENT] File sent successfully.");
        Ok(())
    } else {
        let error_msg = format!("Server responded with error: {}", response.status());
        println!("[CLIENT] {}", error_msg);
        Err(error_msg)
    }
}

async fn upload_handler(headers: HeaderMap, body: Bytes) -> Result<(), StatusCode> {
    let file_name = headers
        .get("X-File-Name")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("received_file.dat");

    let downloads_dir = match dirs::download_dir() {
        Some(path) => path,
        None => {
            eprintln!("[SERVER] Could not find the Downloads directory.");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let dest_path = downloads_dir.join(file_name);
    println!("[SERVER] Receiving file, saving to: {:?}", dest_path);

    let mut file = File::create(&dest_path)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    file.write_all(&body)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    println!("[SERVER] Successfully saved file: {:?}", dest_path);
    Ok(())
}

#[tokio::main]
async fn main() {
    tokio::spawn(async {
        let cors = CorsLayer::new()
            .allow_methods([Method::POST])
            .allow_headers([header::CONTENT_TYPE, HeaderName::from_static("x-file-name")])
            .allow_origin(Any);

        let app = Router::new().route("/upload", post(upload_handler));
        let addr = SocketAddr::from(([0, 0, 0, 0], PORT));
        println!("[SERVER] Listening on http://{}", addr);

        let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
        axum::serve(listener, app.layer(cors)).await.unwrap();
    });

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![send_file, open_file_dialog])
        // --- THIS LINE IS FIXED ---
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
