# Map Server Configuration

## Map server settings

```toml
[mapserver]
# num_fcgi_processes = 4     # Default: number of CPU cores
# wait_timeout = 30000       # FCGI wait timeout in ms. Default: 90s
# search_projects = false    # Scan directories and build inventory
```

## QGIS Server settings

```toml
[mapserver.qgis_backend]
project_basedir = "../assets"  # Base dir for project files (.qgs, .qgz)
qgs.path = "/qgis"             # WMS URL base path
qgz.path = "/qgz"              # WMS URL base path
```

## UMN MapServer settings

```toml
[mapserver.umn_backend]
project_basedir = "../assets"  # Base dir for project files (.map)
path = "/wms/map"              # WMS URL base path
```
