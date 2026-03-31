use crate::{BASE_DIRS, PROJECT_DIRS, USER_DIRS};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::ops::Deref;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs::{File, try_exists};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::Mutex;

lazy_static! {
    static ref DEFAULT_VRCHAT_PICTURES_DIR: PathBuf = find_vrchat_pictures_dir();
    static ref SETTINGS_FILE: PathBuf = settings_file();
    pub static ref SETTINGS: Arc<Mutex<SettingsData>> =
        Arc::new(Mutex::new(SettingsData::default()));
}

fn find_vrchat_pictures_dir() -> PathBuf {
    let mut dir;
    #[cfg(target_os = "windows")]
    {
        dir = USER_DIRS
            .picture_dir()
            .expect("Unable to get pictures dir")
            .join("VRChat");
    }
    #[cfg(not(target_os = "windows"))]
    {
        dir = BASE_DIRS
            .data_dir()
            .join("Steam/steamapps/compatdata/438100/pfx/drive_c/users/steamuser/Pictures/VRChat");
        if !dir.exists() {
            warn!("Unable to find proton vrchat pictures directory, trying default...");
            dir = USER_DIRS
                .picture_dir()
                .expect("Unable to get pictures dir")
                .join("VRChat");
        }
    }

    if !dir.exists() {
        warn!(
            "Unable to find vrchat pictures directory. You will likely need to set this yourself."
        );
    }

    dir
}

fn settings_file() -> PathBuf {
    PROJECT_DIRS.preference_dir().join("settings.toml")
}

#[derive(Debug, Clone, PartialOrd, PartialEq, Serialize, Deserialize)]
pub struct SettingsData {
    #[serde(default = "default_vrchat_pictures_path")]
    vrchat_pictures_path: PathBuf,
}

fn default_vrchat_pictures_path() -> PathBuf {
    DEFAULT_VRCHAT_PICTURES_DIR.clone()
}

impl Default for SettingsData {
    fn default() -> Self {
        Self {
            vrchat_pictures_path: default_vrchat_pictures_path(),
        }
    }
}

impl SettingsData {
    async fn load() -> Self {
        info!("Loading settings...");

        let settings_file = SETTINGS_FILE.deref();
        if !try_exists(settings_file).await.is_ok_and(|exists| exists) {
            info!("No existing settings file found, using defaults.");
            return Self::default();
        }

        let mut file = match File::open(SETTINGS_FILE.deref()).await {
            Ok(file) => file,
            Err(err) => {
                warn!("Error opening settings file: {err:?}");
                return Self::default();
            }
        };

        let mut config_str = String::new();
        if let Err(err) = file.read_to_string(&mut config_str).await {
            warn!("Error reading settings file: {err:?}");
            return Self::default();
        }

        let data = match toml::from_str::<Self>(&config_str) {
            Ok(data) => data,
            Err(err) => {
                warn!("Error parsing settings file: {err:?}");
                return Self::default();
            }
        };

        *SETTINGS.lock().await = data.clone();

        info!("Settings loaded.");

        data
    }

    async fn store() {
        let str = match toml::to_string_pretty(SETTINGS.lock().await.deref()) {
            Ok(str) => str,
            Err(err) => {
                warn!("Error encoding settings file: {err:?}");
                return;
            }
        };

        let mut file = match File::create(SETTINGS_FILE.deref()).await {
            Ok(file) => file,
            Err(err) => {
                warn!("Error opening settings file for writing: {err:?}");
                return;
            }
        };

        if let Err(err) = file.write_all(str.as_bytes()).await {
            warn!("Error writing settings file: {err:?}");
        }
    }
}

pub async fn init() {
    SettingsData::load().await;
}
