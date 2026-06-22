// Tray icon + context menu + close-to-tray handling.
//
// Owns:
//   - The Tauri TrayIcon (icon + menu + click dispatch)
//   - "Minimize-to-tray" behavior on window close
//   - Dynamic tray icon (status dot recolored per aggregate state)
//   - Dynamic tooltip (compact per-provider remaining-pct summary)

use crate::models::{AppConfig, ProviderState, ProviderStatus};
use crate::provider::worst;
use std::collections::HashMap;
use tauri::image::Image;
use tauri::menu::{Menu, MenuBuilder, MenuEvent, MenuItemBuilder};
use tauri::tray::{MouseButton, TrayIcon, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, Runtime, WindowEvent};

/// Logical state of the whole app — drives the tray icon dot color.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AggregateState {
    Ok,
    Warn,
    Danger,
    Unknown,
}

impl From<ProviderState> for AggregateState {
    fn from(s: ProviderState) -> Self {
        match s {
            ProviderState::Danger => AggregateState::Danger,
            ProviderState::Warn => AggregateState::Warn,
            ProviderState::Ok => AggregateState::Ok,
            ProviderState::Unknown => AggregateState::Unknown,
        }
    }
}

/// RGBA color for the status dot.
fn dot_color(state: AggregateState) -> [u8; 4] {
    match state {
        AggregateState::Ok => [52, 211, 153, 255],      // emerald-400
        AggregateState::Warn => [250, 204, 21, 255],    // amber-400
        AggregateState::Danger => [239, 68, 68, 255],   // red-500
        AggregateState::Unknown => [148, 163, 184, 255], // slate-400
    }
}

/// Menu item ids — used in on_menu_event dispatch.
pub mod menu_ids {
    pub const SHOW: &str = "tray_show";
    pub const REFRESH: &str = "tray_refresh";
    pub const SETTINGS: &str = "tray_settings";
    pub const QUIT: &str = "tray_quit";
}

/// Build the tray icon with menu and event handlers.
pub fn install<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<TrayIcon<R>>
where
    AppHandle<R>: Manager<R>,
{
    let handle = app.clone();

    let show_item = MenuItemBuilder::with_id(menu_ids::SHOW, "Show dashboard").build(&handle)?;
    let refresh_item =
        MenuItemBuilder::with_id(menu_ids::REFRESH, "Refresh now").build(&handle)?;
    let settings_item =
        MenuItemBuilder::with_id(menu_ids::SETTINGS, "Open settings").build(&handle)?;
    let quit_item = MenuItemBuilder::with_id(menu_ids::QUIT, "Quit").build(&handle)?;

    let menu: Menu<R> = MenuBuilder::new(&handle)
        .item(&show_item)
        .separator()
        .item(&refresh_item)
        .item(&settings_item)
        .separator()
        .item(&quit_item)
        .build()?;

    // Initial icon: neutral gray dot (Unknown) until first refresh lands.
    let icon_bytes: &[u8] = include_bytes!("../icons/32x32.png");
    let icon = build_icon(icon_bytes, AggregateState::Unknown)
        .unwrap_or_else(|_| Image::new_owned(vec![0, 0, 0, 0], 1, 1));

    let tray = TrayIconBuilder::with_id("main-tray")
        .icon(icon)
        .tooltip("Desktop Usage Helper — initialising…")
        .menu(&menu)
        .show_menu_on_left_click(false) // Windows: left-click toggles window
        .on_menu_event(move |app, event: MenuEvent| {
            handle_menu_event(app, event);
        })
        .on_tray_icon_event(move |_tray, event: TrayIconEvent| {
            handle_tray_click(&handle, event);
        })
        .build(app)?;

    Ok(tray)
}

/// Dispatch a context-menu click.
fn handle_menu_event<R: Runtime>(app: &AppHandle<R>, event: MenuEvent)
where
    AppHandle<R>: Manager<R>,
{
    match event.id().as_ref() {
        menu_ids::SHOW => show_main_window(app),
        menu_ids::REFRESH => {
            use tauri::Emitter;
            let _ = app.emit("tray:refresh_requested", ());
            show_main_window(app);
        }
        menu_ids::SETTINGS => {
            use tauri::Emitter;
            let _ = app.emit("tray:open_settings", ());
            show_main_window(app);
        }
        menu_ids::QUIT => {
            app.exit(0);
        }
        _ => {}
    }
}

/// Left-click on the tray icon toggles the main window (right-click shows menu natively).
fn handle_tray_click<R: Runtime>(handle: &AppHandle<R>, event: TrayIconEvent)
where
    AppHandle<R>: Manager<R>,
{
    if let TrayIconEvent::Click { button, .. } = event {
        if button == MouseButton::Left {
            toggle_main_window(handle);
        }
    }
}

/// Show + focus + unminimize the main window.
pub fn show_main_window<R: Runtime>(app: &AppHandle<R>)
where
    AppHandle<R>: Manager<R>,
{
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

/// Hide main window if visible, else show it.
pub fn toggle_main_window<R: Runtime>(app: &AppHandle<R>)
where
    AppHandle<R>: Manager<R>,
{
    if let Some(window) = app.get_webview_window("main") {
        let visible = window.is_visible().unwrap_or(false);
        if visible {
            let _ = window.hide();
        } else {
            let _ = window.show();
            let _ = window.unminimize();
            let _ = window.set_focus();
        }
    }
}

/// Wire the close-to-tray behavior. Attach in `lib.rs` after the window is created.
pub fn setup_close_to_tray<R: Runtime>(
    window: tauri::WebviewWindow<R>,
    cfg_store: crate::config::ConfigStore,
) where
    tauri::WebviewWindow<R>: Manager<R>,
{
    window.clone().on_window_event(move |event| {
        if let WindowEvent::CloseRequested { api, .. } = event {
            // Default to "hide" if we can't read config — safer than quit.
            let cfg = cfg_store.snapshot_blocking_or_default();
            if cfg.minimize_to_tray {
                let _ = window.hide();
                api.prevent_close();
            }
        }
    });
}

/// Update tray icon + tooltip to reflect the worst current state across all providers.
pub fn update_from_statuses<R: Runtime>(
    app: &AppHandle<R>,
    statuses: &HashMap<String, ProviderStatus>,
) where
    AppHandle<R>: Manager<R>,
{
    let agg = aggregate_state(statuses);
    let tooltip = build_tooltip(statuses, agg);
    let icon_bytes: &'static [u8] = include_bytes!("../icons/32x32.png");

    let app_clone = app.clone();
    let _ = app.run_on_main_thread(move || {
        if let Some(tray) = app_clone.tray_by_id("main-tray") {
            let _ = tray.set_tooltip(Some(tooltip));
            if let Ok(img) = build_icon(icon_bytes, agg) {
                let _ = tray.set_icon(Some(img));
            }
        }
    });
}

/// Worst state across all providers, or Unknown if empty.
fn aggregate_state(statuses: &HashMap<String, ProviderStatus>) -> AggregateState {
    let states: Vec<ProviderState> = statuses.values().map(|s| s.state).collect();
    if states.is_empty() {
        return AggregateState::Unknown;
    }
    worst(&states).into()
}

/// Compose a compact tooltip — top 3 most-critical providers + count.
fn build_tooltip(statuses: &HashMap<String, ProviderStatus>, agg: AggregateState) -> String {
    if statuses.is_empty() {
        return match agg {
            AggregateState::Unknown => "Desktop Usage Helper — no data".into(),
            _ => "Desktop Usage Helper".into(),
        };
    }

    let mut rows: Vec<(String, f64, ProviderState)> = statuses
        .iter()
        .filter_map(|(id, s)| {
            s.primary.as_ref().and_then(|p| {
                if p.limit > 0.0 {
                    let remaining = (((p.limit - p.used) / p.limit) * 100.0).max(0.0);
                    Some((id.clone(), remaining, s.state))
                } else {
                    None
                }
            })
        })
        .collect();
    rows.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

    let head: String = rows
        .iter()
        .take(3)
        .map(|(id, pct, state)| {
            let flag = match state {
                ProviderState::Danger => " ⚠",
                ProviderState::Warn => " !",
                _ => "",
            };
            format!("{} {:.0}%{}", id, pct, flag)
        })
        .collect::<Vec<_>>()
        .join(" · ");

    format!(
        "{}\n— click to open · {} providers",
        head,
        statuses.len()
    )
}

// ---- icon rendering ------------------------------------------------------

fn build_icon(png_bytes: &[u8], state: AggregateState) -> tauri::Result<Image<'_>> {
    let base = Image::from_bytes(png_bytes)?;
    let width = base.width();
    let height = base.height();
    let mut rgba: Vec<u8> = base.rgba().to_vec();

    if width == 0 || height == 0 {
        return Ok(Image::new_owned(rgba, width, height));
    }

    let dot = dot_color(state);
    let ring: [u8; 4] = [15, 23, 42, 255]; // slate-900
    let radius: i32 = (width.min(height) as i32 / 5).max(5).min(10);
    let cx: i32 = width as i32 - radius - 2;
    let cy: i32 = radius + 2;

    for y in 0..height as i32 {
        for x in 0..width as i32 {
            let dx = x - cx;
            let dy = y - cy;
            let d2 = dx * dx + dy * dy;
            let ring_r = radius + 1;
            if d2 <= ring_r * ring_r {
                let idx = ((y as u32 * width + x as u32) * 4) as usize;
                let (color, alpha) = if d2 <= radius * radius {
                    (dot, 1.0f32)
                } else {
                    let dist = (d2 as f32).sqrt();
                    let edge = ring_r as f32 - dist;
                    let alpha = edge.clamp(0.0, 1.0);
                    (ring, alpha)
                };
                let dst = &mut rgba[idx..idx + 4];
                let a = alpha;
                dst[0] = ((color[0] as f32) * a + (dst[0] as f32) * (1.0 - a)) as u8;
                dst[1] = ((color[1] as f32) * a + (dst[1] as f32) * (1.0 - a)) as u8;
                dst[2] = ((color[2] as f32) * a + (dst[2] as f32) * (1.0 - a)) as u8;
                dst[3] = (color[3] as f32).max(dst[3] as f32) as u8;
            }
        }
    }

    Ok(Image::new_owned(rgba, width, height))
}

// ---- ConfigStore helper for synchronous read in close handler ------------

pub trait ConfigStoreSyncExt {
    fn snapshot_blocking_or_default(&self) -> AppConfig;
}

impl ConfigStoreSyncExt for crate::config::ConfigStore {
    fn snapshot_blocking_or_default(&self) -> AppConfig {
        self.try_snapshot().unwrap_or_else(|_| AppConfig {
            minimize_to_tray: true, // safe default — don't accidentally quit
            ..AppConfig::default()
        })
    }
}