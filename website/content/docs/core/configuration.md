# Core Configuration

Configuration is read from `bbox.toml` and environment variables.

## Webserver

```toml
[webserver]
# Web server settings
# Environment variable prefix: BBOX_WEBSERVER__
server_addr = "0.0.0.0:8080"  # Default: 127.0.0.1:8080
# worker_threads = 4  # Default: number of CPU cores
loglevel = "Info" # Error, Warn, Info, Debug, Trace
```
