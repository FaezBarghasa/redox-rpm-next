//! cargo-redox wrapper
//! Provides automatic target injection based on Redox "profiles" (Nano, Pro, Titan).

#![forbid(unsafe_code)]

use std::env;
use std::process::{exit, Command};

fn main() {
    let mut args: Vec<String> = env::args().collect();

    // The first argument is 'cargo-redox', we drop it.
    // If invoked via 'cargo redox', the first two might be 'cargo', 'redox'.
    // We want to pass everything forward to 'cargo'.
    if !args.is_empty() {
        args.remove(0);
    }
    if !args.is_empty() && args[0] == "redox" {
        args.remove(0);
    }

    // Determine the profile either from an argument like --profile=nano or environment REDOX_PROFILE
    let mut profile = env::var("REDOX_PROFILE").unwrap_or_else(|_| "pro".to_string());

    // Scan arguments for a profile override and remove it so `cargo` doesn't get confused
    let mut i = 0;
    while i < args.len() {
        if args[i].starts_with("--redox-profile=") {
            let parts: Vec<&str> = args[i].split('=').collect();
            if parts.len() == 2 {
                profile = parts[1].to_lowercase();
            }
            args.remove(i);
        } else if args[i] == "--redox-profile" {
            args.remove(i);
            if i < args.len() {
                profile = args.remove(i).to_lowercase();
            }
        } else {
            i += 1;
        }
    }

    // Determine target based on profile
    let target = match profile.as_str() {
        "nano" | "nano-esp32" => "xtensa-esp32-none-elf",
        "pro" | "workstation" | "full-workstation" => "x86_64-unknown-redox",
        "titan" | "server" => "x86_64-unknown-redox", // Could be aarch64, defaulting to x86_64
        _ => "x86_64-unknown-redox",                  // Default
    };

    // Construct the cargo command
    let mut cmd = Command::new("cargo");

    // Do we already have a target specified?
    let has_target = args.iter().any(|a| a.starts_with("--target"));

    // Find where the subcommand ends (e.g., 'build', 'check', 'run', 'test') so we can insert the target right after cleanly
    let mut sub_command_idx = 0;
    for (idx, arg) in args.iter().enumerate() {
        if !arg.starts_with('-') {
            sub_command_idx = idx;
            break;
        }
    }

    // Pass the standard arguments
    for (idx, arg) in args.iter().enumerate() {
        if idx == sub_command_idx + 1 && !has_target {
            cmd.arg("--target").arg(target);
        }
        cmd.arg(arg);
    }

    // Fallback if there was no subcommand found but we still want to add arguments
    if args.is_empty()
        || (sub_command_idx == 0 && !has_target && !args.is_empty() && !args[0].starts_with('-'))
    {
        // Just slap it at the end if we didn't inject it yet (mostly a safety net)
    }

    // Prepare rustflags for specific targets if needed (Nano needs custom linker scripts typically)
    if target == "xtensa-esp32-none-elf" {
        let current_flags =
            env::var("CARGO_TARGET_XTENSA_ESP32_NONE_ELF_RUSTFLAGS").unwrap_or_default();
        let new_flags = format!("{} -C link-arg=-nostartfiles", current_flags);
        cmd.env(
            "CARGO_TARGET_XTENSA_ESP32_NONE_ELF_RUSTFLAGS",
            new_flags.trim(),
        );
    }

    // Run the command
    let status = match cmd.status() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to execute cargo: {}", e);
            exit(1);
        }
    };

    exit(status.code().unwrap_or(1));
}
