## Binary Distributions

You can download BBOX tile server from [GitHub releases page](https://github.com/bbox-services/bbox/releases).

|  Platform |     Downloads (latest)    |
|-----------|---------------------------|
| Linux     | [64-bit][rl-linux-tar]    |
| macOS     | [64-bit][rl-macos-tar]    |
| macOS ARM | [ARM64][rl-macos-arm-tar] |
| Windows   | [64-bit][rl-win64-zip]    |

[rl-linux-tar]: https://github.com/bbox-services/bbox/releases/download/v0.5.0-beta2/bbox-tile-server-x86_64-unknown-linux-gnu.tar.gz
[rl-macos-tar]: https://github.com/bbox-services/bbox/releases/download/v0.5.0-beta2/bbox-tile-server-x86_64-apple-darwin.tar.gz
[rl-macos-arm-tar]: https://github.com/bbox-services/bbox/releases/download/v0.5.0-beta2/bbox-tile-server-aarch64-apple-darwin.tar.gz
[rl-win64-zip]: https://github.com/bbox-services/bbox/releases/download/v0.5.0-beta2/bbox-tile-server-x86_64-pc-windows-msvc.zip

## Debian packages

|   Distribution  |   Downloads (latest)  |
|-----------------|-----------------------|
| Ubuntu Jammy    | [x86_64][deb-jammy]    |
| Debian Bookworm | [x86_64][deb-bookworm] |
| Debian Bullseye | [x86_64][deb-bullseye] |

[deb-jammy]: https://github.com/bbox-services/bbox/releases/download/v0.5.0-beta2/bbox-tile-server_0.5.0.beta2-jammy_amd64.deb
[deb-bookworm]: https://github.com/bbox-services/bbox/releases/download/v0.5.0-beta2/bbox-tile-server_0.5.0.beta2-bookworm_amd64.deb
[deb-bullseye]: https://github.com/bbox-services/bbox/releases/download/v0.5.0-beta2/bbox-tile-server_0.5.0.beta2-bullseye_amd64.deb

## Building with Cargo

If you [install Rust](https://www.rust-lang.org/tools/install), you can build BBOX from source with Cargo:

```shell
cd bbox-tile-server
cargo install --path .
bbox-tile-server --help
```

## Docker

BBOX tile server is also available as a [Docker image](https://hub.docker.com/r/sourcepole/bbox-tile-server). You could either share a configuration file from the host with the container via the `-v` param, or you can run BBOX in auto-discover mode.

```shell
docker run -p 8080:8080 -v $PWD/bbox.toml:/var/www/bbox.toml:ro -v $PWD/assets:/assets:ro sourcepole/bbox-tile-server:v0.5.0-beta2
```
