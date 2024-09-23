# Asset server Configuration

Static file serving:
```toml
[[assets.static]]
# ./assets/* -> http://localhost:8080/assets/
dir = "./assets"
path = "/assets"
```

Template file serving:
```toml
[[assets.template]]
# ./templates/name.html -> http://localhost:8080/html/name/param
dir = "./templates"
path = "/html"
```

QGIS plugin repository:
```toml
[[assets.repo]]
# ./plugins/*.zip -> http://localhost:8080/qgisrepo/plugins.xml
dir = "./plugins"
path = "/qgisrepo"
```
