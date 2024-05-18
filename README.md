
# Install 
```sh
git clone https://github.com/leb-kuchen/cosmic-applet-apps-menu
cd cosmic-applet-apps-menu
cargo b -r
sudo just install
```

# Config

The configuration directory is `.config/cosmic/dev.dominiccgeh.CosmicAppletAppsMenu/`.

Each configuration option coresponds to a filename, e.g. you can set `skip_empty_categories` with `true > .config/cosmic/dev.dominiccgeh.CosmicAppletAppsMenu/skip_empty_categories`.

These are the default options:

```
skip_empty_categories: true,
categories: [
    "Favorites",
    "Audio",
    "AudioVideo",
    "COSMIC",
    "Education",
    "Game",
    "Graphics",
    "Network",
    "Office",
    "Science",
    "Settings",
    "System",
    "Utility",
    "Other",
],
sort_categories: true,
```

Note that, categories are case-sensitive and `Favorites` and `Other` are not
acutally categories in your desktop files.

# Dependencies
(some may not be required)
```
Build-Depends:
  debhelper (>= 11),
  debhelper-compat (= 11),
  rustc ,
  cargo,
  libdbus-1-dev,
  libegl-dev,
  libpulse-dev,
  libudev-dev,
  libxkbcommon-dev,
  libwayland-dev,
  libinput-dev,
  just,
  pkg-config,
```
