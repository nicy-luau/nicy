/*
Copyright (C) 2026 Yanlvl99 | Nicy Luau Runtime Development

This Source Code Form is subject to the terms of the Mozilla Public
License, v. 2.0. If a copy of the MPL was not distributed with this
file, You can obtain one at http://mozilla.org/MPL/2.0/.
*/

use libloading::{Library, Symbol};
use std::env;
use std::ffi::{CStr, CString};
use std::path::{Path, PathBuf};
use std::process;

fn print_help() {
    println!("nicy - The Ultimate Luau Runtime");
    println!("Usage:");
    println!("  nicy run <script.luau>");
    println!("  nicy eval <\"code\">");
    println!("  nicy compile <script.luau>");
    println!("  nicy help");
    println!("  nicy version");
    println!("  nicy runtime-version");
}

unsafe fn load_nicy_lib() -> Library {
    let dll_name = if cfg!(target_os = "windows") {
        "nicyrtdyn.dll"
    } else if cfg!(target_os = "macos") {
        "libnicyrtdyn.dylib"
    } else {
        "libnicyrtdyn.so"
    };

    let local_dll = env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join(dll_name)))
        .unwrap_or_else(|| PathBuf::from(dll_name));

    let lib_result = unsafe { Library::new(&local_dll).or_else(|_| Library::new(dll_name)) };

    match lib_result {
        Ok(l) => l,
        Err(e) => {
            eprintln!("[FATAL] Coudn't load library '{}'. Check if it's in the same directory as the executable or in the PATH. Error: {}", dll_name, e);
            process::exit(1);
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_help();
        return;
    }

    let command = args[1].as_str();

    unsafe {
        match command {
            "help" | "--help" | "-h" => print_help(),
            
            "run" => {
                if args.len() < 3 {
                    eprintln!("[ERROR] Missing script file. Example: nicy run script.luau");
                    process::exit(1);
                }
                execute_file(&args[2]);
            }

            "version" | "--version" | "-v" => {
                println!("nicy 0.1.0");
            }

            "runtime-version" | "--runtime-version" | "-rv" => {
                let lib = load_nicy_lib();
                
                let get_version: Symbol<unsafe extern "C" fn() -> *const std::os::raw::c_char> = 
                    lib.get(b"nicy_version").expect("[FATAL] Failed to load 'nicy_version'");
                let get_luau_version: Symbol<unsafe extern "C" fn() -> *const std::os::raw::c_char> = 
                    lib.get(b"nicy_luau_version").expect("[FATAL] Failed to load 'nicy_luau_version'");

                let engine_ver = CStr::from_ptr(get_version()).to_string_lossy();
                let luau_ver = CStr::from_ptr(get_luau_version()).to_string_lossy();

                println!("Engine: {}", engine_ver);
                println!("Luau: {}", luau_ver);
            }
            
            "eval" => {
                if args.len() < 3 {
                    eprintln!("[ERROR] Missing code to evaluate. Example: nicy eval \"print('hello')\"");
                    process::exit(1);
                }
                execute_eval(&args[2]);
            }
            
            "compile" => {
                if args.len() < 3 {
                    eprintln!("[ERROR] Missing script file to compile. Example: nicy compile script.luau");
                    process::exit(1);
                }
                execute_compile(&args[2]);
            }
            
            _ => {
                let path = Path::new(command);
                if path.exists() {
                    execute_file(command);
                } else {
                    eprintln!("[ERROR] Unknown command or file not found: '{}'", command);
                    process::exit(1);
                }
            }
        }
    }
}

unsafe fn execute_file(script_rel_path: &str) {
    let path = Path::new(script_rel_path);
    if !path.exists() {
        eprintln!("[ERROR] File '{}' does not exist.", script_rel_path);
        process::exit(1);
    }

    let lib = unsafe { load_nicy_lib() };
    let c_path = CString::new(path.to_str().unwrap()).unwrap();
    
    let start: Symbol<unsafe extern "C" fn(*const std::os::raw::c_char)> = 
        unsafe { lib.get(b"nicy_start").expect("[FATAL] Failed to load 'nicy_start' symbol from library.") };

    unsafe { start(c_path.as_ptr()) };
}

unsafe fn execute_eval(code: &str) {
    let lib = unsafe { load_nicy_lib() };
    let c_code = CString::new(code).unwrap();
    
    let eval: Symbol<unsafe extern "C" fn(*const std::os::raw::c_char)> = 
        unsafe { lib.get(b"nicy_eval").expect("[FATAL] Failed to load 'nicy_eval' symbol from library.") };

    unsafe { eval(c_code.as_ptr()) };
}

unsafe fn execute_compile(script_rel_path: &str) {
    let path = Path::new(script_rel_path);
    if !path.exists() {
        eprintln!("[ERROR] File '{}' does not exist.", script_rel_path);
        process::exit(1);
    }

    let lib = unsafe { load_nicy_lib() };
    let c_path = CString::new(path.to_str().unwrap()).unwrap();
    
    let compile: Symbol<unsafe extern "C" fn(*const std::os::raw::c_char)> = 
        unsafe { lib.get(b"nicy_compile").expect("[FATAL] Failed to load 'nicy_compile' symbol from library.") };

    unsafe { compile(c_path.as_ptr()) };
}