# EVMap OSM Loader

A Rust implementation of the Overpass API data loader for charging stations.
This tool downloads worldwide charging station data from OpenStreetMap via the
Overpass API, preprocesses it, and saves the result as a compressed JSON file
(for consumption by EVMap).

## Data Format

The resulting gzipped JSON file contains data in the following format:

```json5
{
  "timestamp": 1633282807, // UNIX timestamp in seconds
  "count": 4376, // Number of elements below
  "elements": [
    {
      // Unique numeric ID
      "id": 9079237567,
      // Latitude, longitude (WGS84 coordinates, I assume)
      "lat": 47.0701573,
      "lon": 7.5664432,
      // Type of element (node, way, relation)
      "type": "node",
      // Timestamp of last update
      "timestamp": "2021-09-10T11:47:56Z",
      // Numeric, monotonically increasing version number
      "version": 1,
      // User that last modified this POI
      "user": "dbrgn",
      // Raw key-value OSM tags
      "tags": {
        "amenity": "charging_station",
        ... // More OSM tags
      }
    },
    ... // More elements
  ]
}
```

For the tagging schema, see <https://wiki.openstreetmap.org/wiki/DE:Tag:amenity%3Dcharging_station>

## Building

Make sure you have Rust and Cargo installed.

Build the project:

    cargo build --release

The binary will be available at `target/release/evmap-osm-loader`.

## Usage

### Basic Usage

Download charging station data with default settings:

    evmap-osm-loader

**Note:** The API query may take multiple minutes. The default timeout is set
to 15 minutes, but depending on the load on the API endpoint, this may not be
sufficient.

Show usage:

    evmap-osm-loader --help

### Command-Line Options

```
evmap-osm-loader [OPTIONS]

Options:
      --keep-intermediate <BOOL>
          Keep intermediate raw response file
          [default: false]

      --overpass-api-endpoint <OVERPASS_API_ENDPOINT>
          Overpass API to use ('switzerland', 'world', or a custom URL)

          The following hardcoded endpoints are supported:
          - switzerland (https://overpass.osm.ch/api/interpreter)
          - world (https://overpass-api.de/api/interpreter)

      --timeout-seconds <TIMEOUT_SECONDS>
          Timeout in seconds for the Overpass query
          [default: 900]

      --outfile-raw <OUTFILE_RAW>
          Output file for raw response (only written if --keep-intermediate=true)
          [default: overpass-result.json]

      --outfile-compressed <OUTFILE_COMPRESSED>
          Output file for compressed result
          [default: charging-stations-osm.json.gz]

  -h, --help
          Print help

  -V, --version
          Print version
```

## Query Details

The tool executes the following Overpass QL query:

```
[out:json][timeout:900];
(
  node[amenity=charging_station];
  area[amenity=charging_station];
  relation[amenity=charging_station];
);
out meta qt;
```

This query retrieves:

- All nodes tagged as charging stations
- All areas (ways) tagged as charging stations
- All relations tagged as charging stations

## Future Enhancements

The async architecture allows for future parallelization opportunities, such as:

- Parallel processing of large datasets
- Concurrent API requests for different regions
- Streamed parsing and processing of HTTP response

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
   http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   http://opensource.org/licenses/MIT) at your option.

### Contributions

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
