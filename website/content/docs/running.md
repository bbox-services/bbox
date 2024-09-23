---
weight: 4
---

# Running BBOX

## Command line options

```shell
Usage: bbox-server [OPTIONS] <COMMAND>

Commands:
  serve   Run service
  seed    Seed tiles
  upload  Upload tiles
  help    Print this message or the help of the given subcommand(s)

Options:
  -c, --config <FILE>        Config file (Default: bbox.toml)
      --loglevel <LOGLEVEL>  Log level (Default: info) [possible values: error, warn, info, debug, trace]
  -t, --t-rex-config <FILE>  T-Rex config file
  -h, --help                 Print help
```

```shell
Usage: bbox-server serve [FILE_OR_URL]

Arguments:
  [FILE_OR_URL]  Serve service from file or URL

Options:
  -h, --help  Print help
```

## Access Web Backend

    x-www-browser http://127.0.0.1:8080/


## Service components

Service components are included in `bbox-server`, but can also be run as standalone service:

- `bbox-feature-server`: [Feature server](feature-server/)
- `bbox-map-server`: [Map server](map-server/)
- `bbox-tile-server`: [Tile Server](tile-server/)
- `bbox-asset-server`: [Asset server](asset-server/)
- `bbox-processes-server`: [Processes server](processes-server/)
- `bbox-routing-server`: [Routing server](routing-server/)


## Docker

    docker run -p 8080:8080 sourcepole/bbox-server-qgis

Serve tiles from file:

    docker run -p 8080:8080 -v $PWD/assets:/assets:ro sourcepole/bbox-server-qgis bbox-server serve /assets/liechtenstein.mbtiles

Run with configuration file:

    docker run -p 8080:8080 -v $PWD/bbox.toml:/var/www/bbox.toml:ro -v $PWD/assets:/var/www/assets:ro sourcepole/bbox-server-qgis
