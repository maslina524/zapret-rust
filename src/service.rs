use std::ffi::OsStr;
use std::process::{Command, Stdio};
use std::io::{self};
use std::fs;
use std::env;
use sysinfo::{System};
use windows_service::service::ServiceState;
use lazy_static::lazy_static;
use std::fs::OpenOptions;
use std::io::Write;
use winreg::RegKey;
use winreg::enums::*;
use std::ptr;
use winapi::um::winuser::{SendMessageTimeoutW, HWND_BROADCAST, WM_SETTINGCHANGE, SMTO_ABORTIFHUNG};
use std::os::windows::ffi::OsStrExt;


use crate::bat;

const RUST_ZAPRET_VER: &str = "0.1.1";
const ZAPRET_VER: &str = "1.9.6";
const REPO_URL: &str = "https://github.com/maslina524/zapret-rust";
const ZAPRET_URL: &str = "https://github.com/Flowseal/zapret-discord-youtube";
const SRVCNAME: &str = "zapret-rust";
const GAME_FILTER_MN: u8 = 12; // only for test

const PATH_SEP: &str = if cfg!(windows) { ";" } else { ":" };

lazy_static! {
    // static ref ABSOLUTE_PATH: String = String::from(r"C:\Users\lukki\Documents\zapret-rust\");
    static ref ABSOLUTE_PATH: String = {
        match env::current_exe() {
            Ok(exe_path) => {
                if let Some(exe_dir) = exe_path.parent() {
                    format!(r"{}\", exe_dir.to_string_lossy().to_string())
                } else {
                    bat::pause();
                    String::from("Error!")
                }
            }
            Err(e) => {
                bat::pause();
                format!("Error: {e}")
            }
        }
    };
    static ref CONFIGS_PATH: String = format!("{}configs\\", ABSOLUTE_PATH.to_string());
    static ref BIN_PATH: String = format!("{}bin\\", ABSOLUTE_PATH.to_string());
    static ref LISTS_PATH: String = format!("{}lists\\", ABSOLUTE_PATH.to_string());
}

pub fn menu() {
    loop {
        bat::cls();

        println!("ZAPRET SERVICE MANAGER");
        println!("RUSTZAPRET v{} | {}", RUST_ZAPRET_VER, REPO_URL);
        println!("ZAPRET v{} | {}", ZAPRET_VER, ZAPRET_URL);
        println!("----------------------------------------\n");
        
        println!(":: SERVICE");
        println!("   1. Install Service");
        println!("   2. Remove Services");
        println!("   3. Check Status\n");

        println!(":: SPECIAL");
        println!("   4. Add to PATH       [{}]", if get_path_vec().contains(&ABSOLUTE_PATH.to_string()) { "IN PATH" } else { "NOT IN PATH" });
        println!("   5. Remove from PATH");
        println!("   6. Add Ip/Domain\n");
        
        // println!(":: SETTINGS");
        // println!("   4. Game Filter");
        // println!("   5. IPSet Filter");
        // println!("   6. Auto-Update Check\n");
        
        // println!(":: UPDATES");
        // println!("   7. Update IPSet List");
        // println!("   8. Update Hosts File");
        // println!("   9. Check for Updates\n");
        
        // println!(":: TOOLS");
        // println!("   10. Run Diagnostics");
        // println!("   11. Run Tests\n");

        println!("----------------------------------------");
        println!("   0. Exit\n");
        
        println!("Select option (0-3):");
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read line");

        match input.trim() {
            // SERVICE
            "1" =>  service_install(),
            "2" =>  service_remove(),
            "3" =>  service_status(),
            // SPECIAL
            "4" =>  add_to_path(),
            "5" =>  remove_from_path(),
            "6" =>  add_domain(None),
            // "7" =>  unavaible(),
            // "8" =>  unavaible(),
            // "9" =>  unavaible()
            // "10" => unavaible(),
            // "11" => unavaible(),
            "0" =>  return,
            _ => {}
        }
    }
}

fn replace_placeholders(data: String) -> String {
    data.replace("%GameFilter%", &GAME_FILTER_MN.to_string())
        .replace("%LISTS%", &LISTS_PATH)
        .replace("%BIN%", &BIN_PATH)
}

fn get_path_vec() -> Vec<String> {
    get_path_string().split(PATH_SEP).map(String::from).collect()
}

fn get_path_string() -> String {
    env::var("PATH").unwrap_or_default()
}

fn println_green(text: &str) {
    println!("\x1b[32m{text}\x1b[0m")
}

fn println_red(text: &str) {
    println!("\x1b[31m{text}\x1b[0m")
}

fn tcp_start() -> bool {
    // Not for a separate use, use `tcp_enable`
    Command::new("netsh")
        .args(["interface", "tcp", "set", "global", "timestamps=enabled"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn tcp_enable() -> bool {
    let output = Command::new("netsh")
        .args(["interface", "tcp", "show", "global"])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output();

    let output = match output {
        Ok(o) if o.status.success() => o,
        _ => {
            return tcp_start()
        }
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let found = stdout.lines().any(|line| {
        let lower = line.to_lowercase();
        lower.contains("timestamps") && lower.contains("enabled")
    });

    if found {
        true
    } else {
        tcp_start()
    }
}

fn test_service(name: &str, soft: bool) {
    if let Ok(s) = bat::get_service_status(SRVCNAME) {
        if let Some(state) = s {
            match state {
                ServiceState::Running => {
                    if soft {
                        println!("\"{}\" is ALREADY RUNNING as service, choose `Remove Services` first.", name);
                        bat::pause();
                        return;
                    } else {
                        println!("\"{}\" service is RUNNING.", name)
                    }
                },
                ServiceState::StopPending => {
                    println!("\x1b[33m\"{}\" is STOP_PENDING, that may be caused by a conflict with another bypass.\x1b[0m", name)
                }
                _ => {
                    if !soft {
                        println!("\"{}\" service is NOT running.", name)
                    }
                }
            }
        }
    }
}

pub fn service_status() {
    bat::cls();

    test_service(SRVCNAME, false);
    test_service("WinDivert", false);

    println!();
    let mut system = System::new_all();
    system.refresh_all();
    
    match system.processes_by_name(OsStr::new("winws.exe")).next().is_some() {
        true => println_green("Bypass (winws.exe) is RUNNING."),
        false => println_red("Bypass (winws.exe) is NOT running.")
    }

    bat::pause();
}

pub fn service_remove() {
    bat::cls();
    let _ = bat::stop_service(SRVCNAME);
    if let Err(_) = bat::delete_service(SRVCNAME) {
        println!("Service \"{}\" is not installed.", SRVCNAME)
    }

    bat::kill_winws();
    bat::remove_windivert();

    bat::pause();
}

pub fn service_install() {
    bat::cls();

    println!("Pick one of the options:");
    let mut configs: Vec<String> = vec![];

    let entries = match fs::read_dir(&CONFIGS_PATH.to_string()) {
        Ok(entries) => entries,
        Err(e) => {
            eprintln!("Failed to read directory ({})", e);
            eprintln!("{}", &CONFIGS_PATH.to_string());
            bat::pause();
            return;
        }
    };
    for entry in entries {
        match entry {
            Ok(entry) => {
                if let Some(name) = entry.file_name().to_str() {
                    configs.push(name.to_string());
                }
            }
            Err(e) => eprintln!("Err ({})", e),
        }
    }

    configs.sort();
    for (i, config) in configs.iter().enumerate() {
        println!("{}: {:?}", i + 1, config);
    }
    println!("Input file index (number):");
    let mut choice = String::new();
    io::stdin().read_line(&mut choice).expect("Failed to read line");

    if choice.trim() == "" {
        println!("The choice is empty, exiting...");
        bat::pause();
        return;
    }

    let selected_file: &str;
    match choice.trim().parse::<usize>() {
        Ok(c) => {
            if c >= 1 && c <= configs.len() {
                selected_file = configs[c - 1].as_str()
            } else {
                println!("Incorrect choice (1 - {})", configs.len());
                bat::pause();
                return;
            }
        },
        Err(e) => {
            println!("Error parsing choice ({})", e);
            bat::pause();
            return;
        }
    }

    install_selected_file(selected_file);
}

pub fn install_selected_file(selected_file: &str) {
    // println!("{selected_file}");
    // We process the selected file and get a command ready for execution.
    let config_data: String;
    if let Ok(c) = fs::read_to_string(format!("{}{}", &CONFIGS_PATH.to_string(), selected_file)) {
        config_data = c;
    } else {
        eprintln!("Failed to read file `{}`", format!("{}{}", &CONFIGS_PATH.to_string(), selected_file));
        bat::pause();
        return;
    }

    tcp_enable();

    if let Err(e) = bat::stop_service(SRVCNAME) {
        if let windows_service::Error::Winapi(ref err) = e {
            let code = err.raw_os_error().unwrap_or(0);
            if code != 1062 && code != 1060 {
                eprintln!("Couldn't stop service: {e}");
                eprintln!("WinAPI err code: {}", err.raw_os_error().unwrap_or(0))
            }
        }
    }

    if let Err(e) = bat::delete_service(SRVCNAME) {
        eprintln!("Couldn't delete the service: {e}")
    }

    let mut args: Vec<String> = replace_placeholders(config_data).split(" --").map(str::to_string).collect();
    for (i, arg) in args.clone().iter().enumerate() {
        if i != 0 {
            args.insert(i, format!("--{arg}"))
        }
    }

    let exe_path = format!("{}winws.exe", BIN_PATH.to_string());
    if !std::path::Path::new(&exe_path).exists() {
        eprintln!("Executable not found: {exe_path}");
        bat::pause();
        return;
    }

    if let Err(e) = bat::create_service(SRVCNAME, &exe_path, args, "zapret-rust") {
        eprintln!("Couldn't create the service: {e}")
    }

    if let Err(e) = bat::reg_add(SRVCNAME, "ActiveConfig", selected_file) {
        eprintln!("Couldn't add active config value: {e}")
    }

    if let Err(e) = bat::start_service(SRVCNAME) {
        eprintln!("Couldn't start the service: {e}")
    }

    bat::pause();
}

pub fn add_domain(selected: Option<String>) {
    bat::cls();

    let mut domain = String::new();
    match selected {
        Some(d) => domain = d,
        None => {
            println!("Enter the IP/domain:");
            io::stdin().read_line(&mut domain).expect("Failed to read line");
            domain = domain.trim().to_string();
        }
    }
    let file: Result<fs::File, io::Error> = OpenOptions::new()
        .write(true)
        .append(true)
        .open(format!(r"{}lists\list-general.txt", ABSOLUTE_PATH.to_string()));

    match file {
        Ok(mut f) => {
            if let Err(e) = writeln!(f, "{domain}") {
                eprintln!("Couldn't write the IP/domain to the file: {e}")
            } else {
                println!("Domain `{domain}` successfully added!\n");
                println!(r"Restart the service to apply the changes? [Y\n]");
                let mut choice = String::new();
                io::stdin().read_line(&mut choice).expect("Failed to read line");
                choice = choice.trim().to_lowercase();
                println!();

                if choice != "n" && choice != "no" {
                    bat::kill_winws();
                    bat::remove_windivert();
                    match bat::start_service(SRVCNAME) {
                        Ok(()) => println_green("The service has been successfully restarted!\n"),
                        Err(e) => println_red(&format!("Couldn't restart the service: {e}\n")),
                    }
                }

                bat::pause();
            }
        },
        Err(e) => {
            eprintln!("Couldn't open the file: {e}")
        }
    }
    
    
}

fn add_to_path() {
    bat::cls();

    let current_path = get_path_string();
    let new_dir = ABSOLUTE_PATH.to_string();

    if get_path_vec().contains(&new_dir) {
        println!("The path `{new_dir}` is already present in the PATH! (session)");
    } else {
        println!("Adding `{new_dir}` to the user's permanent PATH...");
    }

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let env_key = match hkcu.open_subkey_with_flags("Environment", KEY_READ | KEY_WRITE) {
        Ok(key) => key,
        Err(e) => {
            eprintln!("Couldn't open Environment registry key: {}", e);
            bat::pause();
            return;
        }
    };

    let reg_path: String = env_key.get_value("PATH").unwrap_or_default();
    let mut paths: Vec<String> = reg_path.split(PATH_SEP).map(String::from).collect();

    if !paths.contains(&new_dir) {
        paths.push(new_dir.clone());
        let new_reg_path = paths.join(PATH_SEP);

        if let Err(e) = env_key.set_value("PATH", &new_reg_path) {
            eprintln!("Reg err: {}", e);
            bat::pause();
            return;
        }

        let msg: Vec<u16> = OsStr::new("Environment").encode_wide().chain(Some(0)).collect();
        unsafe {
            SendMessageTimeoutW(
                HWND_BROADCAST,
                WM_SETTINGCHANGE,
                0,
                msg.as_ptr() as isize,
                SMTO_ABORTIFHUNG,
                5000,
                ptr::null_mut(),
            );
        }

        println_green("The directory was successfully added to the PATH!");
    } else {
        println!("The path `{new_dir}` is already present in the PATH! (reg)");
    }

    unsafe { env::set_var("PATH", format!("{current_path}{PATH_SEP}{new_dir}")) };

    bat::pause();
}

fn remove_from_path() {
    bat::cls();

    let target_dir = ABSOLUTE_PATH.to_string();
    println!("Removing `{target_dir}` from the user's permanent PATH...");

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let env_key = match hkcu.open_subkey_with_flags("Environment", KEY_READ | KEY_WRITE) {
        Ok(key) => key,
        Err(e) => {
            eprintln!("Couldn't open Environment registry key: {}", e);
            bat::pause();
            return;
        }
    };

    let reg_path: String = match env_key.get_value("PATH") {
        Ok(p) => p,
        Err(_) => {
            println!("The PATH variable is missing from the registry.");
            bat::pause();
            return;
        }
    };

    let mut paths: Vec<String> = reg_path.split(PATH_SEP).map(String::from).collect();
    let original_len = paths.len();

    paths.retain(|p| p != &target_dir);

    if paths.len() == original_len {
        println!("The `{target_dir}` path was not found in the PATH.");
    } else {
        let new_reg_path = paths.join(PATH_SEP);
        if let Err(e) = env_key.set_value("PATH", &new_reg_path) {
            eprintln!("Reg err: {}", e);
            bat::pause();
            return;
        }

        let msg: Vec<u16> = OsStr::new("Environment").encode_wide().chain(Some(0)).collect();
        unsafe {
            SendMessageTimeoutW(
                HWND_BROADCAST,
                WM_SETTINGCHANGE,
                0,
                msg.as_ptr() as isize,
                SMTO_ABORTIFHUNG,
                5000,
                ptr::null_mut(),
            );
        }

        println_green("The path has been successfully deleted from the user's permanent PATH.");

        if let Ok(new_path) = env_key.get_value::<String, _>("PATH") {
            unsafe { env::set_var("PATH", &new_path) };
        } else {
            unsafe { env::remove_var("PATH") };
        }
    }

    bat::pause();
}
