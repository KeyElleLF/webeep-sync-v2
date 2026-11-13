//! Application state shared by the Tauri commands.

use std::error::Error;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

use tauri::{AppHandle, Manager};

use crate::store::Store;

const DATABASE_FILE: &str = "webeep-sync.sqlite";

pub struct AppState {
    pub store: Mutex<Store>,
    syncing: AtomicBool,
}

impl AppState {
    /// Opens (or creates) the database in the app data directory.
    pub fn open(app: &AppHandle) -> Result<Self, Box<dyn Error>> {
        let dir = app.path().app_data_dir()?;
        std::fs::create_dir_all(&dir)?;
        Ok(Self {
            store: Mutex::new(Store::open(&dir.join(DATABASE_FILE))?),
            syncing: AtomicBool::new(false),
        })
    }

    /// Marks a sync as running; `None` if one already is. The flag is cleared on drop.
    pub fn begin_sync(&self) -> Option<SyncGuard<'_>> {
        let already = self.syncing.swap(true, Ordering::AcqRel);
        (!already).then_some(SyncGuard(&self.syncing))
    }
}

pub struct SyncGuard<'a>(&'a AtomicBool);

impl Drop for SyncGuard<'_> {
    fn drop(&mut self) {
        self.0.store(false, Ordering::Release);
    }
}
