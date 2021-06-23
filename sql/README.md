# About
This subdirectory contains various SQL files used to test
the plausibility of results calculated by the q1 binary.

# Setup
This guide assumes that
- PostgreSQL and the PostGIS extension are installed
- the code in this repository has been compiled in release mode
- `DEBS_DATA_ROOT` is set correctly (see the README in the parent directory)
- the psql binary is available and it is possible to connect to the database
  by running `psql`

# SQL data conversion
Create the SQL output directories:
```sh
mkdir -p ${DEBS_DATA_ROOT}/sql/{0..100}
```

Run the `geo2sql` and `batch2sql` binaries to convert the
data to SQL. The binaries will write into the created folders.
The data can now imported.
Note: As the SQL files get quite large, currently only
the 10% of the files are converted.

# SQL data import
## Table creation
Run
```sh
psql -f setup.sql
```
to create the necessary tables.

## The actual import
Note: The import can take a while (it took one hour to import 10%
of the data on my laptop, which the `gen_import.sh` is currently set
to import).

Then, run
```sh
./gen_import.sh > import.sql
```
to generate an psql script containing instructions,
and run it using
```sh
psql -f import.sql
```

# Calculating top cities with SQL
Use
```bash
psql -f calculate.sql | tee results.out
```
to calculate the list of top cities.
Contrary to the binary,
all cities are calculated (not only the top 50).
The data is calculated for the first pair of 5-day windows.
(2020-03-01 00:00:00 to 2020-03-06 00:00:00 for current year,
2019-03-02 00:00:00 to 2019-03-07 00:00:00 for the last year).
For other times, `calculate.sql` will need to be adapted.



# Short table overview
The database will contain 5 tables:
- 2 containing the measurement data (1 for current year, 1 for last year)
- 1 for the location data
- 2 for the AQI calculation
## Measurement data
The `meas_current`/`meas_lastyear` tables contain the batch number, the timestamp (at second precision)
and the particle concentrations.
See `create_tables.sql` for the schema.

## Location data
The `locations` table uses the postal code as a primary key. Every entry has a city name and a geography.
See `create_tables.sql` for the schema.

## AQI calculation
For AQI calculation, two tables are used, `pm25_rows` and `pm10_rows`,
each using the values provided in the pdf
[here](https://web.archive.org/web/20201026120832if_/https://www.airnow.gov/sites/default/files/2018-05/aqi-technical-assistance-document-may2016.pdf#page=14).
The calculations are performed using the `aqi`/`aqi_p1`/`aqi_p2` functions.
See `aqi.sql` for the schema and values.
