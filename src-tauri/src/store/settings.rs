//! User settings stored as key/value pairs, validated on the way in and out.

use std::str::FromStr;

use serde::{Deserialize, Serialize};

use super::{Store, StoreError};

const CONCURRENCY_KEY: &str = "download_concurrency";
const FLATTEN_KEY: &str = "flatten_materials";
const CLEANUP_KEY: &str = "cleanup_policy";
const AUTOSYNC_KEY: &str = "autosync";
const AUTOSYNC_MINUTES_KEY: &str = "autosync_minutes";
const NOTIFICATIONS_KEY: &str = "notifications";
/// §7: default 4, at most 8 to be gentle with WeBeep.
pub const DEFAULT_CONCURRENCY: u8 = 4;
pub const MAX_CONCURRENCY: u8 = 8;
/// §7: automatic sync every 2 hours by default, never more often than every 30 minutes.
pub const DEFAULT_AUTOSYNC_MINUTES: u32 = 120;
pub const MIN_AUTOSYNC_MINUTES: u32 = 30;
pub const MAX_AUTOSYNC_MINUTES: u32 = 24 * 60;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    /// Files downloaded at the same time (1..=8).
    pub download_concurrency: u8,
    /// No folder for the "Materiali" section (like the v1).
    pub flatten_materials: bool,
    /// What to do with the files of a course marked as passed (§6.5).
    pub cleanup_policy: CleanupPolicy,
    /// Sync in the background while the app runs.
    pub autosync: bool,
    pub autosync_minutes: u32,
    /// System notifications (new files, announcements, deadlines).
    pub notifications: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CleanupPolicy {
    /// Ask at startup (Yes / No / Later).
    #[default]
    Ask,
    /// Move untouched files to the bin right away (modified ones are always kept).
    Always,
    Never,
}

impl CleanupPolicy {
    fn name(self) -> &'static str {
        match self {
            Self::Ask => "ask",
            Self::Always => "always",
            Self::Never => "never",
        }
    }

    fn parse(name: &str) -> Option<Self> {
        [Self::Ask, Self::Always, Self::Never]
            .into_iter()
            .find(|p| p.name() == name)
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            download_concurrency: DEFAULT_CONCURRENCY,
            flatten_materials: true,
            cleanup_policy: CleanupPolicy::Ask,
            autosync: true,
            autosync_minutes: DEFAULT_AUTOSYNC_MINUTES,
            notifications: true,
        }
    }
}

impl Settings {
    /// Brings values into their valid range.
    pub fn clamped(self) -> Self {
        Self {
            download_concurrency: self.download_concurrency.clamp(1, MAX_CONCURRENCY),
            autosync_minutes: self
                .autosync_minutes
                .clamp(MIN_AUTOSYNC_MINUTES, MAX_AUTOSYNC_MINUTES),
            ..self
        }
    }
}

impl Store {
    /// Saved settings; missing or unreadable values fall back to the defaults.
    pub fn settings(&self) -> Result<Settings, StoreError> {
        let d = Settings::default();
        Ok(Settings {
            download_concurrency: self.parsed(CONCURRENCY_KEY, d.download_concurrency)?,
            flatten_materials: self.parsed(FLATTEN_KEY, d.flatten_materials)?,
            cleanup_policy: self
                .setting(CLEANUP_KEY)?
                .and_then(|v| CleanupPolicy::parse(&v))
                .unwrap_or_default(),
            autosync: self.parsed(AUTOSYNC_KEY, d.autosync)?,
            autosync_minutes: self.parsed(AUTOSYNC_MINUTES_KEY, d.autosync_minutes)?,
            notifications: self.parsed(NOTIFICATIONS_KEY, d.notifications)?,
        }
        .clamped())
    }

    pub fn save_settings(&self, settings: Settings) -> Result<Settings, StoreError> {
        let s = settings.clamped();
        self.set_setting(CONCURRENCY_KEY, &s.download_concurrency.to_string())?;
        self.set_setting(FLATTEN_KEY, &s.flatten_materials.to_string())?;
        self.set_setting(CLEANUP_KEY, s.cleanup_policy.name())?;
        self.set_setting(AUTOSYNC_KEY, &s.autosync.to_string())?;
        self.set_setting(AUTOSYNC_MINUTES_KEY, &s.autosync_minutes.to_string())?;
        self.set_setting(NOTIFICATIONS_KEY, &s.notifications.to_string())?;
        Ok(s)
    }

    fn parsed<T: FromStr>(&self, key: &str, default: T) -> Result<T, StoreError> {
        Ok(self
            .setting(key)?
            .and_then(|v| v.parse().ok())
            .unwrap_or(default))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_when_nothing_is_saved() {
        let store = Store::open_in_memory().unwrap();
        assert_eq!(store.settings().unwrap(), Settings::default());
    }

    #[test]
    fn saves_and_clamps() {
        let store = Store::open_in_memory().unwrap();
        let saved = store
            .save_settings(Settings {
                download_concurrency: 50,
                flatten_materials: false,
                cleanup_policy: CleanupPolicy::Never,
                autosync: false,
                autosync_minutes: 5,
                notifications: false,
            })
            .unwrap();
        assert_eq!(saved.download_concurrency, MAX_CONCURRENCY);
        assert_eq!(saved.autosync_minutes, MIN_AUTOSYNC_MINUTES);
        assert_eq!(store.settings().unwrap(), saved);

        let saved = store
            .save_settings(Settings {
                download_concurrency: 0,
                autosync_minutes: 100_000,
                ..saved
            })
            .unwrap();
        assert_eq!(saved.download_concurrency, 1);
        assert_eq!(saved.autosync_minutes, MAX_AUTOSYNC_MINUTES);
    }

    #[test]
    fn garbage_values_fall_back_to_defaults() {
        let store = Store::open_in_memory().unwrap();
        store.set_setting(CONCURRENCY_KEY, "tanti").unwrap();
        store.set_setting(FLATTEN_KEY, "forse").unwrap();
        assert_eq!(store.settings().unwrap(), Settings::default());
    }
}
