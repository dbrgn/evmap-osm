# EVMap OSM Data Updater (PoC)

This is a proof-of-concept for a charging station data fetcher for
OpenStreetMap. The script fetches all charging stations worldwide through the
Overpass API, preprocesses the data and writes it to a gzip-compressed JSON
file.

## Requirements

- [curl](https://curl.se/)
- [jq](https://stedolan.github.io/jq/)
- [gzip](https://www.gnu.org/software/gzip/)

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
