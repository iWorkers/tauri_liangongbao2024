// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]  

mod get_qrcode_url;
use get_qrcode_url::{get_qrcode_url, monitor_wechat_scan};
use tokio;
use std::result::Result::Ok;
use anyhow:: Result;




#[tokio::main]
async fn main() -> Result<()> {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![qrcode_url,my_custom_command])
        .run(tauri::generate_context!())
        .expect("failed to run app");
    
    Ok(())
    
}

//.run(tauri::generate_context!())
#[tauri::command]
async fn qrcode_url() -> Result<String,  ()> {
    get_qrcode_url().await 
    
}


#[tauri::command]
async fn my_custom_command(app_handle: tauri::AppHandle) {
    monitor_wechat_scan(&app_handle).await
}