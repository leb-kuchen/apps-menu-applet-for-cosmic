// SPDX-License-Identifier: GPL-3.0-only

use cosmic::widget::icon;
use std::{
    collections::HashMap,
    sync::{Mutex, OnceLock},
};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct IconCacheKey {
    name: String,
    size: u16,
}

pub struct IconCache {
    cache: HashMap<IconCacheKey, icon::Handle>,
}

impl IconCache {
    pub fn new() -> Self {
        let mut cache = HashMap::new();
        Self { cache }
    }

    pub fn get(&mut self, name: String, size: u16) -> icon::Handle {
        self.cache
            .entry(IconCacheKey {
                name: name.clone(),
                size,
            })
            .or_insert_with(|| {
                icon::from_name(name)
                    .size(size)
                    .fallback(Some(icon::IconFallback::Names(vec![
                        "application-default".into(),
                        "application-x-executable".into(),
                    ])))
                    .handle()
            })
            .clone()
    }
}

static ICON_CACHE: OnceLock<Mutex<IconCache>> = OnceLock::new();

pub fn icon_cache_handle(name: String, size: u16) -> icon::Handle {
    let mut icon_cache = ICON_CACHE
        .get_or_init(|| Mutex::new(IconCache::new()))
        .lock()
        .unwrap();
    icon_cache.get(name, size)
}

pub fn icon_cache_icon(name: String, size: u16) -> icon::Icon {
    icon::icon(icon_cache_handle(name, size)).size(size)
}
