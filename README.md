# geo302 — HTTP redirect proxy with healthcheck

`geo302` is not an actual proxy, but a "pathfinder", which responses with [`302 Found`](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/302) redirecting the HTTP-client to the actual URL.
We use [geolite2 geoIP](https://dev.maxmind.com/geoip/geolite2-free-geolocation-data) database to determine user's location and select the most suitable upstream for this location.
Client's IP is determined using proxy headers like `X-FORWARDED-FOR` with a fallback to the socket IP address.
`geo302` performs active health checks against all upstreams pinging them every few seconds.

The main use case of `geo302` is redirecting a user to the closest server to minimize download time of large files.

## Quick start

- Edit configuration file `geo302.toml`
- `cargo run --release -- ./geo302.toml`

## Geo-IP databases

`geo302` supports two databases: proprietary [Maxmind DB](https://dev.maxmind.com)
and [ripe-geo](https://github.com/cbuijs/ripe-geo) based on RIPE, GEONAMES and IPDENY.
A fork of the ripe-geo database is available as a git submodule of this repository,
`geo302` can be built with this database embedded into the executable.
`geo302` also supports automatically updates to the most recent version of this database.

Database support can be turned on or off by compile-time features (flags).

## Compile-time features

`geo302` build can be configured to have more functionality in the cost of the executable size and larger dependency graph.
All features are additive and could activate other features.

For example the following command will compile `./target/release/geo302` with a support of Maxmind DB only: 
```bash
cargo build --release --no-default-features --features=maxminddb
```

| Feature               | in `default` | includes | Description                                                                                                     | 
|-----------------------|-------------|----------|-----------------------------------------------------------------------------------------------------------------|
| `maxminddb`           | ✓ | — | Maxmind DB support                                                                                              |
| `multi-thread`        | ✓ | — | Mutli-thread support and `threads` condiguration option                                                         |
| `ripe-geo`            | ✓ | — | ripe-geo DB support, if no `ripe-geo-*` options specified, then DB can be loaded from filesystem only           |
| `ripe-geo-autoupdate` | ✓ | `multi-thread`, `ripe-geo` | Loading and autoupdating of the ripe-geo DB from the web                                                        |
| `ripe-geo-embedded`   | | `ripe-geo` | Compiles ripe-geo DB into `geo302` executable, user needs no local or web ripe-geo distribution to be available |                                                   |
| `default`             | ✓ | `maxminddb`, `ripe-geo-autoupdate`                            | Default feature set, adds no functionality itself                                                               |
| `full`                | | `maxminddb`, `ripe-geo-autoupdate`, `ripe-geo-embedded`       | Activates all features, adds no functionality itself                                                            |

## Configuration

See examples of the configuration in `config-examples` directory.

Here we present a configuration for the default compile-time feature set, optional entries have the default values:

```toml
host = "127.0.0.1:8080" # address to listen
ip_headers = ["x-forwarded-for"] # optional headers to get client's IP, the first available is used
ip_header_recursive = true # each haeder could have multiple IPs. true: get the first ip in the header, false: get the last one
healthckeck_interval = 5 # healthcheck interval in seconds
log_level = "info" # logging level
response_headers = { <header>: "<VALUE>" } # a pairs of header key-values to add to the server reply
threads = 2 # number of threads to use, requires compile-time support. Special value "cores" means number of available CPU cores

# Geo-IP database configuration
[geoip]
type = "<TYPE>" # type of database to use, "maxminddb" and "ripe-geo" are supported

# Options for type = "maxminddb"
path = "<PATH>" # .mmdb geolite2 file, get it from https://dev.maxmind.com

# Options for "ripe-geo"
# The database can be loaded from directory (if path option specified), from embedded (compile-time
# feature=ripe-gep-embedded required) or downloaded (if autoupdate option is not false) automatically 
path = "<PATH>" # "continents" folder of ripe-geo database, get it from https://github.com/cbuijs/ripe-geo
overlaps = "skip" # ripe-geo database has overlaping IP ranges, the default is to ignore it with "skip" value
autoupdate = false # Whether to automatically download and update the database
# autoupdate = true # is equivalent to:
# [geoip.autoupdate]
# url = "https://github.com/hombit/ripe-geo-history/archive/refs/heads/continents.tar.gz" # only .tar.gz is supported
# interval = 86400 # update cadence in seconds


# List of mirrors, both upstream and healthcheck keys are required
# If requested URL is <host>/<path>, then redirect URL is <UPSTREAM_URL>/<path>
[mirrors]
some_mirror = { upstream = "<UPSTREAM_URL>", healthcheck = "<HEALTHCHECK_URL>" }
another_mirror = { upstream = "<UPSTREAM2_URL>", healthcheck = "<HEALTHCHECK2_URL>" }


# List of locations
# - some subset of continents
# - the mandatory "default" entry for the cases of unknown/unspecified client location
# For each location the first healthy mirror is used
[continents]
# Africa = 
# Asia = 
# Europe = 
# NorthAmerica = 
# Oceania = 
# SouthAmerica = 
# Antarctica = 
default = ["<some_mirror>", "<another_mirror>"]

```

## Limitations

**`geo302` is a failover and not a load-balancer.**
Currently `geo302` doesn't support an upstream rotation for a single location, but you can specify a list of upstreams: the first available location will be used.
If you need a load balancing to optimize a network usage, but do not need geoIP support, consider using another redirect proxy like [`rlb`](https://github.com/umputun/rlb).

**Locations are continent-level only.**
See https://github.com/hombit/geo302/issues/3 for country-level support

**Only `GET` is supported.**
See https://github.com/hombit/geo302/issues/4 for `HEAD` support for health checks

All these limitations are not a part of the design and can be fixed in the future version.
Feel free to open an issue or a PR.

## License

MIT licensed.
