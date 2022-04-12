# geo302 — HTTP redirect proxy with healthcheck

`geo302` is not an actual proxy, but a "pathfinder", which responses with [`302 Found`](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/302) redirecting the HTTP-client to the actual URL.
We use [geolite2 geoIP](https://dev.maxmind.com/geoip/geolite2-free-geolocation-data) database to determine user's location and select the most suitable upstream for this location.
Client's IP is determined using proxy headers like `X-FORWARDED-FOR` with a fallback to the socket IP address.
`geo302` performs active health checks against all upstreams pinging them every few seconds.

The main use case of `geo302` is redirecting a user to the closest server to minimize download time of large files.

## Start

- Download [geolite2 geoIP database](https://dev.maxmind.com/geoip/geolite2-free-geolocation-data)
- Edit configuration file `geo302.toml`
- `cargo run --release -- ./geo302.toml`

## Configuration

See an example of the configuration in `geo302.toml`.

Here we present a configuration where the optional entries have the default values:

```toml
geolite2 = "<PATH>" # .mmdb geolite2 file, get it from http://dev.maxmind.com
host = "127.0.0.1:8080" # address to listen
ip_headers = ["x-forwarded-for"] # optional headers to get client's IP, the first available is used
ip_header_recursive = true # true: get the first ip in the header, false: get the last one
healthckeck_interval = 5 # healthcheck interval in seconds

# List of mirrors, both upstream and healthcheck keys are required
# If requested URL is <host>/<path>, then redirect URL is <UPSTREAM_URL>/<path>
[mirrors]
some_mirror = { upstream = "<UPSTREAM_URL>", healthcheck = "<HEALTHCHECK_URL>" }
another_mirror = { upstream = "<UPSTREAM2_URL>", healthcheck = "<HEALTHCHECK2_URL>" }

# List of locations:
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

**`geo302` is not a load-balancer.** `geo302` currently doesn't support an upstream rotation for a single location, but you can specify a list of upstreams: the first available location will be used.
If you need a load balancing to optimize a network usage, but do not need geoIP support, consider using another redirect proxy like [`rlb`](https://github.com/umputun/rlb).

**Locations are continent-level only.** See https://github.com/hombit/geo302/issues/3 for country-level support

**Only `GET` is supported.** See https://github.com/hombit/geo302/issues/4 for `HEAD` support for health checks

All these limitations are not a part of the design and can be fixed in the future version. Feel free to open an issue or a PR.

## License

MIT licensed.