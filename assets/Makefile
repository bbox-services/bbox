all: ne_extracts.gpkg

download/natural_earth_vector.gpkg.zip:
	mkdir -p download
	wget -O $@ https://naciscdn.org/naturalearth/packages/natural_earth_vector.gpkg.zip

download/packages/natural_earth_vector.gpkg: download/natural_earth_vector.gpkg.zip
	unzip -d download $<
	touch $@

ne_extracts.gpkg: download/packages/natural_earth_vector.gpkg
	ogr2ogr -f GPKG -select scalerank,featurecla,name -nlt PROMOTE_TO_MULTI $@ $< ne_10m_rivers_lake_centerlines
	ogr2ogr -update -select scalerank,featurecla,name -nlt PROMOTE_TO_MULTI $@ $< ne_10m_lakes
	ogr2ogr -update -select scalerank,labelrank,featurecla,name $@ $< ne_10m_populated_places
