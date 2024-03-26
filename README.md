
# Install 
```sh
git clone https://github.com/leb-kuchen/cosmic-applet-apps-menu
cd cosmic-applet-apps-menu
cargo b -r
sudo just install
```

# Config

Configuration Directory: `.config/cosmic/dev.dominiccgeh.CosmicAppletAppsMenu/`

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