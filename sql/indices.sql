-- Localize faster by creating spatial indexes
CREATE INDEX geog_idx_gist ON locations USING GIST(geog);
CREATE INDEX geog_idx_brin ON locations USING BRIN(geog);

-- Make it possible to quickly limit by timestamp
CREATE INDEX ts_idx ON meas_current(at);
CREATE INDEX ts_lastyear_idx ON meas_lastyear(at);
