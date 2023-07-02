QWC2_DIR = ../../../gis/qwc2-wmsapi

files = index.html themes.json config.json assets/forms/form.ui assets/css/qwc2.css assets/img/mapthumbs/default.jpg assets/img/mapthumbs/mapnik.jpg assets/img/app_icon_72.png assets/img/favicon.ico assets/img/app_icon.png assets/img/app_icon_144.png assets/img/app_icon_114.png assets/img/logo-mobile.svg assets/img/logo.svg assets/templates/legendprint.html

static_map:
	mkdir -p static/qwc2/translations static/qwc2/assets/forms static/qwc2/assets/css static/qwc2/assets/img/mapthumbs static/qwc2/assets/templates static/qwc2/js
	cp $(QWC2_DIR)/translations/data* static/qwc2/translations/
	cp $(QWC2_DIR)/dist/QWC2App.js static/qwc2/js/
	for f in $(files); do cp $(QWC2_DIR)/$$f static/qwc2/$$f; done

# For reverting mvtbench.gpkg add the following .git/hooks/pre-commit file:
# #!/bin/sh
# 
# if [ -f data/mvtbench.gpkg-wal ]; then
# 	echo Revert mvtbench.gpkg
# 	git checkout 2c96bb5 data/mvtbench.gpkg
# 	rm data/mvtbench.gpkg-*
# fi
