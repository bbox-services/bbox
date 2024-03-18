# Running BBOX tile server

## Command line options

```shell
Usage: bbox-tile-server [OPTIONS] <COMMAND>

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
Usage: bbox-tile-server serve [FILE_OR_URL]

Arguments:
  [FILE_OR_URL]  Serve service from file or URL

Options:
  -h, --help  Print help
```

```shell
Usage: bbox-tile-server seed [OPTIONS] --tileset <TILESET> [FILE_OR_URL]

Arguments:
  [FILE_OR_URL]  Read tiles from file or URL

Options:
      --tileset <TILESET>      tile set name
      --minzoom <MINZOOM>      Minimum zoom level
      --maxzoom <MAXZOOM>      Maximum zoom level
      --extent <EXTENT>        Extent minx,miny,maxx,maxy (in grid reference system)
      --tile-path <TILE_PATH>  Base directory for file store
      --s3-path <S3_PATH>      S3 path to upload to (e.g. s3://tiles)
      --mb-path <MB_PATH>      MBTiles path to store tiles
      --pm-path <PM_PATH>      PMTiles path to store tiles
      --no-store               No tile store (for read benchmarks)
  -t, --threads <THREADS>      Number of threads to use, defaults to number of logical cores
      --tasks <TASKS>          Size of tasks queue for parallel processing
      --overwrite <OVERWRITE>  Overwrite previously cached tiles [possible values: true, false]
  -h, --help                   Print help

```
