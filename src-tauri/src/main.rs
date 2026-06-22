// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--service") {
        // Headless service mode — no GUI window, just tray + poll + notify.
        desktop_usage_helper_lib::run_with_options(desktop_usage_helper_lib::RunOptions {
            headless: true,
        });
    } else {
        desktop_usage_helper_lib::run();
    }
}