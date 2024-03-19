use cosmic::app::Core;
use cosmic::iced::wayland::popup::{destroy_popup, get_popup};
use cosmic::iced::window::Id;
use cosmic::iced::{Command, Limits};
use cosmic::iced_core::Alignment;
use cosmic::iced_futures::Subscription;
use cosmic::iced_runtime::core::window;
use cosmic::iced_style::application;
use cosmic::widget;
use cosmic::{Element, Theme};
use freedesktop_desktop_entry::DesktopEntry;

use crate::config::{AppListConfig, Config, CONFIG_VERSION};
use cosmic::cosmic_config;

use crate::mouse_area_copy;

use cosmic_time::Timeline;

use cosmic::iced::Length;

pub const ID: &str = "dev.dominiccgeh.CosmicAppletAppsMenu";

pub struct Window {
    core: Core,
    popup: Option<Id>,

    config: Config,
    app_list_config: AppListConfig,
    #[allow(dead_code)]
    config_handler: Option<cosmic_config::Config>,
    active_category: Category,
    timeline: Timeline,
    entries: Vec<Entry>,
}

#[derive(Clone, Debug)]
pub enum Message {
    Config(Config),
    AppListConfg(AppListConfig),
    TogglePopup,
    PopupClosed(Id),
    Category(Category),
    SpawnExec(String),
    Frame(std::time::Instant),
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
        let entries = entries();
        dbg!(&flags.app_list_config);
        let window = Window {
            core,
            config: flags.config,
            config_handler: flags.config_handler,
            entries,
            active_category: Category::Favorites,
            popup: None,
            app_list_config: flags.app_list_config,
            timeline: Timeline::new(),
        };
        (window, Command::none())
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
                    self.config = config
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
                        .max_width(372.0)
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
                self.active_category = category;
            }
            Message::SpawnExec(exec) => {
                cosmic::desktop::spawn_desktop_exec(exec, Vec::<(&str, &str)>::new());
            }
            Message::AppListConfg(config) => {
                self.app_list_config = config;
            }
        }
        Command::none()
    }

    fn view(&self) -> Element<Self::Message> {
        cosmic::widget::button(widget::text("Applications").size(14.0))
            .style(cosmic::theme::Button::AppletIcon)
            .on_press(Message::TogglePopup)
            .into()
    }

    fn view_window(&self, _id: Id) -> Element<Self::Message> {
        let mut content_list = widget::column::with_capacity(1).padding([8, 8]);
        let mut rows = widget::row::with_capacity(2);

        let mut left_side = widget::column::with_capacity(ALL_CATEGORIES.len());
        for category in ALL_CATEGORIES {
            let txt = widget::text(format!("{category:?}")).width(Length::Fill);

            let btn = widget::button(txt)
                .on_press(Message::Category(*category))
                .selected(self.active_category == *category)
                .style(cosmic::theme::Button::HeaderBar);
            let area =
                mouse_area_copy::MouseArea::new(btn).on_mouse_hover(Message::Category(*category));
            let container = widget::container(area).width(Length::Fill);
            left_side = left_side.push(container);
        }

        let mut active_entries: Vec<_> = self
            .entries
            .iter()
            .filter(|entry| match &self.active_category {
                Category::Favorites => self.app_list_config.favorites.contains(&entry.appid),
                category => entry.categories.contains(category),
            })
            .collect();
        active_entries.sort_by(|a, b| a.name.cmp(&b.name));
        let mut right_side = widget::column::with_capacity(active_entries.len()).width(400);

        for entry in active_entries {
            let txt = widget::text(entry.name.clone()).width(Length::Fill);

            let icon = widget::icon::from_name(entry.icon.clone());
            let row = widget::row::with_capacity(2)
                .push(icon)
                .push(txt)
                .align_items(Alignment::Center);
            let btn = widget::button(row)
                .on_press(Message::SpawnExec(entry.exec.clone()))
                .style(cosmic::theme::Button::HeaderBar);
            let container = widget::container(btn).width(Length::Fill);
            right_side = right_side.push(container);
        }
        let right_scroll = widget::scrollable(right_side).height(500);
        use unicode_segmentation::UnicodeSegmentation;
        let max_width = ALL_CATEGORIES
            .iter()
            .map(|category| format!("{:?}", category).graphemes(true).count())
            .max()
            .unwrap_or(0) as f32
            * 11.0;

        let left_container = widget::container(left_side).width(Length::Fixed(max_width));
        let right_container = widget::container(right_scroll).width(Length::Fill);
        rows = rows.push(left_container).push(right_container);
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

        let timeline = self
            .timeline
            .as_subscription()
            .map(|(_, now)| Message::Frame(now));

        Subscription::batch(vec![config, app_list_config, timeline])
    }

    fn style(&self) -> Option<<Theme as application::StyleSheet>::Style> {
        Some(cosmic::applet::style())
    }
}
use std::fs;
use std::path::Path;
impl Window {}

#[derive(Debug)]
struct Entry {
    name: String,
    exec: String,
    categories: Vec<Category>,
    icon: String,
    appid: String,
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Category {
    Favorites,
    AudioVideo,
    Audio,
    Video,
    Development,
    Education,
    Game,
    Graphics,
    Science,
    Network,
    Office,
    System,
    Utility,
    Other,
}

const ALL_CATEGORIES: &'static [Category] = &[
    Category::Favorites,
    Category::AudioVideo,
    Category::Audio,
    Category::Video,
    Category::Development,
    Category::Education,
    Category::Game,
    Category::Graphics,
    Category::Science,
    Category::Network,
    Category::Office,
    Category::System,
    Category::Utility,
    Category::Other,
];

const _ALL_CATEGORIES_STR: &'static [&'static str] = &[
    "Favorites",
    "AudioVideo",
    "Audio",
    "Video",
    "Development",
    "Education",
    "Game",
    "Graphics",
    "Science",
    "Network",
    "Office",
    "System",
    "Utility",
];
impl Category {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "AudioVideo" => Some(Category::AudioVideo),
            "Audio" => Some(Category::Audio),
            "Video" => Some(Category::Video),
            "Development" => Some(Category::Development),
            "Education" => Some(Category::Education),
            "Game" => Some(Category::Game),
            "Graphics" => Some(Category::Graphics),
            "Science" => Some(Category::Science),
            "Network" => Some(Category::Network),
            "Office" => Some(Category::Office),
            "System" => Some(Category::System),
            "Utility" => Some(Category::Utility),
            _ => Some(Category::Other),
        }
    }
}

fn entries() -> Vec<Entry> {
    use freedesktop_desktop_entry::{default_paths, Iter};
    Iter::new(default_paths())
        .filter_map(|p| parse_entry(&p))
        .collect()
}
fn parse_entry(path: &Path) -> Option<Entry> {
    let bytes = fs::read_to_string(path).ok()?;
    let desktop_entry = DesktopEntry::decode(path, &bytes).ok()?;
    (!desktop_entry.no_display()).then_some(())?;
    let name = desktop_entry.name(None)?.to_string();
    let exec = desktop_entry.exec()?.to_string();
    let icon = desktop_entry.icon()?.to_string();
    let appid = desktop_entry.appid.to_string();
    let categories: Vec<_> = desktop_entry
        .categories()?
        .split(";")
        .filter_map(Category::from_str)
        .collect();
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
