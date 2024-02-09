## Binary Distributions

You can download BBOX from [GitHub releases page](https://github.com/bbox-services/bbox/releases).

|  Platform |     Downloads (latest)    |
|-----------|---------------------------|
| Linux     | [64-bit][rl-linux-tar]    |
| macOS     | [64-bit][rl-macos-tar]    |
| macOS ARM | [ARM64][rl-macos-arm-tar] |
| Windows   | [64-bit][rl-win64-zip]    |

[rl-linux-tar]: https://github.com/bbox-services/bbox/releases/download/v0.5.0-alpha4/bbox-server-Linux-x86_64.tar.gz
[rl-macos-tar]: https://github.com/bbox-services/bbox/releases/download/v0.5.0-alpha4/bbox-server-Darwin-x86_64.tar.gz
[rl-macos-arm-tar]: https://github.com/bbox-services/bbox/releases/download/v0.5.0-alpha4/bbox-server-Darwin-arch64.tar.gz
[rl-win64-zip]: https://github.com/bbox-services/bbox/releases/download/v0.5.0-alpha4/bbox-server-Windows-x86_64.zip

# Building with Cargo

If you [install Rust](https://www.rust-lang.org/tools/install), you can build BBOX from source with Cargo:

```shell
cd bbox-server
cargo install --path .
bbox-server --help
```

## Docker

BBOX is also available as a [Docker image](https://hub.docker.com/r/sourcepole/bbox-server-qgis). You could either share a configuration file from the host with the container via the `-v` param, or you can run BBOX in auto-discover mode.

```shell
docker run -p 8080:8080 -v $PWD/bbox.toml:/var/www/bbox.toml:ro -v $PWD/assets:/assets:ro sourcepole/bbox-server-qgis
```
