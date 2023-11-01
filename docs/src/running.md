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
  -h, --help                 Print help
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
