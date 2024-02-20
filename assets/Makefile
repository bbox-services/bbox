# Download and prapare data

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

# For reverting mvtbench.gpkg add the following .git/hooks/pre-commit file:
# #!/bin/sh
# 
# if [ -f data/mvtbench.gpkg-wal ]; then
# 	echo Revert mvtbench.gpkg
# 	git checkout 2c96bb5 data/mvtbench.gpkg
# 	rm data/mvtbench.gpkg-*
# fi


# Setup PostgreSQL database

DBNAME = bbox_tests

createdb:
	psql postgres -c "DROP DATABASE IF EXISTS $(DBNAME)"
	psql postgres -c "CREATE DATABASE $(DBNAME)"
	psql $(DBNAME) -c "CREATE EXTENSION postgis"
	psql $(DBNAME) -c "CREATE SCHEMA ne"

loaddata:
	ogr2ogr -f PostgreSQL PG:dbname=$(DBNAME) -lco SCHEMA=ne ne_extracts.gpkg


# Misc targets

mvtbench:
	# docker run -p 127.0.0.1:5439:5432 -d --name mvtbenchdb --rm sourcepole/mvtbenchdb
	cd ../bbox-tile-server && cargo run -- --t-rex-config=../assets/mvtbench_t_rex.toml serve
	# curl -o /tmp/tile.mvt http://localhost:8080/xyz/ne_countries/0/0/0.pbf

t_rex_multi:
	# docker run -p 127.0.0.1:5439:5432 -d --name trextestdb --rm sourcepole/trextestdb
	cd ../bbox-tile-server && cargo run -- --t-rex-config=../../t-rex/examples/multiple-ds.toml serve

t_rex_grid:
	cd ../bbox-tile-server && cargo run -- --t-rex-config=../../t-rex/examples/utmgrid.toml serve
