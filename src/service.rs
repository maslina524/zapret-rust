use std::ffi::OsStr;
use std::process::{Command, Stdio};
use std::io::{self};
use std::fs;
use std::env;
use sysinfo::{System};
use windows_service::service::ServiceState;

use crate::bat;

const RUST_ZAPRET_VER: &str = "0.1.0";
const ZAPRET_VER: &str = "1.9.6";
const REPO_URL: &str = "https://www.google.com";
const ZAPRET_URL: &str = "https://github.com/Flowseal/zapret-discord-youtube";
const SRVCNAME: &str = "zapret-rust";
const GAME_FILTER_MN: u8 = 12; // only for test

struct Paths {
    configs: String,
    bin_win: String,
    lists: String
}

impl Paths {
    pub fn new(configs: String, bin_win: String, lists: String) -> Self {
        Self {
            configs: configs,
            bin_win: bin_win,
            lists: lists
        }
    }
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
            "1" =>  service_install(),
            "2" =>  service_remove(),
            "3" =>  service_status(),
            // "4" =>  unavaible(),
            // "5" =>  unavaible(),
            // "6" =>  unavaible(),
            // "7" =>  unavaible(),
            // "8" =>  unavaible(),
            // "9" =>  unavaible(),
            // "10" => unavaible(),
            // "11" => unavaible(),
            "0" =>  return,
            _ => {}
        }
    }
}

fn replace_placeholders(data: String, paths: &Paths) -> String {
    data.replace("%GameFilter%", &GAME_FILTER_MN.to_string())
        .replace("%LISTS%", &paths.lists)
        .replace("%BIN%", &paths.bin_win)
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
    match bat::get_service_status(SRVCNAME) {
        Ok(s) => {
            match s {
                Some(state) => {
                    match state {
                        ServiceState::Running => {
                            if soft {
                                println!("\"{}\" is ALREADY RUNNING as service, choose \"Remove Services\" first.", name);
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
                },
                None => {}
            }
        },
        Err(_) => {}
    }
}

fn service_status() {
    bat::cls();

    test_service(SRVCNAME, false);
    test_service("WinDivert", false);

    println!();
    let mut system = System::new_all();
    system.refresh_all();
    
    match system.processes_by_name(OsStr::new("winws.exe")).next().is_some() {
        true => println!("\x1b[32mBypass (winws.exe) is RUNNING.\x1b[0m"),
        false => println!("\x1b[31mBypass (winws.exe) is NOT running.\x1b[0m")
    }

    bat::pause();
}

fn service_remove() {
    bat::cls();
    let _ = bat::stop_service(SRVCNAME);
    match bat::delete_service(SRVCNAME) {
        Ok(_) => {},
        Err(_) => println!("Service \"{}\" is not installed.", SRVCNAME)
    }

    Command::new("taskkill")
        .args(&["/F", "/IM", "winws.exe"])
        .status()
        .unwrap();
    
    bat::remove_windivert();

    bat::pause();
}

fn service_install() {
    bat::cls();

    let mut absolute_path: String = String::new();
    if let Ok(exe_path) = env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            absolute_path = exe_dir
                .canonicalize()
                .unwrap_or_else(|_| exe_dir.to_path_buf())
                .to_string_lossy()
                .to_string();
        }
    }
    absolute_path = format!("{}\\", absolute_path);
    if absolute_path.starts_with("\\\\?\\") {
        absolute_path = absolute_path[4..].to_string();
    }
    println!("{}", absolute_path);

    let paths = Paths::new(
        format!("{}configs\\", absolute_path), 
        format!("{}bin\\", absolute_path), 
        format!("{}lists\\", absolute_path)
    );
    //println!("{}\nconfigs: {}\nbin: {}\nlists: {}\n", absolute_path, paths.configs, paths.bin_win, paths.lists);

    println!("Pick one of the options:");
    let mut configs: Vec<String> = vec![];

    let entries = match fs::read_dir(&paths.configs) {
        Ok(entries) => entries,
        Err(e) => {
            eprintln!("Failed to read directory ({})", e);
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
    
    // We process the selected file and get a command ready for execution.
    let config_data: String;
    match fs::read_to_string(format!("{}{}", &paths.configs, selected_file)) {
        Ok(c) => config_data = c.replace("\"", ""),
        Err(_) => {
            eprintln!("Failed to read file `{}`", format!("{}{}", &paths.configs, selected_file));
            bat::pause();
            return;
        }
    }

    tcp_enable();

    match bat::stop_service(SRVCNAME) {
        Ok(_) => {},
        Err(e) => {
            if let windows_service::Error::Winapi(ref err) = e {
                let code = err.raw_os_error().unwrap_or(0);
                if code != 1062 && code != 1060 {
                    eprintln!("Couldn't stop service: {}", e);
                    eprintln!("WinAPI err code: {}", err.raw_os_error().unwrap_or(0))
                }
            }
        }
    }

    match bat::delete_service(SRVCNAME) {
        Ok(_) => {},
        Err(e) => eprintln!("Couldn't delete service: {}", e)
    }

    let mut args: Vec<String> = replace_placeholders(config_data, &paths).split(" --").map(str::to_string).collect();
    for (i, arg) in args.clone().iter().enumerate() {
        if i != 0 {
            args.insert(i, format!("--{}", arg))
        }
    }

    let exe_path = format!("{}winws.exe", paths.bin_win);
    if !std::path::Path::new(&exe_path).exists() {
        eprintln!("Executable not found: {}", exe_path);
        bat::pause();
        return;
    }

    match bat::create_service(SRVCNAME, &exe_path, args, "zapret-rust") {
        Ok(_) => {},
        Err(e) => eprintln!("Couldn't create service: {}", e)
    }

    match bat::reg_add(SRVCNAME, "ActiveConfig", selected_file.split(".").nth(0).unwrap()) {
        Ok(_) => {},
        Err(e) => eprintln!("Couldn't add active config value: {}", e)
    }

    match bat::start_service(SRVCNAME) {
        Ok(_) => {},
        Err(e) => eprintln!("Couldn't start the service: {}", e)
    }

    bat::pause();
}