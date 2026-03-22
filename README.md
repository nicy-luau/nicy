# nicy

A blazing-fast command-line interface (CLI) and executable host that provides a Luau runtime environment by dynamically loading the `nicyrtdyn` core engine.

## Overview

`nicy` is designed to be a flexible and high-performance terminal wrapper for Luau scripts. It's built in Rust and dynamically links to the `nicyrtdyn` shared library (`.dll`, `.so`, or `.dylib`) at runtime. It gives you instant access to a sandboxed environment for Luau scripts with a custom `require` implementation that supports caching, file fingerprinting, and aliasing.

**⚠️ IMPORTANT:** This repository only contains the CLI host. To execute scripts, you **must** download the core engine library from the official repository:
👉 **[https://github.com/nicy-luau/nicyrtdyn](https://github.com/nicy-luau/nicyrtdyn)**

Place the downloaded shared library in the same directory as the `nicy` executable or in your system's `PATH`.

## Features

- **CLI Host:** A lightweight executable that safely delegates the heavy lifting to the `nicyrtdyn` engine.
- **Dynamic Loading:** Automatically locates and loads the engine library from the local directory or system `PATH`.
- **Command Routing:** Built-in commands for running (`run`), evaluating (`eval`), and compiling (`compile`) Luau scripts.
- **Native Code Integration:** Enables loading of native shared libraries directly from Luau using `runtime.loadlib`.
- **Custom Module Resolver:** A sophisticated `require()` implementation with:
  - Module caching based on file fingerprints.
  - Automatic cache invalidation.
  - Support for `.luaurc` alias files.
  - Circular dependency detection.
- **Asynchronous Task Scheduler:** A cooperative multitasking scheduler for Luau coroutines, with support for `task.spawn`, `task.defer`, `task.delay`, and `task.wait`.

## Luau API

When running scripts through the `nicy` CLI, the following APIs are exposed to your Luau environment:

### `runtime` object

A global `runtime` object is available for host interaction:

- `runtime.version`: The version of the underlying `nicyrtdyn` library.
- `runtime.native_enabled`: A boolean indicating if native library loading is enabled (via `!native`).
- `runtime.entry_file`: The path to the main script being executed.
- `runtime.entry_dir`: The directory of the main script.
- `runtime.loadlib(path: string)`: Loads a dynamic library. The path can be relative and use the `@self` alias.

### `task` library

A global `task` library is available for cooperative multitasking:

- `task.spawn(f, ...)`: Spawns a new coroutine.
- `task.defer(f, ...)`: Similar to `task.spawn`.
- `task.delay(seconds, f, ...)`: Spawns a coroutine after a delay.
- `task.wait(seconds)`: Pauses the current coroutine for a given number of seconds.
- `task.cancel(thread|delay_id)`: Cancels a running task.

## Architecture

The CLI is structured to be as minimal as possible:

- `main.rs`: The main entry point, CLI argument parsing, command routing (`run`, `eval`, `compile`, `help`), and dynamic linking via `libloading` to the core engine.

## Build

- Stable manifest: `Cargo.toml` (`panic = "abort"`).
- Nightly manifest: `nightly/Cargo.toml` (`panic = "immediate-abort"`).

```powershell
./build.ps1 -target win-x64 -toolchain stable -force
./build.ps1 -target win-x64 -toolchain nightly -force
```

## License

This project is licensed under the Mozilla Public License 2.0. See the [LICENSE](LICENSE) file for details.
