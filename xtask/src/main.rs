use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() {
        print_help();
        return;
    }

    match args[0].as_str() {
        "list" | "ls" => cmd_list(),
        "new" => cmd_new(&args[1..]),
        "build" => cmd_build(&args[1..]),
        "run" | "flash" => cmd_run(&args[1..]),
        "monitor" => cmd_monitor(&args[1..]),
        "help" | "-h" | "--help" => print_help(),
        other => {
            eprintln!("Unknown command: '{}'", other);
            print_help();
            std::process::exit(1);
        }
    }
}

fn print_help() {
    println!(
        r#"
K10 ESP32-S3 Workspace Task Runner (xtask)

USAGE:
    cargo xtask <COMMAND> [APP] [OPTIONS]

COMMANDS:
    run, flash    Build, flash to K10, and open serial monitor
    build         Build the specified subproject for ESP32-S3
    new <NAME>    Create a new subproject with templates pre-configured
    monitor       Open serial monitor without flashing
    list, ls      List all subprojects in the workspace
    help          Print this help message

OPTIONS:
    --release     Build/flash with optimizations [default]
    --debug       Build/flash in debug mode
    -p, --port    Serial port to use (e.g. COM3 or /dev/ttyUSB0)
    -B, --baud    Baud rate for flashing and monitoring (default: 115200)

EXAMPLES:
    cargo xtask list
    cargo xtask run hardware_test
    cargo xtask run modbus_server --port COM3
    cargo xtask build hardware_test
    cargo xtask monitor --port COM3
"#
    );
}

/// Discovers all subprojects in `apps/`
fn list_apps() -> Vec<String> {
    let apps_dir = Path::new("apps");
    let mut apps = Vec::new();

    if let Ok(entries) = fs::read_dir(apps_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() && path.join("Cargo.toml").exists() {
                if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
                    apps.push(name.to_string());
                }
            }
        }
    }
    apps.sort();
    apps
}

fn cmd_list() {
    let apps = list_apps();
    println!("Available subprojects in 'apps/':");
    if apps.is_empty() {
        println!("  (No subprojects found in apps/)");
    } else {
        for app in apps {
            println!("  - {}", app);
        }
    }
}

fn cmd_new(args: &[String]) {
    if args.is_empty() {
        eprintln!("Error: Missing subproject name.");
        eprintln!("Usage: cargo xtask new <NAME>");
        std::process::exit(1);
    }

    let name = &args[0];

    // Basic name validation
    if !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') {
        eprintln!(
            "Error: Subproject name '{}' contains invalid characters. Use letters, numbers, hyphens, and underscores.",
            name
        );
        std::process::exit(1);
    }

    let app_dir = Path::new("apps").join(name);
    if app_dir.exists() {
        eprintln!("Error: Subproject '{}' already exists at '{}'.", name, app_dir.display());
        std::process::exit(1);
    }

    let src_dir = app_dir.join("src");
    if let Err(e) = fs::create_dir_all(&src_dir) {
        eprintln!("Failed to create directory '{}': {}", src_dir.display(), e);
        std::process::exit(1);
    }

    // 1. Cargo.toml
    let cargo_toml = format!(
r#"[package]
name = "{name}"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "{name}"
harness = false

[dependencies]
log = {{ workspace = true }}
esp-idf-svc = {{ workspace = true }}

[build-dependencies]
embuild = {{ workspace = true }}
"#
    );
    if let Err(e) = fs::write(app_dir.join("Cargo.toml"), cargo_toml) {
        eprintln!("Failed to write Cargo.toml: {}", e);
        std::process::exit(1);
    }

    // 2. build.rs
    let build_rs = r#"fn main() {
    embuild::espidf::sysenv::output();
}
"#;
    if let Err(e) = fs::write(app_dir.join("build.rs"), build_rs) {
        eprintln!("Failed to write build.rs: {}", e);
        std::process::exit(1);
    }

    // 3. src/main.rs
    let main_rs = format!(
r#"use esp_idf_svc::sys::link_patches;

fn main() {{
    link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    println!("=============================================");
    println!("        K10 - {name}");
    println!("=============================================");
}}
"#
    );
    if let Err(e) = fs::write(src_dir.join("main.rs"), main_rs) {
        eprintln!("Failed to write src/main.rs: {}", e);
        std::process::exit(1);
    }

    println!("Successfully created subproject '{}' in 'apps/{}'.", name, name);
    println!("\nNext steps:");
    println!("  cargo xtask build {}", name);
    println!("  cargo xtask run {}", name);
}

struct ParsedArgs {
    app: Option<String>,
    release: bool,
    port: Option<String>,
    baud: Option<String>,
    extra: Vec<String>,
}

fn parse_subcommand_args(args: &[String]) -> ParsedArgs {
    let mut app = None;
    let mut release = true; // default to release for embedded ESP-IDF
    let mut port = None;
    let mut baud = None;
    let mut extra = Vec::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--release" => release = true,
            "--debug" => release = false,
            "-p" | "--port" => {
                if i + 1 < args.len() {
                    port = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            "-B" | "--baud" => {
                if i + 1 < args.len() {
                    baud = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            arg if arg.starts_with("--port=") => {
                port = Some(arg.trim_start_matches("--port=").to_string());
            }
            arg if arg.starts_with("--baud=") => {
                baud = Some(arg.trim_start_matches("--baud=").to_string());
            }
            arg if arg.starts_with('-') => {
                extra.push(arg.to_string());
            }
            val => {
                if app.is_none() {
                    // Match app name or relative path
                    let clean = val.trim_start_matches("apps/").trim_start_matches("apps\\");
                    app = Some(clean.to_string());
                } else {
                    extra.push(val.to_string());
                }
            }
        }
        i += 1;
    }

    ParsedArgs {
        app,
        release,
        port,
        baud,
        extra,
    }
}

fn resolve_app(app_name: Option<String>) -> String {
    let available = list_apps();
    match app_name {
        Some(name) => {
            if available.contains(&name) {
                name
            } else {
                eprintln!("Error: Subproject '{}' not found in apps/.", name);
                eprintln!("Available subprojects: {:?}", available);
                std::process::exit(1);
            }
        }
        None => {
            if available.len() == 1 {
                let default_app = available[0].clone();
                println!("No subproject specified. Defaulting to '{}'.", default_app);
                default_app
            } else {
                eprintln!("Error: Please specify a subproject name.");
                eprintln!("Available subprojects: {:?}", available);
                eprintln!("Usage: cargo xtask run <APP>");
                std::process::exit(1);
            }
        }
    }
}

fn run_command(mut cmd: Command) {
    println!(">>> {:?}", cmd);
    let status = cmd.status().unwrap_or_else(|e| {
        eprintln!("Failed to execute command: {}", e);
        std::process::exit(1);
    });

    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }
}

fn cmd_build(args: &[String]) {
    let parsed = parse_subcommand_args(args);
    let app = resolve_app(parsed.app);

    let mut cmd = Command::new("cargo");
    cmd.arg("build").arg("--package").arg(&app);

    if parsed.release {
        cmd.arg("--release");
    }

    for extra in parsed.extra {
        cmd.arg(extra);
    }

    println!("Building subproject: {} (mode: {})...", app, if parsed.release { "release" } else { "debug" });
    run_command(cmd);
}

fn cmd_run(args: &[String]) {
    let parsed = parse_subcommand_args(args);
    let app = resolve_app(parsed.app);

    // 1. Build subproject with standard cargo build
    let mut build_cmd = Command::new("cargo");
    build_cmd.arg("build").arg("--package").arg(&app);
    if parsed.release {
        build_cmd.arg("--release");
    }
    println!(
        "Building subproject: {} (mode: {})...",
        app,
        if parsed.release { "release" } else { "debug" }
    );
    run_command(build_cmd);

    // 2. Flash and monitor directly via native `espflash` executable
    // (Bypasses `cargo-espflash` which hangs on Windows due to nested stdin/stdout pipeline deadlocks)
    let profile = if parsed.release { "release" } else { "debug" };
    let elf_path = format!("target/xtensa-esp32s3-espidf/{}/{}", profile, app);

    let mut flash_cmd = Command::new("espflash");
    flash_cmd.arg("flash").arg(&elf_path).arg("--monitor");

    if let Some(port) = parsed.port {
        flash_cmd.arg("--port").arg(port);
    }

    if let Some(baud) = parsed.baud {
        flash_cmd.arg("--monitor-baud").arg(baud);
    }

    for extra in parsed.extra {
        flash_cmd.arg(extra);
    }

    println!(
        "Flashing and running subproject: {} (mode: {})...",
        app,
        profile
    );
    run_command(flash_cmd);
}

fn cmd_monitor(args: &[String]) {
    let parsed = parse_subcommand_args(args);

    let mut cmd = Command::new("espflash");
    cmd.arg("monitor");

    if let Some(port) = parsed.port {
        cmd.arg("--port").arg(port);
    }

    if let Some(baud) = parsed.baud {
        cmd.arg("--baud").arg(baud);
    }

    for extra in parsed.extra {
        cmd.arg(extra);
    }

    println!("Opening serial monitor...");
    run_command(cmd);
}
