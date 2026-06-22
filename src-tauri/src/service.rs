// Headless service mode — runs poll loop + notifications without a GUI window.
//
// Activated by `desktop-usage-helper.exe --service`.
// The app creates no main window, only the tray icon + background poll.
// This is useful for running as a Windows service or in kiosk scenarios
// where the user never needs to see the dashboard.

use tauri::Manager;

/// Run the app in headless service mode (no main window).
/// The tray icon and background poll still work; the user can click the
/// tray icon to show the window if needed.
pub fn run_service() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info,desktop_usage_helper_lib=debug")),
        )
        .init();

    tracing::info!("starting in service mode (headless)");

    // Reuse the normal Tauri builder but skip window creation.
    // The setup() will still create the tray, poll loop, and notifier.
    // The main window config in tauri.conf.json is ignored when we don't
    // create it explicitly — but Tauri v2 auto-creates windows from conf
    // unless we prevent it. We use a custom setup to hide it immediately.
    crate::run_with_options(crate::RunOptions {
        headless: true,
    });
}