use winreg::enums::*;
use winreg::RegKey;
use std::env;
use std::io;
pub fn add_win_ctx(icon_bytes : &[u8]) {
    let path = format!("{}", env::current_dir().unwrap().display());
                let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
                let environment_key = hklm.open_subkey_with_flags(
                    r"SYSTEM\CurrentControlSet\Control\Session Manager\Environment",
                    KEY_READ | KEY_WRITE,
                );

                match environment_key {
                    Ok(key) => {
                        // Read the existing PATH value
                        let mut path_value: String = key.get_value("PATH").unwrap_or_default();

                        // Append the new path to the existing value
                        if !path_value.is_empty() {
                            path_value.push(';');
                        }
                        path_value.push_str(&path);

                        // Set the modified PATH value
                        key.set_value("PATH", &path_value)
                            .expect("Failed to set value");

                        // Notify the user about the updated PATH
                    }
                    Err(err) => {
                        eprintln!("Error: {}", err);
                    }
                }

                if let Ok(exe_path) = env::current_exe() {
                    let app_path = exe_path;
                    let _ = std::process::Command::new("reg")
                        .args([
                            "add",
                            "HKEY_CLASSES_ROOT\\*\\shell\\Open file with Marcide\\command",
                            "/ve",
                            "/d",
                            &format!("\"{}\" \"%1\"", app_path.display()),
                            "/f",
                        ])
                        .output()
                        .expect("Failed to execute command");
                    let path = format!(
                        "C:\\Users\\{}\\AppData\\Roaming\\Marcide\\data\\icon.ico",
                        env::var("USERNAME").unwrap()
                    );
                    let mut output_file =
                        std::fs::File::create(path).expect("Failed to create file");
                    io::Write::write_all(&mut output_file, icon_bytes)
                        .expect("Failed to write to file");
                    let icon_path = format!(
                        r#""C:\\Users\\{}\\AppData\\Roaming\\Marcide\\data\\icon.ico""#,
                        env::var("USERNAME").unwrap()
                    );
                    let _ = std::process::Command::new("reg")
                        .args([
                            "add",
                            "HKEY_CLASSES_ROOT\\*\\shell\\Open file with Marcide",
                            "/v",
                            "Icon",
                            "/d",
                            &icon_path.to_string(),
                            "/f",
                        ])
                        .output()
                        .expect("Failed to execute command");
                }
}
pub fn remove_win_ctx() {
    let path = format!("{}", env::current_dir().unwrap().display());
                let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
                let environment_key = hklm.open_subkey_with_flags(
                    r"SYSTEM\CurrentControlSet\Control\Session Manager\Environment",
                    KEY_READ | KEY_WRITE,
                );

                match environment_key {
                    Ok(key) => {
                        // Read the existing PATH value
                        let path_value: String = key.get_value("Path").unwrap_or_default();

                        // Split the existing paths by semicolon
                        let paths: Vec<_> = path_value.split(';').collect();

                        // Create a new PATH value without the path to remove
                        let new_paths: Vec<_> = paths.into_iter().filter(|&p| p != path).collect();
                        let new_path_value = new_paths.join(";");

                        // Set the modified PATH value
                        key.set_value("Path", &new_path_value)
                            .expect("Failed to set value");

                        // Notify the user about the updated PATH
                        println!("Updated PATH: {}", new_path_value);
                    }
                    Err(err) => {
                        eprintln!("Error: {}", err);
                    }
                }
                let _ = std::process::Command::new("reg")
                    .args([
                        "delete",
                        "HKEY_CLASSES_ROOT\\*\\shell\\Open file with Marcide",
                        "/f",
                    ])
                    .output()
                    .expect("Failed to execute command");
}



