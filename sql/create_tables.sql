BEGIN;
CREATE TABLE meas_current(batchnum integer, at timestamp, location geometry(point), p1 double precision, p2 double precision);
CREATE TABLE meas_lastyear(batchnum integer, at timestamp, location geometry(point), p1 double precision, p2 double precision);
CREATE TABLE locations(postalcode text primary key, city text not null, geog geometry(multipolygon));
COMMIT;
