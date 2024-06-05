# Installs files into the system
install:
    sudo install -Dm0755 ./target/release/cosmic-applet-apps-menu  /usr/bin/apps-menu-applet-for-cosmic_tm
    sudo install -Dm0644 data/dev.dominiccgeh.CosmicAppletAppsMenu.desktop /usr/share/applications/dev.dominiccgeh.CosmicAppletAppsMenu.desktop
    find 'data'/'icons' -type f -exec echo {} \; | rev | cut -d'/' -f-3 | rev | xargs -d '\n' -I {} sudo install -Dm0644 'data'/'icons'/{} /usr/share/icons/hicolor/{}