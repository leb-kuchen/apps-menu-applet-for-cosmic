use crate::window::Window;

use config::{AppListConfig, Config, CONFIG_VERSION};
use cosmic::cosmic_config;
use cosmic::cosmic_config::CosmicConfigEntry;
mod config;
mod mouse_area_copy;
mod icon_cache;
use window::Flags;

mod localize;
mod window;

fn main() -> cosmic::iced::Result {
    localize::localize();

    let (config_handler, config) = match cosmic_config::Config::new(window::ID, CONFIG_VERSION) {
        Ok(config_handler) => {
            let config = match Config::get_entry(&config_handler) {
                Ok(ok) => ok,
                Err((errs, config)) => {
                    eprintln!("errors loading config: {:?}", errs);
                    config
                }
            };
            (Some(config_handler), config)
        }
        Err(err) => {
            eprintln!("failed to create config handler: {}", err);
            (None, Config::default())
        }
    };
    let app_list_config = match cosmic_config::Config::new("com.system76.CosmicAppList", 1) {
        Ok(config_handler) => {
            let config = match AppListConfig::get_entry(&config_handler) {
                Ok(ok) => ok,
                Err((errs, config)) => {
                    eprintln!("errors loading config: {:?}", errs);
                    config
                }
            };
            config
        }
        Err(err) => {
            eprintln!("failed to create config handler: {}", err);
            AppListConfig::default()
        }
    };

    let flags = Flags {
        config_handler,
        config,
        app_list_config,
    };
    cosmic::applet::run::<Window>(true, flags)
}
