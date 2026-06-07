INSERT INTO urban_topology (neighborhood_name, feature_name, footprint, elevation_min, elevation_max)
VALUES
    (
        'Daryakenar',
        'Coastal Parcel A',
        ST_SetSRID(
            ST_GeomFromText('POLYGON((52.610 36.700, 52.615 36.700, 52.615 36.705, 52.610 36.705, 52.610 36.700))'),
            4326
        ),
        0,
        12.5
    ),
    (
        'Daryakenar',
        'Coastal Parcel B',
        ST_SetSRID(
            ST_GeomFromText('POLYGON((52.620 36.710, 52.625 36.710, 52.625 36.715, 52.620 36.715, 52.620 36.710))'),
            4326
        ),
        0,
        18.0
    ),
    (
        'Iranshahr',
        'Harbor Block 1',
        ST_SetSRID(
            ST_GeomFromText('POLYGON((60.680 27.200, 60.685 27.200, 60.685 27.205, 60.680 27.205, 60.680 27.200))'),
            4326
        ),
        0,
        9.0
    ),
    (
        'Iranshahr',
        'Harbor Block 2',
        ST_SetSRID(
            ST_GeomFromText('POLYGON((60.690 27.210, 60.695 27.210, 60.695 27.215, 60.690 27.215, 60.690 27.210))'),
            4326
        ),
        0,
        14.5
    );
