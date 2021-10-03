# EVMap OSM Data Updater (PoC)

This is a proof-of-concept for a charging station data fetcher for
OpenStreetMap. The script fetches all charging stations worldwide through the
Overpass API, preprocesses the data and writes it to a gzip-compressed JSON
file.

## Requirements

- [curl](https://curl.se/)
- [jq](https://stedolan.github.io/jq/)
- [gzip](https://www.gnu.org/software/gzip/)

## Data Format

The resulting gzipped JSON file contains data in the following format:

```json5
{
  "timestamp": 1633282807.294814, // UNIX timestamp in seconds
  "elements": [
    {
      // Unique numeric ID
      "id": 9079237567,
      // Latitude, longitude (WGS84 coordinates, I assume)
      "lat": 47.0701573,
      "lon": 7.5664432,
      // Timestamp of last update
      "timestamp": "2021-09-10T11:47:56Z",
      // Numeric, monotonically increasing version number
      "version": 1,
      // User that last modified this POI
      "user": "dbrgn",
      // Raw key-value OSM tags
      "tags": {
        "amenity": "charging_station",
        ...
      }
    },
    ...
  ]
}
```

For the tagging schema, see <https://wiki.openstreetmap.org/wiki/DE:Tag:amenity%3Dcharging_station>

## Usage

Simply invoke the script:

    ./load-overpass.sh

Note: The API query may take multiple minutes. The default timeout is set to 15
minutes, but depending on the load on the API endpoint, this may not be
sufficient.

The script can be adjusted by editing the configuration variables. Possible
configuration variables include the Overpass API endpoint or the download
timeout.

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
