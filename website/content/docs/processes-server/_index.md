---
weight: 10
---

# BBOX Processes Service

The OGC API - Processes standard specifies an interface for executing computational tasks.

Overview: https://ogcapi.ogc.org/processes/

Features:
- [ ] OGC API - Processes - Part 1: Core
- [x] Support synchronous and asynchronous process execution
- [x] OpenAPI endpoint
- [x] Multiple backend engines
  - [x] [Dagster](https://dagster.io/)
  - [ ] [Windmill](https://www.windmill.dev/)


## Usage

Run feature server with `bbox.toml` configuration:

    bbox-processes-server serve
