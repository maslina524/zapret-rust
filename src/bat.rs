use std::process::{Command};
use std::io::{self, Write};
use windows_service::service::{ServiceAccess, ServiceErrorControl, ServiceInfo, ServiceStartType, ServiceType, ServiceState};
use windows_service::service_manager::{ServiceManager, ServiceManagerAccess};
use std::ffi::{OsStr, OsString};
use winreg::enums::*;
use winreg::RegKey;

// SERVICE 
pub fn stop_service(service_name: &str) -> Result<(), windows_service::Error> {
    let manager = ServiceManager::local_computer(
        None::<&str>,
        ServiceManagerAccess::CONNECT
    )?;
    
    let service = manager.open_service(
        service_name,
        ServiceAccess::STOP
    )?;
    
    service.stop()?;
    
    Ok(())
}

pub fn delete_service(service_name: &str) -> Result<(), windows_service::Error> {
    let manager = ServiceManager::local_computer(
        None::<&str>,
        ServiceManagerAccess::CONNECT | ServiceManagerAccess::CREATE_SERVICE
    )?;

    match manager.open_service(service_name, ServiceAccess::DELETE) {
        Ok(service) => {
            match service.delete() {
                Ok(_) => {},
                Err(e) => {
                    if let windows_service::Error::Winapi(ref os_err) = e {
                        if os_err.raw_os_error() == Some(1072) {
                            std::thread::sleep(std::time::Duration::from_secs(2));
                            return Ok(());
                        }
                    }
                    return Err(e);
                }
            }
            Ok(())
        }
        Err(windows_service::Error::Winapi(os_err)) if os_err.raw_os_error() == Some(1060) => {
            Ok(())
        }
        Err(e) => Err(e),
    }
}


pub fn create_service(service_name: &str, bin_path: &str, args: Vec<String>, display_name: &str) -> Result<(), windows_service::Error> {
    let manager = ServiceManager::local_computer(None::<&str>, ServiceManagerAccess::CREATE_SERVICE)?;
    let service_bin_path = std::path::Path::new(bin_path);

    let service_info = ServiceInfo {
        name: OsStr::new(service_name).to_os_string(),
        display_name: OsStr::new(display_name).to_os_string(),
        service_type: ServiceType::OWN_PROCESS,
        start_type: ServiceStartType::AutoStart,
        error_control: ServiceErrorControl::Normal,
        executable_path: service_bin_path.to_path_buf(),
        launch_arguments: args.into_iter().map(OsString::from).collect(),
        dependencies: vec![],
        account_name: None,
        account_password: None,
    };
    manager.create_service(&service_info, ServiceAccess::CHANGE_CONFIG)?;
    Ok(())
}

pub fn start_service(service_name: &str) -> Result<(), windows_service::Error> {
    let manager = ServiceManager::local_computer(None::<&str>, ServiceManagerAccess::CONNECT)?;
    let service = manager.open_service(service_name, ServiceAccess::START)?;
    service.start(&[] as &[OsString])?;
    Ok(())
}

pub fn reg_add(service_name: &str, value_name: &str, value_data: &str) -> io::Result<()> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let path = format!(r"System\CurrentControlSet\Services\{}", service_name);
    let (key, _disposition) = hklm.create_subkey(path)?;

    key.set_value(value_name, &value_data)?;

    Ok(())
}

pub fn remove_windivert() {
    let manager = match ServiceManager::local_computer(None::<&str>, ServiceManagerAccess::CONNECT) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Couldn't open the service manager: {}", e);
            return;
        }
    };

    for &name in &["WinDivert", "WinDivert14"] {
        if let Ok(service) = manager.open_service(name, ServiceAccess::QUERY_STATUS | ServiceAccess::STOP) {
            if let Ok(status) = service.query_status() {
                if status.current_state == ServiceState::Running {
                    let _ = service.stop();
                }
            }
        }

        if let Ok(service) = manager.open_service(name, ServiceAccess::DELETE) {
            let _ = service.delete();
        }
    }
}

pub fn get_service_status(service_name: &str) -> Result<Option<ServiceState>, windows_service::Error> {
    let manager = ServiceManager::local_computer(None::<&str>, ServiceManagerAccess::CONNECT)?;
    
    match manager.open_service(service_name, ServiceAccess::QUERY_STATUS) {
        Ok(service) => {
            let status = service.query_status()?;
            Ok(Some(status.current_state))
        }
        Err(windows_service::Error::Winapi(e)) if e.raw_os_error() == Some(1060) => {
            Ok(None)
        }
        Err(e) => Err(e),
    }
}

pub fn kill_winws() {
    if let Err(e) = Command::new("taskkill").args(&["/F", "/IM", "winws.exe"]).status() {
        eprintln!("Failed to kill winws.exe: {}", e);
    }
}

// BAT COMMANDS
pub fn pause() {
    print!("Press any key to continue . . . ");
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut String::new()).unwrap();
}

pub fn cls() {
    if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(&["/c", "cls"])
            .status()
            .unwrap();
    } else {
        Command::new("clear")
            .status()
            .unwrap();
    }
}