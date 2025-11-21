-- migrations/001_init.sql

CREATE TABLE IF NOT EXISTS earthquakes (
    id UUID PRIMARY KEY,
    usgs_id TEXT UNIQUE,
    location TEXT NOT NULL,
    magnitude REAL NOT NULL,
    depth_km REAL NOT NULL,
    latitude REAL NOT NULL,
    longitude REAL NOT NULL,
    time TIMESTAMP WITH TIME ZONE NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_earthquakes_time ON earthquakes (time DESC);
CREATE INDEX IF NOT EXISTS idx_earthquakes_magnitude ON earthquakes (magnitude DESC);
CREATE INDEX IF NOT EXISTS idx_earthquakes_usgs_id ON earthquakes (usgs_id);
