<p align="center">
  <h1 align="center">ipgeod</h1>
</p>

**Exposing IP geolocation data from local databases via HTTP**

> [!NOTE]
>
> Only IPv4 addresses are supported for now.

## Getting started

> [!NOTE]
>
> This section demonstrates using the [herrbischoff/country-ip-blocks](https://github.com/herrbischoff/country-ip-blocks) database. See the full list of [supported databases](#supported-database-sources) for other sources.

First clone the [herrbischoff/country-ip-blocks](https://github.com/herrbischoff/country-ip-blocks) repository anywhere in the filesystem. Then run from this repository:

```console
cargo run --release -- --herrbischoff-path /path/to/country-ip-blocks-repo/
```

`ipgeod` will listen on port `3000` (configurable via `--port`). Test the API with:

```console
curl http://localhost:3000/ipv4/1.2.3.4
```

## Supported database sources

The following databases are supported:

- [herrbischoff/country-ip-blocks](https://github.com/herrbischoff/country-ip-blocks)

  To use this database, simply clone the repository anywhere in the filesystem, and set `--herrbischoff-path` (or the `HERRBISCHOFF_PATH` environment variable) to the path.

- [IP2Location LITE](https://lite.ip2location.com/)

  Download the CSV version of the `DB1.LITE` database (code `DB1LITECSV`), and set `--ip2location-db` (or the `IP2LOCATION_DB` environment variable) to the file path.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](./LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](./LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.
