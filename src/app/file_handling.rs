use egui::epaint::tessellator::Path;
use rfd::FileDialog;
use std::{path::PathBuf, ffi::OsStr};

use super::cmdmod::{
    openfile, savetofile
};

pub fn savef(
    mut last_save_path: Option<PathBuf>,
    text_to_save: String,
    mut code_editor_text_lenght: usize,
) -> (usize, Option<PathBuf>) {
    if last_save_path.clone().is_none() {
        let files = FileDialog::new()
            .set_title("Save")
            .set_directory("/")
            .save_file();
        last_save_path = files.clone();
        savetofile(last_save_path.clone(), text_to_save.clone());
        code_editor_text_lenght = text_to_save.len();
    } else if code_editor_text_lenght <= text_to_save.len() {
        savetofile(last_save_path.clone(), text_to_save.clone());
        code_editor_text_lenght = text_to_save.len();
    }
    return (code_editor_text_lenght, last_save_path);
}

pub fn openf(
    mut last_save_path: Option<PathBuf>,
    mut code_editor_text_lenght: usize,
    mut code_editor_code: String,
) -> (usize, String, Option<PathBuf>) {
    let files = FileDialog::new()
        .set_title("Open")
        .set_directory("/")
        .pick_file();
    if files.clone().is_some() {
        last_save_path = files.clone();
        code_editor_code = openfile(last_save_path.clone());
        code_editor_text_lenght = code_editor_code.len();

        return (code_editor_text_lenght, code_editor_code, last_save_path);
    } else {
        return (code_editor_text_lenght, code_editor_code, last_save_path);
    }
}

pub fn savefas(
    mut last_save_path: Option<PathBuf>,
    mut code_editor_text_lenght: Option<usize>,
    code_editor_code: String,
) -> (Option<usize>, Option<PathBuf>) {
    
    let files = FileDialog::new()
        .set_title("Save as")
        .set_directory("/")
        .save_file();
    if files.clone().is_some() {
        last_save_path = files.clone();
        savetofile(files.clone(), code_editor_code.clone());
        code_editor_text_lenght = Some(code_editor_code.len());
        return ((code_editor_text_lenght), last_save_path);
    } else {
        return (None, None);
    }
}

pub fn savefas_w(
    file_dialog_title: &str,
    code_editor_code: String,
) {
    let files = FileDialog::new()
        .set_title(&file_dialog_title)
        .set_directory("/")
        .add_filter("Marcide workspace", &["m-workspace"])
        .save_file();
    if files.clone().is_some() {
        savetofile(files.clone(), code_editor_code.clone());
    }
}

pub fn openf_w(file_dialog_title: &str) -> (Option<String>, Option<String>) {
    let files = FileDialog::new()
        .set_title(file_dialog_title)
        .set_directory("/")
        .add_filter("Marcide workspace", &["m-workspace"])
        .pick_file();

    if let Some(files) = files {
        let code_editor_code = openfile(Some(files.clone()));
        //excetution date 2024 13 69
        return (Some(code_editor_code), Some(files.file_name().to_owned().unwrap().to_string_lossy().to_string()));
    } else {
        return (None, None);
    }
}