#!/bin/bash
set -euo pipefail

function log() { echo -e "\e[32m$1\e[0m"; }
function loge() { echo -e "\e[31m$1\e[0m"; }

# Parse arguments
if [ "$#" -ne 1 ]; then
    echo "Download and parse OSM file."
    echo ""
    echo "Usage: $0 (planet|switzerland)"
    echo "Example: $0 planet"
    exit 1
fi
target=$1

# Download data file
if [ "$target" == "switzerland" ]; then
    DATA=switzerland-latest.osm.pbf
    URL="https://download.geofabrik.de/europe/$DATA"
elif [ "$target" == "planet" ]; then
    DATA=planet-latest.osm.pbf
    URL="https://planet.openstreetmap.org/pbf/$DATA"
else
    loge "Invalid target: $target"
    exit 1
fi
if [ -f "$DATA" ]; then
    log "$DATA already present"
else
    log "Downloading $DATA..."
    curl -L -O $URL
fi

# Filter OSM data
log "Filtering OSM data..."
osmium tags-filter $DATA \
    w/amenity=charging_station \
    n/amenity=charging_station \
    n/man_made=charge_point \
    --overwrite \
    -o charging-station-data.osm.pbf

# Load into database
DB="charging-station-data.sqlite"
log "Clean existing database..."
rm -f "$DB"
log "Load into database..."
ogr2ogr -f SQLite "$DB" \
    -dsco SPATIALITE=YES \
    -dsco INIT_WITH_EPSG=YES \
    -lco SPATIAL_INDEX=YES \
    -gt 65536 \
    -progress \
    --config OSM_CONFIG_FILE osmconf.ini \
    --config OGR_SQLITE_SYNCHRONOUS OFF \
    charging-station-data.osm.pbf

log "Database tables created:"
ogrinfo -so "$DB" | grep -E "^[0-9]+:"
