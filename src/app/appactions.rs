use std::path::PathBuf;
use rfd::FileDialog;


use super::{cmdmod::{
    finder, mkdir, openfile, rmdir, runfile, savetofile, terminalr
}, code_editor};

pub fn savef(mut last_save_path : Option<PathBuf>, text_to_save : String, mut code_editor_text_lenght : usize) -> usize {
    if last_save_path.clone().is_none() {
        let files = FileDialog::new()
            .set_title("Save")
            .set_directory("/")
            .save_file();
        last_save_path = files.clone();
        savetofile(last_save_path.clone(), text_to_save.clone());
        code_editor_text_lenght = text_to_save.len();
    } else if code_editor_text_lenght <= text_to_save.len() {
        savetofile(last_save_path, text_to_save.clone());
        code_editor_text_lenght = text_to_save.len();
    }
    return code_editor_text_lenght;
}

pub fn openf(mut last_save_path : Option<PathBuf>, mut code_editor_text_lenght : usize, mut code_editor_code : String) -> (usize, String, Option<PathBuf>) {
    let files = FileDialog::new()
        .set_title("Open")
        .set_directory("/")
        .pick_file();
    if files.clone().is_some() {
        last_save_path = files.clone();
        code_editor_code = openfile(last_save_path.clone());
        code_editor_text_lenght = code_editor_code.len();

        return (code_editor_text_lenght, code_editor_code, last_save_path);
    }
    else {
        return (code_editor_text_lenght, code_editor_code, last_save_path);
    }
}

pub fn savefas(mut last_save_path : Option<PathBuf>, mut code_editor_text_lenght : Option<usize>, mut code_editor_code : String) -> (Option<usize>, Option<PathBuf>) {
    let mut code_editor_text_lenght = code_editor_text_lenght.unwrap();
    let files = FileDialog::new()
        .set_title("Save as")
        .set_directory("/")
        .save_file();
    if files.clone().is_some() {
        last_save_path = files.clone();
        savetofile(files.clone(), code_editor_code.clone());
        code_editor_text_lenght = code_editor_code.len();
        return (Some(code_editor_text_lenght), last_save_path);
    }
    else {
        return (None,None);
    }
}