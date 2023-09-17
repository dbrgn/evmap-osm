#!/bin/bash
#
# Download worldwide charging station data from Overpass API and write them to
# a compressed JSON file.
#
# Requirements:
#
# - curl
# - jq

set -euo pipefail

# Configuration

#OVERPASS_INTERPRETER="https://overpass.osm.ch/api/interpreter" # CH only, good for quick testing
OVERPASS_INTERPRETER="https://overpass-api.de/api/interpreter"
TIMEOUT_SECONDS=900 # 15m
OUTFILE_RAW="overpass-result.json"
OUTFILE_COMPRESSED="charging-stations-osm.json.gz"
CURL_BIN=curl
JQ_BIN=jq
GZIP_BIN=gzip

# Helper functions

function log() { echo -e "\e[32m$1\e[0m"; }
function loge() { echo -e "\e[31m$1\e[0m"; }

# Download

log "1: Downloading data through Overpass API (this may take up to $TIMEOUT_SECONDS seconds...)"
$CURL_BIN \
    --data "[out:json][timeout:$TIMEOUT_SECONDS]; node[amenity=charging_station]; out meta qt;" \
    --header 'content-type: text/plain' \
    -o $OUTFILE_RAW \
    $OVERPASS_INTERPRETER
found_elements=$(jq ".elements | length" $OUTFILE_RAW)
if [ "$found_elements" -eq 0 ]; then
    loge "Query failed, found 0 elements."
    loge "Details: $(jq -r .remark $OUTFILE_RAW)"
    exit 1
fi
size_raw=$(du -h $OUTFILE_RAW | cut -f1)

# Process

log "2: Processing $found_elements entries in $size_raw of raw JSON"
$JQ_BIN "{
    timestamp: now,
    count: $found_elements,
    elements: [
        .elements[] | {id,lat,lon,timestamp,version,user,tags}
    ]
}"  $OUTFILE_RAW | $GZIP_BIN -9 > $OUTFILE_COMPRESSED
size_compressed=$(du -h $OUTFILE_COMPRESSED | cut -f1)
log "Done: $OUTFILE_COMPRESSED ($size_compressed)"
