use std::vec;

use cosmic::cosmic_config::{self, cosmic_config_derive::CosmicConfigEntry, CosmicConfigEntry};

use serde::{Deserialize, Serialize};
pub const CONFIG_VERSION: u64 = 1;

#[derive(Clone, CosmicConfigEntry, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Config {
    pub skip_empty_categories: bool,
    pub categories: Vec<String>,
    pub sort_categories: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            skip_empty_categories: true,
            categories: vec![
                "Favorites".into(),
                "Audio".into(),
                "AudioVideo".into(),
                "COSMIC".into(),
                "Education".into(),
                "Game".into(),
                "Graphics".into(),
                "Network".into(),
                "Office".into(),
                "Science".into(),
                "Settings".into(),
                "System".into(),
                "Utility".into(),
                "Other".into(),
            ],
            sort_categories: true,
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq, Eq, CosmicConfigEntry)]
pub struct AppListConfig {
    pub favorites: Vec<String>,
}
