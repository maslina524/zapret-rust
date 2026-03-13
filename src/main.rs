use std::env;
use std::ptr;
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::iter::once;

use winapi::shared::minwindef::DWORD;
use winapi::um::handleapi::CloseHandle;
use winapi::um::processthreadsapi::{GetCurrentProcess, OpenProcessToken};
use winapi::um::securitybaseapi::GetTokenInformation;
use winapi::um::shellapi::ShellExecuteW;
use winapi::um::winnt::{HANDLE, TOKEN_QUERY, TOKEN_ELEVATION, TokenElevation};
use winapi::um::winuser::SW_SHOWDEFAULT;

mod service;
use service::{menu, install_selected_file, service_install, service_remove, service_status};

use crate::service::add_domain;

mod bat;

fn is_running_as_admin() -> bool {
    let mut token_handle: HANDLE = ptr::null_mut();
    if unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token_handle) } == 0 {
        return false;
    }

    let mut elevation = std::mem::MaybeUninit::<TOKEN_ELEVATION>::uninit();
    let mut return_length: DWORD = 0;
    let success = unsafe {
        GetTokenInformation(
            token_handle,
            TokenElevation,
            elevation.as_mut_ptr() as *mut _,
            std::mem::size_of::<TOKEN_ELEVATION>() as DWORD,
            &mut return_length,
        ) != 0
    };

    unsafe { CloseHandle(token_handle) };
    if success {
        let elevation = unsafe { elevation.assume_init() };
        elevation.TokenIsElevated != 0
    } else {
        false
    }
}

fn run_as_admin() -> bool {
    let exe = env::current_exe().expect("failed to get exe path");
    let args: Vec<String> = env::args().skip(1).collect();

    let args_joined = args.join(" ");
    let args_wide: Vec<u16> = OsStr::new(&args_joined).encode_wide().chain(once(0)).collect();

    let exe_wide: Vec<u16> = exe.as_os_str().encode_wide().chain(once(0)).collect();

    let operation: Vec<u16> = OsStr::new("runas").encode_wide().chain(once(0)).collect();

    unsafe {
        let result = ShellExecuteW(
            ptr::null_mut(),
            operation.as_ptr(),
            exe_wide.as_ptr(),
            if !args.is_empty() { args_wide.as_ptr() } else { ptr::null() },
            ptr::null(),
            SW_SHOWDEFAULT,
        );
        result as isize > 32
    }
}

fn main() {
    if !is_running_as_admin() {
        println!("Requesting admin rights...");
        if run_as_admin() {
            return;
        } else {
            eprintln!("Couldn't get administrator rights.");
            bat::pause();
            std::process::exit(1);
        }
    }

    let argv: Vec<String> = env::args().collect();
    if argv.len() > 1 {
        match argv[1].as_str() {
            "install" => {
                if argv.len() > 2 {
                    install_selected_file(&argv[2]);
                } else {
                    service_install();
                }
            },
            "remove" => service_remove(),
            "status" => service_status(),
            "add" => {
                if argv.len() > 2 {
                    add_domain(Some(argv[2].clone()));
                } else {
                    add_domain(None);
                }
            }
            _ => menu(),
        }
    } else {
        menu();
    }
}