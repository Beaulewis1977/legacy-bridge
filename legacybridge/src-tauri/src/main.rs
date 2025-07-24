// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod conversion;

use commands::{
    get_version_info, markdown_to_rtf, rtf_to_markdown, test_connection,
    read_rtf_file, write_markdown_file, read_file_base64, write_file_base64,
    batch_convert_rtf_to_markdown, rtf_to_markdown_pipeline, read_rtf_file_pipeline,
};

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            rtf_to_markdown,
            markdown_to_rtf,
            test_connection,
            get_version_info,
            read_rtf_file,
            write_markdown_file,
            read_file_base64,
            write_file_base64,
            batch_convert_rtf_to_markdown,
            rtf_to_markdown_pipeline,
            read_rtf_file_pipeline
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}