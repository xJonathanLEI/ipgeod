<p align="center">
  <h1 align="center">ipgeod</h1>
</p>

**Exposing IP geolocation data from [herrbischoff/country-ip-blocks](https://github.com/herrbischoff/country-ip-blocks) via HTTP**

> [!NOTE]
>
> Only IPv4 addresses are supported for now.

## Getting started

First clone the [herrbischoff/country-ip-blocks](https://github.com/herrbischoff/country-ip-blocks) repository anywhere in the filesystem. Then run from this repository:

```console
cargo run --release -- --repo-path /path/to/country-ip-blocks-repo/
```

`ipgeod` will listen on port `3000` (configurable via `--port`). Test the API with:

```console
curl http://localhost:3000/ipv4/1.2.3.4
```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](./LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](./LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.
