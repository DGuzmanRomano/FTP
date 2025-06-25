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
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tower_http::cors::{Any, CorsLayer};

const PORT: u16 = 3033; // Using a slightly uncommon port

// This is the Tauri command that will be called from the frontend to SEND a file.
#[tauri::command]
async fn file_dropped(file_path: String, target_ip: String) -> Result<(), String> {
    println!(
        "[CLIENT] Preparing to send file: {} to IP: {}",
        file_path, target_ip
    );

    let path = Path::new(&file_path);
    let file_name = match path.file_name() {
        Some(name) => name.to_str().unwrap_or("unknown_file"),
        None => "unknown_file",
    };

    let file_bytes = match tokio::fs::read(&file_path).await {
        Ok(bytes) => bytes,
        Err(e) => return Err(format!("Failed to read file: {}", e)),
    };

    let client = reqwest::Client::new();
    let url = format!("http://{}:{}/upload", target_ip, PORT);

    println!("[CLIENT] Sending POST request to {}", url);

    let response = client
        .post(&url)
        .header("X-File-Name", file_name)
        .body(file_bytes)
        .send()
        .await;

    match response {
        Ok(res) if res.status().is_success() => {
            println!("[CLIENT] File sent successfully.");
            Ok(())
        }
        Ok(res) => {
            let error_msg = format!(
                "Failed to send file. Server responded with: {}",
                res.status()
            );
            println!("[CLIENT] {}", error_msg);
            Err(error_msg)
        }
        Err(e) => {
            let error_msg = format!("Failed to send file: {}", e);
            println!("[CLIENT] {}", error_msg);
            Err(error_msg)
        }
    }
}

// This function sets up and runs the Axum web server to RECEIVE files.
async fn run_server() {
    let cors = CorsLayer::new()
        .allow_methods([Method::POST])
        .allow_headers([
            header::CONTENT_TYPE,
            HeaderName::from_static("x-file-name"),
        ])
        .allow_origin(Any);

    let app = Router::new().route("/upload", post(upload_handler)).layer(cors);

    let addr = SocketAddr::from(([0, 0, 0, 0], PORT));
    println!("[SERVER] Listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// This is the handler for the /upload route. It saves the received file.
async fn upload_handler(headers: HeaderMap, body: Bytes) -> Result<(), StatusCode> {
    let file_name = if let Some(header_value) = headers.get("X-File-Name") {
        header_value.to_str().unwrap_or("received_file")
    } else {
        "received_file"
    };

    let downloads_dir = match dirs::download_dir() {
        Some(path) => path,
        None => {
            eprintln!("[SERVER] Could not find downloads directory.");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let dest_path = downloads_dir.join(file_name);
    println!("[SERVER] Receiving file, will save to: {:?}", dest_path);

    let mut file = match File::create(&dest_path).await {
        Ok(f) => f,
        Err(e) => {
            eprintln!("[SERVER] Failed to create file: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    if let Err(e) = file.write_all(&body).await {
        eprintln!("[SERVER] Failed to write to file: {}", e);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    println!("[SERVER] Successfully saved file: {:?}", dest_path);
    Ok(())
}

// --- THIS IS THE FIX ---
// Add the `#[tokio::main]` attribute to initialize the async runtime.
#[tokio::main]
async fn main() {
    // Start the Axum server in a background thread
    tokio::spawn(async {
        run_server().await;
    });

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![file_dropped])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
