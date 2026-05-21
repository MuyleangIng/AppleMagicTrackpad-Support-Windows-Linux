use std::env;
use std::process::ExitCode;

use mt2_settings::{preset_settings, save_settings, Settings};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("error: {message}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);
    let Some(command) = args.next() else {
        print_help();
        return Ok(());
    };

    match command.as_str() {
        "help" | "--help" | "-h" => {
            print_help();
            Ok(())
        }
        "preset" => {
            require_windows()?;
            let preset = args.next().ok_or("missing preset name")?;
            save_settings(preset_settings(&preset)?)
        }
        "defaults" => {
            require_windows()?;
            save_settings(Settings::default())
        }
        _ => Err(format!("unknown command: {command}")),
    }
}

fn require_windows() -> Result<(), String> {
    if cfg!(windows) {
        Ok(())
    } else {
        Err("mt2-control can change settings only on Windows".to_string())
    }
}

fn print_help() {
    println!("Magic Trackpad 2 Rust Control");
    println!();
    println!("Usage:");
    println!("  mt2-control defaults");
    println!("  mt2-control preset macos-light");
    println!("  mt2-control preset macos-medium");
    println!("  mt2-control preset macos-firm");
    println!("  mt2-control preset silent");
    println!("  mt2-control preset disabled");
    println!("  mt2-control preset maximum");
    println!();
    println!("Run as Administrator so Windows allows registry writes.");
}
