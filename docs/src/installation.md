## Binary Distributions

You can download BBOX from [GitHub releases page](https://github.com/bbox-services/bbox/releases).

|  Platform |     Downloads (latest)    |
|-----------|---------------------------|
| Linux     | [64-bit][rl-linux-tar]    |
| Linux ARM | [ARM64][rl-linux-arm-tar] |
| macOS     | [64-bit][rl-macos-tar]    |
| macOS ARM | [ARM64][rl-macos-arm-tar] |
| Windows   | [64-bit][rl-win64-zip]    |

[rl-linux-tar]: https://github.com/bbox-services/bbox/releases/download/v0.6.1/bbox-server-x86_64-unknown-linux-gnu.tar.gz
[rl-linux-arm-tar]: https://github.com/bbox-services/bbox/releases/download/v0.6.1/bbox-server-aarch64-unknown-linux-gnu.tar.gz
[rl-macos-tar]: https://github.com/bbox-services/bbox/releases/download/v0.6.1/bbox-server-x86_64-apple-darwin.tar.gz
[rl-macos-arm-tar]: https://github.com/bbox-services/bbox/releases/download/v0.6.1/bbox-server-aarch64-apple-darwin.tar.gz
[rl-win64-zip]: https://github.com/bbox-services/bbox/releases/download/v0.6.1/bbox-server-x86_64-pc-windows-msvc.zip

## Installing with Cargo

If you [install Rust](https://www.rust-lang.org/tools/install), you can install BBOX from crates.io:

```shell
cargo install bbox-server --locked
# or
cargo binstall bbox-server

bbox-server --help
```

## Docker

BBOX is also available as a [Docker image](https://hub.docker.com/r/sourcepole/bbox-server-qgis). You can either share a configuration file from the host with the container via the `-v` param, or run BBOX in auto-discovery mode.

```shell
docker run --rm --user=$UID -p 8080:8080 -v $PWD/bbox.toml:/bbox.toml:ro -v $PWD/assets:/assets:ro sourcepole/bbox-server-qgis:v0.6.1
```
