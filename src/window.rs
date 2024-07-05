#![allow(clippy::needless_return)]

use crate::config::{AppListConfig, Config, CONFIG_VERSION};
use cosmic::app::Core;
use cosmic::cosmic_config;
use cosmic::cosmic_theme::Spacing;
use cosmic::desktop::IconSource;
use cosmic::iced::wayland::popup::{destroy_popup, get_popup};
use cosmic::iced::window::Id;
use cosmic::iced::{self, Command, Limits};
use cosmic::iced_core::Alignment;
use cosmic::iced_futures::futures::SinkExt;
use cosmic::iced_futures::Subscription;
use cosmic::iced_runtime::core::window;
use cosmic::iced_style::application;
use cosmic::iced_widget::scrollable;
use cosmic::{widget, Apply};
use cosmic::{Element, Theme};
use freedesktop_desktop_entry::DesktopEntry;
use lexical_sort::natural_lexical_cmp;
use notify::Watcher;
use tokio::task::spawn_blocking;

use crate::mouse_area_copy;

use cosmic_time::Timeline;

use cosmic::iced::Length;

pub const ID: &str = "dev.dominiccgeh.CosmicAppletAppsMenu";

// todo case insensitive categories

pub struct Window {
    core: Core,
    popup: Option<Id>,
    config: Config,
    app_list_config: AppListConfig,
    #[allow(dead_code)]
    config_handler: Option<cosmic_config::Config>,
    active_category: String,
    timeline: Timeline,
    entry_map: HashMap<String, Vec<Entry>>,
    scrollable_id: widget::Id,
}

#[derive(Clone, Debug)]
pub enum Message {
    Config(Config),
    AppListConfg(AppListConfig),
    TogglePopup,
    PopupClosed(Id),
    Category(String),
    SpawnExec(String),
    Frame(std::time::Instant),
    NotifyEvent(notify::Event),
    CategoryUpdate(Option<HashMap<String, Vec<Entry>>>),
}

#[derive(Clone, Debug)]
pub struct Flags {
    pub config_handler: Option<cosmic_config::Config>,
    pub config: Config,
    pub app_list_config: AppListConfig,
}

impl cosmic::Application for Window {
    type Executor = cosmic::SingleThreadExecutor;
    type Flags = Flags;
    type Message = Message;
    const APP_ID: &'static str = ID;

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(
        core: Core,
        flags: Self::Flags,
    ) -> (Self, Command<cosmic::app::Message<Self::Message>>) {
        let mut config = flags.config;
        if config.sort_categories {
            config.categories.sort_by(|a, b| category_cmp(a, b));
        }
        let favorites = flags.app_list_config.favorites.clone();
        let entry_map = HashMap::new();
        let window = Window {
            core,
            config: config.clone(),
            config_handler: flags.config_handler,
            active_category: config.categories.first().cloned().unwrap_or(String::new()),
            popup: None,
            app_list_config: flags.app_list_config,
            entry_map,
            timeline: Timeline::new(),
            scrollable_id: widget::Id::unique(),
        };
        (window, update_entry_map(favorites, config))
    }

    fn on_close_requested(&self, id: window::Id) -> Option<Message> {
        Some(Message::PopupClosed(id))
    }

    fn update(&mut self, message: Self::Message) -> Command<cosmic::app::Message<Self::Message>> {
        // Helper for updating config values efficiently
        #[allow(unused_macros)]
        macro_rules! config_set {
            ($name: ident, $value: expr) => {
                match &self.config_handler {
                    Some(config_handler) => {
                        match paste::paste! { self.config.[<set_ $name>](config_handler, $value) } {
                            Ok(_) => {}
                            Err(err) => {
                                eprintln!("failed to save config {:?}: {}", stringify!($name), err);
                            }
                        }
                    }
                    None => {
                        self.config.$name = $value;
                        eprintln!(
                            "failed to save config {:?}: no config handler",
                            stringify!($name),
                        );
                    }
                }
            };
        }

        match message {
            Message::Config(config) => {
                if config != self.config {
                    self.config = config.clone();
                    if self.config.sort_categories {
                        self.config.categories.sort_by(|a, b| category_cmp(a, b));
                    }
                    let favorites = self.app_list_config.favorites.clone();
                    return update_entry_map(favorites, config);
                }
            }

            Message::Frame(now) => self.timeline.now(now),

            Message::TogglePopup => {
                return if let Some(p) = self.popup.take() {
                    destroy_popup(p)
                } else {
                    let new_id = Id::unique();
                    self.popup.replace(new_id);
                    let mut popup_settings =
                        self.core
                            .applet
                            .get_popup_settings(Id::MAIN, new_id, None, None, None);
                    popup_settings.positioner.size_limits = Limits::NONE
                        .max_width(500.0)
                        .min_width(300.0)
                        .min_height(200.0)
                        .max_height(1080.0);
                    get_popup(popup_settings)
                }
            }
            Message::PopupClosed(id) => {
                if self.popup.as_ref() == Some(&id) {
                    self.popup = None;
                }
            }
            Message::Category(category) => {
                if category == self.active_category {
                    return Command::none();
                }
                self.active_category = category;
                return scrollable::scroll_to(
                    self.scrollable_id.clone(),
                    scrollable::AbsoluteOffset::default(),
                );
            }
            Message::SpawnExec(exec) => {
                cosmic::desktop::spawn_desktop_exec(exec, Vec::<(&str, &str)>::new());
                if let Some(p) = self.popup.take() {
                    return destroy_popup(p);
                };
            }
            Message::AppListConfg(config) => {
                if config != self.app_list_config {
                    let favorites = config.favorites.clone();
                    self.app_list_config = config;
                    let config = self.config.clone();
                    return update_entry_map(favorites, config);
                }
            }
            Message::NotifyEvent(_event) => {
                let favorites = self.app_list_config.favorites.clone();
                let config = self.config.clone();
                return update_entry_map(favorites, config);
            }
            Message::CategoryUpdate(entry_map) => {
                if let Some(entry_map) = entry_map {
                    self.entry_map = entry_map;
                }
            }
        }
        Command::none()
    }

    fn view(&self) -> Element<Self::Message> {
        self.core
            .applet
            .icon_button(ID)
            .on_press(Message::TogglePopup)
            .into()
    }

    fn view_window(&self, _id: Id) -> Element<Self::Message> {
        #[allow(unused_variables)]
        let Spacing {
            space_xxxs,
            space_xxs,
            space_xs,
            space_s,
            space_l,
            ..
        } = self.core.system_theme().cosmic().spacing;

        let mut content_list = widget::column::with_capacity(1).padding([8, 0]);
        let mut rows = widget::row::with_capacity(2);
        let Config { categories, .. } = &self.config;
        let mut left_side = widget::grid().row_spacing(0);

        let empty_vec = Vec::new();
        let active_entries = self
            .entry_map
            .get(&self.active_category)
            .unwrap_or(&empty_vec);

        // HACK: determine the largest item and do not set the width to Fill
        // alternative might be a mouse area which return the bounds of the widget
        // uniform widget ( so it would net to implement into width for self, and then you get the layout bounds?

        let mut max_width = 0;
        let mut max_category = None;

        for category in categories {
            if self.config.skip_empty_categories && !self.entry_map.contains_key(category) {
                continue;
            }
            let count = unicode_display_width::width(&category);
            if count > max_width {
                max_width = count;
                max_category = Some(category);
            }
        }
        for category in categories {
            if self.config.skip_empty_categories && !self.entry_map.contains_key(category) {
                continue;
            }
            let txt = widget::text(category)
                .apply(widget::container)
                .padding([0, space_xxxs]);

            let mut btn = widget::button(txt)
                .on_press(Message::Category(category.clone()))
                .selected(self.active_category == *category)
                .style(cosmic::theme::Button::HeaderBar);

            if max_category.map_or(true, |max| max != category) {
                btn = btn.width(Length::Fill);
            }
            let area =
                mouse_area_copy::MouseArea::new(btn).on_enter(Message::Category(category.clone()));
            left_side = left_side.push(area).insert_row();
        }
        let mut right_side = widget::column::with_capacity(active_entries.len());

        for entry in active_entries {
            let txt = widget::text(entry.name.clone()).width(Length::Fill);

            let icon = entry.icon.as_cosmic_icon().size(20);
            let row = widget::row::with_capacity(2)
                .push(icon)
                .push(txt)
                .spacing(space_xxs)
                .align_items(Alignment::Center);
            let btn = widget::button(row)
                .on_press(Message::SpawnExec(entry.exec.clone()))
                .style(cosmic::theme::Button::HeaderBar);
            let container = widget::container(btn).width(Length::Fill);
            right_side = right_side.push(container);
        }
        let right_scroll = widget::scrollable(right_side)
            .height(500)
            .id(self.scrollable_id.clone());

        let left_container = widget::container(left_side).width(Length::Shrink);
        let right_container = widget::container(right_scroll).width(Length::Fill);
        rows = rows
            .push(left_container)
            .push(right_container)
            .spacing(space_xs);
        content_list = content_list.push(rows);

        self.core.applet.popup_container(content_list).into()
    }
    fn subscription(&self) -> Subscription<Self::Message> {
        struct ConfigSubscription;
        struct AppListConfigSubscription;
        let config = cosmic_config::config_subscription(
            std::any::TypeId::of::<ConfigSubscription>(),
            Self::APP_ID.into(),
            CONFIG_VERSION,
        )
        .map(|update| {
            if !update.errors.is_empty() {
                eprintln!(
                    "errors loading config {:?}: {:?}",
                    update.keys, update.errors
                );
            }
            Message::Config(update.config)
        });
        let app_list_config = cosmic_config::config_subscription(
            std::any::TypeId::of::<AppListConfigSubscription>(),
            "com.system76.CosmicAppList".into(),
            1,
        )
        .map(|update| {
            if !update.errors.is_empty() {
                eprintln!(
                    "errors loading config {:?}: {:?}",
                    update.keys, update.errors
                );
            }
            Message::AppListConfg(update.config)
        });
        struct WatcherSubscription;
        let id = std::any::TypeId::of::<WatcherSubscription>();
        let watcher = iced::subscription::channel(id, 100, |mut output| async move {
            let mut watcher_res = notify::recommended_watcher(
                move |event_res: Result<notify::Event, notify::Error>| match event_res {
                    Ok(event) => {
                        match &event.kind {
                            notify::EventKind::Access(_) => return,
                            _ => {}
                        }
                        let event_send = iced::futures::executor::block_on(async {
                            output.send(Message::NotifyEvent(event)).await
                        });
                        match event_send {
                            Ok(()) => {}
                            Err(e) => {
                                eprintln!("error sending notify event for desktop files {e:?} ")
                            }
                        }
                    }
                    Err(e) => eprintln!("failed to watch destkop files {e:?}"),
                },
            );
            match &mut watcher_res {
                Ok(watcher) => {
                    for path in freedesktop_desktop_entry::default_paths() {
                        _ = watcher.watch(&path, notify::RecursiveMode::NonRecursive);
                    }
                }
                Err(_) => {}
            }
            loop {
                tokio::time::sleep(tokio::time::Duration::new(1, 0)).await;
            }
        });

        let timeline = self
            .timeline
            .as_subscription()
            .map(|(_, now)| Message::Frame(now));

        Subscription::batch(vec![config, app_list_config, watcher, timeline])
    }

    fn style(&self) -> Option<<Theme as application::StyleSheet>::Style> {
        Some(cosmic::applet::style())
    }
}

fn update_entry_map(
    favorites: Vec<String>,
    config: Config,
) -> Command<cosmic::app::Message<Message>> {
    return Command::perform(
        async move {
            spawn_blocking(move || entry_map(entries(&config), favorites, &config))
                .await
                .ok()
        },
        |entry_map| cosmic::app::message::app(Message::CategoryUpdate(entry_map)),
    );
}
use std::borrow::Cow;
use std::collections::HashMap;
use std::path::Path;
use std::{cmp, fs};
impl Window {}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Entry {
    name: String,
    exec: String,
    categories: Vec<String>,
    icon: IconSource,
    appid: String,
}

fn entry_map(
    mut entries: Vec<Entry>,
    favorites: Vec<String>,
    config: &Config,
) -> HashMap<String, Vec<Entry>> {
    entries.sort_by(|a, b| natural_lexical_cmp(&a.name, &b.name));
    let mut entry_map = HashMap::with_capacity(entries.len());
    for entry in &entries {
        let mut categories: Vec<_> = entry.categories.iter().collect();
        categories.sort_by(|a, b| category_cmp(a, b));
        categories.dedup();
        for category in categories {
            entry_map
                .entry(category.clone())
                .or_insert(Vec::new())
                .push(entry.clone());
        }
    }
    // todo only works if entry is present
    for entry in favorites {
        if let Some(entry) = entries.iter().find(|it| it.appid == entry) {
            entry_map
                .entry("Favorites".into())
                .or_insert(Vec::new())
                .push(entry.clone())
        }
    }
    entry_map
        .entry("Favorites".into())
        .or_insert(Vec::new())
        .sort_by(|a, b| natural_lexical_cmp(&a.name, &b.name));

    let other = entry_map.get("Other");
    if let Some(other) = other {
        let other: Vec<_> = other
            .iter()
            .filter(|entry| {
                !entry_map
                    .iter()
                    .filter(|(k, _)| *k != "Other")
                    .any(|(_, v)| {
                        v.binary_search_by(|a| natural_lexical_cmp(&a.name, &entry.name))
                            .is_ok()
                    })
            })
            .cloned()
            .collect();
        entry_map.insert("Other".to_string(), other);
    }
    if config.skip_empty_categories {
        entry_map.retain(|_, v| !v.is_empty());
    }
    entry_map.shrink_to_fit();
    entry_map.values_mut().for_each(|e| e.shrink_to_fit());
    entry_map
}

fn entries(config: &Config) -> Vec<Entry> {
    use freedesktop_desktop_entry::{default_paths, get_languages_from_env, Iter};
    let locales = get_languages_from_env();

    Iter::new(default_paths())
        .filter_map(|p| parse_entry(&p, config, &locales))
        .collect()
}

fn category_cmp(a: &str, b: &str) -> cmp::Ordering {
    // favorites top - other bottom
    return match (a, b) {
        ("Favorites", "Favorites") | ("Other", "Other") => cmp::Ordering::Equal,
        ("Favorites", _) => cmp::Ordering::Less,
        (_, "Favorites") => cmp::Ordering::Greater,
        ("Other", _) => cmp::Ordering::Greater,
        (_, "Other") => cmp::Ordering::Less,
        _ => natural_lexical_cmp(a, b),
    };
}

fn parse_entry(path: &Path, config: &Config, locales: &[String]) -> Option<Entry> {
    let bytes = fs::read_to_string(path).ok()?;
    let desktop_entry = DesktopEntry::from_str(path, &bytes, locales).ok()?;

    (!desktop_entry.no_display()).then_some(())?;

    let name = desktop_entry
        .name(locales)
        .unwrap_or_else(|| Cow::from(&*desktop_entry.appid))
        .to_string();

    let exec = desktop_entry.exec()?.to_string();

    let icon = desktop_entry.icon().unwrap_or(&desktop_entry.appid);
    let icon = IconSource::from_unknown(icon);
    let appid = desktop_entry.appid.to_string();
    let mut categories = Vec::new();
    for mut category in desktop_entry.categories()?.split_terminator(";") {
        let category_lowercase = category.to_lowercase();
        category = if let Some(config_category) = config
            .categories
            .iter()
            .find(|c| c.to_lowercase() == category_lowercase.as_str())
        {
            &config_category
        } else {
            "Other"
        };
        categories.push(category.to_string());
    }
    (!categories.is_empty()).then_some(())?;

    let entry = Entry {
        appid,
        name,
        categories,
        exec,
        icon,
    };
    Some(entry)
}
