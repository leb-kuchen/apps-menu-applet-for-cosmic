# Installs files into the system
install: 
    sudo install -Dm0755 ./target/release/cosmic-applet-apps-menu  /usr/bin/cosmic-applet-apps-menu
    sudo install -Dm0644 data/dev.dominiccgeh.CosmicAppletAppsMenu.desktop /usr/share/applications/dev.dominiccgeh.CosmicAppletAppsMenu.desktop
    find 'data'/'icons' -type f -exec echo {} \; | rev | cut -d'/' -f-3 | rev | xargs -d '\n' -I {} sudo install -Dm0644 'data'/'icons'/{} /usr/share/icons/hicolor/{}

# Removes files from the system
uninstall:
	sudo rm -f /usr/bin/cosmic-applet-apps-menu
	sudo rm -f /usr/share/applications/dev.dominiccgeh.CosmicAppletAppsMenu.desktop
	find /usr/share/icons/hicolor -path "*/apps/dev.dominiccgeh.CosmicAppletAppsMenu*" -type f -delete