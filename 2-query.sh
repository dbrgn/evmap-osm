#!/bin/bash
set -euo pipefail

function log() { echo -e "\e[32m$1\e[0m"; }
function loge() { echo -e "\e[31m$1\e[0m"; }

DB="charging-station-data.sqlite"
OUTPUT="out.json"

if [ ! -f "$DB" ]; then
    loge "Database $DB not found. Run ./1-load.sh first."
    exit 1
fi

log "Querying database for charging stations and charge points..."

# Run the spatial query and output JSON
sqlite3 "$DB" << 'SQL' | jq '.' > "$OUTPUT"
.load mod_spatialite
.mode list
.separator ''
.headers off

WITH
-- Get all charging station points
station_points AS (
    SELECT
        osm_id as id,
        Y(Transform(GEOMETRY, 4326)) as lat,
        X(Transform(GEOMETRY, 4326)) as lon,
        'node' as type,
        osm_timestamp as timestamp,
        osm_version as version,
        amenity,
        name,
        operator,
        brand,
        capacity,
        fee,
        NULL as geometry_obj,
        'point' as geom_type
    FROM points
    WHERE amenity = 'charging_station'
),
-- Get all charging station lines (convert to ways)
station_lines AS (
    SELECT
        osm_id as id,
        NULL as lat,
        NULL as lon,
        'way' as type,
        osm_timestamp as timestamp,
        osm_version as version,
        amenity,
        name,
        operator,
        brand,
        capacity,
        fee,
        CASE
            WHEN IsClosed(GEOMETRY) = 1 THEN MakePolygon(GEOMETRY)
            ELSE GEOMETRY
        END as geometry_obj,
        'line' as geom_type
    FROM lines
    WHERE amenity = 'charging_station'
),
-- Get all charging station multipolygons
station_polygons AS (
    SELECT
        osm_id as id,
        NULL as lat,
        NULL as lon,
        'way' as type,
        osm_timestamp as timestamp,
        osm_version as version,
        amenity,
        name,
        operator,
        brand,
        capacity,
        fee,
        GEOMETRY as geometry_obj,
        'polygon' as geom_type
    FROM multipolygons
    WHERE amenity = 'charging_station'
),
-- Combine all charging stations
all_stations AS (
    SELECT * FROM station_points
    UNION ALL
    SELECT * FROM station_lines
    UNION ALL
    SELECT * FROM station_polygons
),
-- Get all charge points
all_charge_points AS (
    SELECT
        osm_id,
        Y(Transform(GEOMETRY, 4326)) as lat,
        X(Transform(GEOMETRY, 4326)) as lon,
        GEOMETRY,
        osm_timestamp,
        osm_version,
        man_made,
        name as cp_name,
        operator as cp_operator,
        brand as cp_brand,
        capacity as cp_capacity,
        fee as cp_fee
    FROM points
    WHERE man_made = 'charge_point'
),
-- Find charge points within station areas
station_charge_points AS (
    SELECT
        s.id as station_id,
        json_group_array(
            json_object(
                'id', cp.osm_id,
                'lat', cp.lat,
                'lon', cp.lon,
                'type', 'node',
                'timestamp', cp.osm_timestamp,
                'version', cp.osm_version,
                'tags', json_object(
                    'man_made', cp.man_made,
                    'name', cp.cp_name,
                    'operator', cp.cp_operator,
                    'brand', cp.cp_brand,
                    'capacity', cp.cp_capacity,
                    'fee', cp.cp_fee
                )
            )
        ) as charge_points_json
    FROM all_stations s
    INNER JOIN all_charge_points cp
        ON s.geometry_obj IS NOT NULL
        AND Contains(s.geometry_obj, cp.GEOMETRY)
    GROUP BY s.id
),
-- Prepare final result for each station
stations_with_charge_points AS (
    SELECT
        s.id,
        s.lat,
        s.lon,
        s.type,
        s.timestamp,
        s.version,
        s.amenity,
        s.name,
        s.operator,
        s.brand,
        s.capacity,
        s.fee,
        CASE
            WHEN scp.charge_points_json IS NOT NULL
                AND json_array_length(scp.charge_points_json) > 0
            THEN scp.charge_points_json
            ELSE NULL
        END as charge_points
    FROM all_stations s
    LEFT JOIN station_charge_points scp ON s.id = scp.station_id
)
-- Final output in EVMap format
SELECT json_object(
    'timestamp', CAST(strftime('%s', 'now') AS INTEGER),
    'count', COUNT(*),
    'elements', json_group_array(
        json_object(
            'id', id,
            'lat', lat,
            'lon', lon,
            'type', type,
            'timestamp', timestamp,
            'version', version,
            'tags', json_object(
                'amenity', amenity,
                'name', name,
                'operator', operator,
                'brand', brand,
                'capacity', capacity,
                'fee', fee
            ),
            'charge_points', CASE
                WHEN charge_points IS NOT NULL THEN json(charge_points)
                ELSE NULL
            END
        )
    )
)
FROM stations_with_charge_points;
SQL

log "Query complete. Output written to $OUTPUT"

# Show some statistics
if command -v jq &> /dev/null; then
    TOTAL=$(jq '.count' "$OUTPUT" 2>/dev/null || echo "0")
    WITH_POINTS=$(jq '[.elements[] | select(.charge_points != null)] | length' "$OUTPUT" 2>/dev/null || echo "0")
    TOTAL_CHARGE_POINTS=$(jq '[.elements[].charge_points[]?] | length' "$OUTPUT" 2>/dev/null || echo "0")

    log "Statistics:"
    log "  Total charging stations: $TOTAL"
    log "  Stations with charge points: $WITH_POINTS"
    log "  Total nested charge points: $TOTAL_CHARGE_POINTS"
fi

# Show file size
if [ -f "$OUTPUT" ]; then
    SIZE=$(ls -lh "$OUTPUT" | awk '{print $5}')
    log "Output file size: $SIZE"
fi
