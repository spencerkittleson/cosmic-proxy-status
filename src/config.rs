// SPDX-License-Identifier: MPL-2.0

use cosmic::cosmic_config::{self, CosmicConfigEntry, cosmic_config_derive::CosmicConfigEntry};

#[derive(Debug, Clone, CosmicConfigEntry, Eq, PartialEq)]
#[version = 1]
pub struct AppConfig {
    pub proxy_url: String,
    pub check_url: String,
    pub interval_secs: u64,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            proxy_url: "http://192.168.8.204:3129".into(),
            check_url: "http://example.com".into(),
            interval_secs: 10,
        }
    }
}

#[allow(dead_code)]
pub fn load_config(app_id: &str) -> AppConfig {
    cosmic_config::Config::new(app_id, AppConfig::VERSION)
        .map(|context| match AppConfig::get_entry(&context) {
            Ok(config) => config,
            Err((_errors, config)) => {
                for error in _errors {
                    log::error!("config error: {error:?}");
                }
                config
            }
        })
        .unwrap_or_default()
}

pub fn save_config(app_id: &str, config: &AppConfig) -> Result<(), std::io::Error> {
    let context = cosmic_config::Config::new(app_id, AppConfig::VERSION)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    config
        .write_entry(&context)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    Ok(())
}
