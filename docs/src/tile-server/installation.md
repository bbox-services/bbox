## Binary Distributions

You can download BBOX tile server from [GitHub releases page](https://github.com/bbox-services/bbox/releases).

|  Platform |     Downloads (latest)    |
|-----------|---------------------------|
| Linux     | [64-bit][rl-linux-tar]    |
| macOS     | [64-bit][rl-macos-tar]    |
| macOS ARM | [ARM64][rl-macos-arm-tar] |
| Windows   | [64-bit][rl-win64-zip]    |

[rl-linux-tar]: https://github.com/bbox-services/bbox/releases/download/v0.5.0-beta1/bbox-tile-server-x86_64-unknown-linux-gnu.tar.gz
[rl-macos-tar]: https://github.com/bbox-services/bbox/releases/download/v0.5.0-beta1/bbox-tile-server-x86_64-apple-darwin.tar.gz
[rl-macos-arm-tar]: https://github.com/bbox-services/bbox/releases/download/v0.5.0-beta1/bbox-tile-server-aarch64-apple-darwin.tar.gz
[rl-win64-zip]: https://github.com/bbox-services/bbox/releases/download/v0.5.0-beta1/bbox-tile-server-x86_64-pc-windows-msvc.zip

# Building with Cargo

If you [install Rust](https://www.rust-lang.org/tools/install), you can build BBOX from source with Cargo:

```shell
cd bbox-tile-server
cargo install --path .
bbox-tile-server --help
```
