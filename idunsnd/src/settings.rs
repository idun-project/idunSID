// Copyright (C) 2022 - 2023 Wilfred Bos
// Licensed under the GNU GPL v3 license. See the LICENSE file for the terms and conditions.

use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use app_dirs2::*;
use auto_launch::AutoLaunchBuilder;
use parking_lot::Mutex;

const APP_INFO: AppInfo = AppInfo{ name: "siddevice", author: "siddevice" };
const CONFIG_FILE_NAME: &str = "config.json";
const DEFAULT_FILTER_BIAS_6581: i32 = 24;

#[derive(Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct Config {
    pub digiboost_enabled: bool,
    pub allow_external_connections: bool,
    pub audio_device_number: Option<i32>,
    pub filter_bias_6581: Option<i32>,
    pub default_filter_bias_6581: i32,
    pub launch_at_start_enabled: bool
}

impl Config {
    pub fn new(
        digiboost_enabled: bool,
        launch_at_start_enabled: bool,
        allow_external_connections: bool,
        audio_device_number: Option<i32>,
        filter_bias_6581: Option<i32>,
        default_filter_bias_6581: i32
    ) -> Config {
        Config {
            digiboost_enabled,
            launch_at_start_enabled,
            allow_external_connections,
            audio_device_number,
            filter_bias_6581,
            default_filter_bias_6581
        }
    }
}

pub struct Settings {
    config: Arc<Mutex<Config>>,
}

impl Settings {
    pub fn new() -> Settings {
        let auto_launch = AutoLaunchBuilder::new()
            .set_app_name("sid-device")
            .set_app_path(std::env::current_exe().unwrap().to_str().unwrap())
            .set_use_launch_agent(true)
            .build()
            .unwrap();

        let config = Arc::new(Mutex::new(Self::load_config(auto_launch.is_enabled().unwrap())));

        Settings {
            config,
        }
    }

    pub fn get_config(&mut self) -> Arc<Mutex<Config>> {
        self.config.clone()
    }

    fn get_config_filename() -> PathBuf {
        let app_root = app_root(AppDataType::UserConfig, &APP_INFO).unwrap();
        let path = Path::new(app_root.as_os_str());
        path.join(CONFIG_FILE_NAME)
    }

    fn load_config(auto_launch_enabled: bool) -> Config {
        let config_filename = Self::get_config_filename();
        if Path::new(config_filename.as_path()).exists() {
            let file = File::open(&config_filename).unwrap();
            let reader = BufReader::new(file);
            let config: Option<Config> = serde_json::from_reader(reader).ok();

            if let Some(mut config) = config {
                if config.filter_bias_6581.is_none() {
                    config.filter_bias_6581 = Some(DEFAULT_FILTER_BIAS_6581);
                }
                config.default_filter_bias_6581 = DEFAULT_FILTER_BIAS_6581;

                config.launch_at_start_enabled = auto_launch_enabled;
                return config;
            }
        }
        Self::get_default_config()
    }

    fn get_default_config() -> Config {
        Config::new(
            false,
            false,
            true,
            None,
            Some(DEFAULT_FILTER_BIAS_6581),
            DEFAULT_FILTER_BIAS_6581
        )
    }
}
