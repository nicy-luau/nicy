/*
Copyright (C) 2026 Yanlvl99 | Nicy Luau Runtime Development

This Source Code Form is subject to the terms of the Mozilla Public
License, v. 2.0. If a copy of the MPL was not distributed with this
file, You can obtain one at http://mozilla.org/MPL/2.0/.
*/

use libloading::{Library, Symbol};
use std::env;
use std::ffi::{CStr, CString};
use std::fs;
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

fn runtime_library_basename() -> &'static str {
    if cfg!(target_os = "windows") {
        "nicyrtdyn.dll"
    } else if cfg!(target_os = "macos") {
        "libnicyrtdyn.dylib"
    } else {
        "libnicyrtdyn.so"
    }
}

fn runtime_library_prefix_and_ext() -> (&'static str, &'static str) {
    if cfg!(target_os = "windows") {
        ("nicyrtdyn", ".dll")
    } else if cfg!(target_os = "macos") {
        ("libnicyrtdyn", ".dylib")
    } else {
        ("libnicyrtdyn", ".so")
    }
}

fn collect_local_library_candidates() -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    let base = runtime_library_basename();
    let exe_dir = env::current_exe().ok().and_then(|p| p.parent().map(|d| d.to_path_buf()));

    if let Some(dir) = exe_dir {
        candidates.push(dir.join(base));

        let (prefix, ext) = runtime_library_prefix_and_ext();
        if let Ok(entries) = fs::read_dir(&dir) {
            let mut extra = entries
                .flatten()
                .map(|e| e.path())
                .filter(|p| {
                    p.file_name()
                        .and_then(|n| n.to_str())
                        .map(|name| name.starts_with(prefix) && name.ends_with(ext))
                        .unwrap_or(false)
                })
                .collect::<Vec<_>>();
            extra.sort();
            candidates.extend(extra);
        }
    }

    candidates
}

fn load_nicy_lib() -> Result<Library, String> {
    let base = runtime_library_basename();
    let mut errors = Vec::new();

    for candidate in collect_local_library_candidates() {
        let load_result = unsafe { Library::new(&candidate) };
        match load_result {
            Ok(lib) => return Ok(lib),
            Err(err) => errors.push(format!("local {}: {}", candidate.display(), err)),
        }
    }

    let path_result = unsafe { Library::new(base) };
    match path_result {
        Ok(lib) => Ok(lib),
        Err(err) => {
            errors.push(format!("PATH {}: {}", base, err));
            Err(errors.join("\n"))
        }
    }
}

fn load_symbol<'a, T>(lib: &'a Library, symbol_name: &'static [u8], pretty_name: &'static str) -> Result<Symbol<'a, T>, String> {
    unsafe { lib.get(symbol_name) }.map_err(|e| format!("failed to load symbol '{}': {}", pretty_name, e))
}

fn to_cstring_or_exit(value: &str, field_name: &str) -> CString {
    match CString::new(value) {
        Ok(v) => v,
        Err(_) => {
            eprintln!("[ERROR] Invalid {}: contains NUL byte.", field_name);
            process::exit(1);
        }
    }
}

fn exit_with_library_error(details: &str) -> ! {
    eprintln!("[FATAL] Failed to load runtime library '{}'.", runtime_library_basename());
    eprintln!("[FATAL] Attempt details:\n{}", details);
    process::exit(1);
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
            "version" | "--version" | "-v" => println!("nicy 0.1.0"),
            "runtime-version" | "--runtime-version" | "-rv" => {
                let lib = match load_nicy_lib() {
                    Ok(lib) => lib,
                    Err(details) => exit_with_library_error(&details),
                };

                let get_version: Symbol<unsafe extern "C" fn() -> *const std::os::raw::c_char> =
                    match load_symbol(&lib, b"nicy_version\0", "nicy_version") {
                        Ok(s) => s,
                        Err(err) => {
                            eprintln!("[FATAL] {}", err);
                            process::exit(1);
                        }
                    };

                let get_luau_version: Symbol<unsafe extern "C" fn() -> *const std::os::raw::c_char> =
                    match load_symbol(&lib, b"nicy_luau_version\0", "nicy_luau_version") {
                        Ok(s) => s,
                        Err(err) => {
                            eprintln!("[FATAL] {}", err);
                            process::exit(1);
                        }
                    };

                let engine_ptr = get_version();
                let luau_ptr = get_luau_version();

                if engine_ptr.is_null() || luau_ptr.is_null() {
                    eprintln!("[FATAL] runtime returned invalid version pointers");
                    process::exit(1);
                }

                let engine_ver = CStr::from_ptr(engine_ptr).to_string_lossy();
                let luau_ver = CStr::from_ptr(luau_ptr).to_string_lossy();
                println!("Engine: {}", engine_ver);
                println!("Luau: {}", luau_ver);
            }
            "run" => {
                if args.len() < 3 {
                    eprintln!("[ERROR] Missing script file. Example: nicy run script.luau");
                    process::exit(1);
                }
                execute_file(&args[2]);
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

    let lib = match load_nicy_lib() {
        Ok(lib) => lib,
        Err(details) => exit_with_library_error(&details),
    };

    let script_path = match path.to_str() {
        Some(v) => v,
        None => {
            eprintln!("[ERROR] Script path has invalid UTF-8: '{}'", script_rel_path);
            process::exit(1);
        }
    };

    let c_path = to_cstring_or_exit(script_path, "script path");

    let start: Symbol<unsafe extern "C" fn(*const std::os::raw::c_char)> =
        match load_symbol(&lib, b"nicy_start\0", "nicy_start") {
            Ok(s) => s,
            Err(err) => {
                eprintln!("[FATAL] {}", err);
                process::exit(1);
            }
        };

    unsafe { start(c_path.as_ptr()) };
}

unsafe fn execute_eval(code: &str) {
    let lib = match load_nicy_lib() {
        Ok(lib) => lib,
        Err(details) => exit_with_library_error(&details),
    };

    let c_code = to_cstring_or_exit(code, "eval code");

    let eval: Symbol<unsafe extern "C" fn(*const std::os::raw::c_char)> =
        match load_symbol(&lib, b"nicy_eval\0", "nicy_eval") {
            Ok(s) => s,
            Err(err) => {
                eprintln!("[FATAL] {}", err);
                process::exit(1);
            }
        };

    unsafe { eval(c_code.as_ptr()) };
}

unsafe fn execute_compile(script_rel_path: &str) {
    let path = Path::new(script_rel_path);
    if !path.exists() {
        eprintln!("[ERROR] File '{}' does not exist.", script_rel_path);
        process::exit(1);
    }

    let lib = match load_nicy_lib() {
        Ok(lib) => lib,
        Err(details) => exit_with_library_error(&details),
    };

    let script_path = match path.to_str() {
        Some(v) => v,
        None => {
            eprintln!("[ERROR] Script path has invalid UTF-8: '{}'", script_rel_path);
            process::exit(1);
        }
    };

    let c_path = to_cstring_or_exit(script_path, "script path");

    let compile: Symbol<unsafe extern "C" fn(*const std::os::raw::c_char)> =
        match load_symbol(&lib, b"nicy_compile\0", "nicy_compile") {
            Ok(s) => s,
            Err(err) => {
                eprintln!("[FATAL] {}", err);
                process::exit(1);
            }
        };

    unsafe { compile(c_path.as_ptr()) };
}
