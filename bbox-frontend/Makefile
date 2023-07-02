# Download embedded JS/CSS

redoc_version = 2.0.0
swagger_version = 3.24.2
ol_version = 7.4.0
maplibre_version = 3.1.0
proj_version = 2.8.1

all: static/redoc/redoc.standalone.js \
     static/swagger/swagger-ui-bundle.js static/swagger/swagger-ui.css \
     static/maplibre/maplibre-gl.js static/maplibre/maplibre-gl.css \
     static/ol/ol.min.js static/ol/ol.min.css \
     static/proj/proj4.min.js

static/redoc/redoc.standalone.js:
	# License: https://cdn.jsdelivr.net/npm/redoc@2.0.0/bundles/redoc.standalone.js.LICENSE.txt
	wget -O $@ https://cdn.jsdelivr.net/npm/redoc@$(redoc_version)/bundles/redoc.standalone.js

static/swagger/swagger-ui-bundle.js:
	wget -O $@ https://cdnjs.cloudflare.com/ajax/libs/swagger-ui/$(swagger_version)/swagger-ui-bundle.js

static/swagger/swagger-ui.css:
	wget -O $@ https://cdnjs.cloudflare.com/ajax/libs/swagger-ui/$(swagger_version)/swagger-ui.css

static/maplibre/maplibre-gl.js:
	wget -O $@ https://unpkg.com/maplibre-gl@$(maplibre_version)/dist/maplibre-gl.js

static/maplibre/maplibre-gl.css:
	wget -O $@ https://unpkg.com/maplibre-gl@$(maplibre_version)/dist/maplibre-gl.css

static/ol/ol.min.js:
	wget -O $@ https://cdn.jsdelivr.net/npm/ol@$(ol_version)/dist/ol.min.js

static/ol/ol.min.css:
	wget -O $@ https://cdn.jsdelivr.net/npm/ol@$(ol_version)/ol.min.css

static/proj/proj4.min.js:
	wget -O $@ https://cdn.jsdelivr.net/npm/proj4@$(proj_version)/dist/proj4.min.js


QWC2_DIR = ../../../gis/qwc2-wmsapi
qwc2_files = index.html themes.json config.json \
		assets/forms/form.ui assets/css/qwc2.css assets/img/mapthumbs/default.jpg \
		assets/img/mapthumbs/mapnik.jpg assets/img/app_icon_72.png assets/img/favicon.ico \
		assets/img/app_icon.png assets/img/app_icon_144.png assets/img/app_icon_114.png \
		assets/img/logo-mobile.svg assets/img/logo.svg assets/templates/legendprint.html

qwc2:
	mkdir -p static/qwc2/translations static/qwc2/assets/forms static/qwc2/assets/css static/qwc2/assets/img/mapthumbs static/qwc2/assets/templates static/qwc2/js
	cp $(QWC2_DIR)/translations/data* static/qwc2/translations/
	cp $(QWC2_DIR)/dist/QWC2App.js static/qwc2/js/
	for f in $(qwc2_files); do cp $(QWC2_DIR)/$$f static/qwc2/$$f; done
