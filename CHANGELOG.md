# Changelog

All notable changes to `geo302` will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

--

### Changed

- Minimum supported Rust version 1.60 -> 1.64

### Deprecated

--

### Removed

--

### Fixed

--

### Security

- `openssl` 0.10.48 -> 0.10.55 in `Cargo.lock` for [RUSTSEC-2023-0044](https://rustsec.org/advisories/RUSTSEC-2023-0044.html)

## [0.2.1] 2023-01-23

### Security

- `tokio` 0.23.0 -> 0.24.2 in `Cargo.lock` for [RUSTSEC-2023-0001](https://rustsec.org/advisories/RUSTSEC-2023-0001)

## [0.2.0] 2022-12-23

### Added

- [`ripe-geo`](https://github.com/cbuijs/ripe-geo) Geo-IP database support with auto-updates, embedding and more
- [`ripe-geo`] Git submodule for database embedding and tests
- Cargo features for detailed compile-time configuration
- Optional multi-threading support, it is primary needed by `ripe-geo` autoupdating feature
- `enum_dispatch` v0.3 dependency
- `flate2` v1 optional dependency
- `http-serde` v1.1 dependency
- `include_dir` v0.7 optional dependency
- `lazy-static` v1 optional dependency
- `tar` v0.4 optional dependency

### Changed

- **breaking** Configuration schema: mandatory `geoip` entry is added, `geolite2` entry is replaced with `geoip.path`
- **breaking** Configuration schema: `healthcheck_interval` integer is replaced with `healthcheck = {interval = <SECONDS>, timeout = <SECONDS>}` dictionary
- `maxminddb` dependency is optional now

## [0.1.4] 2022-12-02

### Added

- Minimum supported Rust version (MSRV) is introduced, we support build with Rust toolchain 1.59+

### Changed

- `simple_logger` 2.3 -> 4.0

### Fixed

- Healthcheck could freeze waiting HTTP response infinitely

## [0.1.3] 2022-10-20

### Changed

- Reimplementation with pure `hyper` with no `wasp` and `reqwest` dependencies.
- `geo302` is now a single-thread: I see no reason to run it in multiple threads.
- Reduce allocations
- Binary size reduced a lot by removing unused dependencies and features (from 5.8MB to 1.2MB on aarch64 Linux). Also release build is stripped now.

## [0.1.2] 2022-09-05

### Changed

- `clap` 3.1.10 -> 3.1.14
- `thiserror` 1.0.30 -> 1.0.33
- `maxminddb` 0.22.0 -> 0.23.0
- `anyhow` 1.0.56 -> 1.0.63
- `tokio` 1.17.0 -> 1.20.1
- `reqwest` 0.11.10 -> 0.11.11
- `clap` 3.1.18 -> 3.2.2
- `smallvec` 1.8.0 -> 1.9.0
- `serde` 1.0.137 -> 1.0.144
- `simple_logger` 2.1.0 -> 2.3.0

## [0.1.1] 2022-04-21

### Added

- `response_headers` config option

### Fixed

- Query is attached to the location

## [0.1.0] 2022-04-20

Initial version
