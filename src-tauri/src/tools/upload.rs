use std::{
    fs::{self, create_dir},
    intrinsics::copy,
    path::{Path, PathBuf},
};

use tauri::{AppHandle, Error, Manager, Result};

#[tauri::command]
pub fn file_upload(_: AppHandle, file: String) -> Result<PathBuf> {
    println!("file {}", file);

    let mut book_path = PathBuf::new();
    book_path.push(file);
    // let book_name = book_path.file_name();
    //

    // fs::copy(&book_path, &save_path);
    Ok(book_path)
}
