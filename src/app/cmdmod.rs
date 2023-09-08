use dirs::home_dir;
use std::fs::File;
use std::fs::OpenOptions;
use std::io;
use std::io::{Read, Write};
use std::path::PathBuf;
use windows_sys::w;
use windows_sys::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONERROR, MB_OK};

pub fn terminalr(path: Option<PathBuf>) -> std::process::Output {
    let command_to_be_excecuted = format!("{}", path.unwrap().display());
    let cmdcomm = std::process::Command::new("cmd")
        .arg("/C")
        .arg(command_to_be_excecuted)
        .output();
    match cmdcomm {
        Ok(mut ok) => {
            if ok.stdout.is_empty() {
                ok.stdout = ok.stderr.clone();
            };
            ok
        }
        Err(_) => {
            unsafe {
                MessageBoxW(0,  w!("Troubleshoot : Did you add python / lua to system variables?\n(as py | as lua)"), w!("Fatal error"), MB_ICONERROR | MB_OK)
            };
            cmdcomm.unwrap()
        }
    }
}

pub fn finder(text: String, to_find: String) -> io::Result<Vec<usize>> {
    //let reader = BufReader::new(file);
    let mut line_numbers: Vec<usize> = Vec::new();

    for (line_number, line) in text.lines().enumerate() {
        let line_content = line;
        if line_content.contains(&to_find) {
            line_numbers.push(line_number + 1); // Add 1 to convert zero-based index to line number
        }
    }

    Ok(line_numbers)
}
pub fn mkdir() {
    let mut command = String::new();
    if let Some(home_dir) = home_dir() {
        command = format!("mkdir {}\\%marcide.temp%", home_dir.display())
    }
    let cmdcomm = std::process::Command::new("cmd")
        .arg("/C")
        .arg(command)
        .status();
    match cmdcomm {
        Ok(_) => {
            println!("Failed to excecute command!")
        }
        Err(_) => {}
    }
}
pub fn rmdir() {
    let mut command = String::new();
    if let Some(home_dir) = home_dir() {
        command = format!("rmdir /s /q {}\\%marcide.temp%", home_dir.display())
    }
    let cmdcomm = std::process::Command::new("cmd")
        .arg("/C")
        .arg(command)
        .status();
    match cmdcomm {
        Ok(_) => {
            println!("Failed to excecute command!")
        }
        Err(_) => {}
    }
}
pub fn runfile(path: Option<PathBuf>, mut language: String) -> std::process::Output {
    //first check if the env variables are set
    let env = std::process::Command::new(language.clone()).output();
    match env {
        Ok(_) => {
            /*Env variable found, run py in quiet mode*/
            if language == "py" {
                language = "py -q".to_owned()
            }
        }
        Err(_) => {
            //notify user
            println!("env varaible not found");
            unsafe {
                MessageBoxW(0,  w!("Troubleshoot : did you add the compiler to the PATH system variable?\nDid you check the spelling of which programming language you want to syntax?"), w!("Fatal error"), MB_ICONERROR | MB_OK)
            };
        }
    }
    let command_to_be_excecuted = format!(
        "{} {}",
        /*lang if first asked so we can decide which script compiler needs to be run ie: py test.py or lua test.lua */
        language,
        path.unwrap().display()
    );
    let cmdcomm = std::process::Command::new("cmd")
        .arg("/C")
        .arg(command_to_be_excecuted)
        .output();
    match cmdcomm {
        Ok(mut ok) => {
            if ok.stdout.is_empty() {
                ok.stdout = ok.stderr.clone();
            };
            ok
        }
        Err(_) => {
            unsafe {
                MessageBoxW(0,  w!("Troubleshoot : Did you add python / lua to system variables?\n(as py | as lua)"), w!("Fatal error"), MB_ICONERROR | MB_OK)
            };
            cmdcomm.unwrap()
        }
    }
}
pub fn openfile(path: Option<PathBuf>) -> String {
    let mut contents = String::new();
    if let Some(file_path) = path {
        let mut file = File::open(file_path).expect("Failed to open file");

        file.read_to_string(&mut contents)
            .expect("Failed to read file");
    }

    return contents;
}
pub fn savetofile(path: Option<PathBuf>, text: String) {
    if let Some(file_path) = path {
        match OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(file_path.clone())
        {
            Ok(mut file) => match write!(file, "{}", text) {
                Ok(_) => {}
                Err(e) => {
                    println!("Error opening the file : {}", e);
                }
            },
            Err(err) => {
                println!("Err : {}", err);
            }
        };

        // Write some data to the file
    }
}
