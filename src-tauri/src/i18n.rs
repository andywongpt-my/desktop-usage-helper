// i18n — currently just passes the language preference to the frontend.
// All actual translation strings live in src/i18n/*.js on the frontend side.
// This module exists so the language preference is persisted in config
// and the Rust side can use it for tray menu labels and notification text.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Language {
    #[serde(rename = "en-US")]
    EnUS,
    #[serde(rename = "zh-CN")]
    ZhCN,
}

impl Language {
    pub fn from_str(s: &str) -> Self {
        match s {
            "zh-CN" => Language::ZhCN,
            _ => Language::EnUS,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Language::EnUS => "en-US",
            Language::ZhCN => "zh-CN",
        }
    }

    /// Get a translated string for tray menu / notifications.
    pub fn t(&self, key: &str) -> &'static str {
        match (self, key) {
            (Language::EnUS, "tray.show") => "Show dashboard",
            (Language::EnUS, "tray.refresh") => "Refresh now",
            (Language::EnUS, "tray.settings") => "Open settings",
            (Language::EnUS, "tray.quit") => "Quit",
            (Language::EnUS, "notify.below") => "below",
            (Language::EnUS, "notify.check") => "check provider",

            (Language::ZhCN, "tray.show") => "显示面板",
            (Language::ZhCN, "tray.refresh") => "立即刷新",
            (Language::ZhCN, "tray.settings") => "打开设置",
            (Language::ZhCN, "tray.quit") => "退出",
            (Language::ZhCN, "notify.below") => "低于",
            (Language::ZhCN, "notify.check") => "请检查供应商",

            _ => "",
        }
    }
}