-- Create table with values to calculate the p1 AQI
CREATE TABLE pm25_rows(imin double precision NOT NULL, imax double precision NOT NULL, bpmin double precision NOT NULL, bpmax double precision NOT NULL);
INSERT INTO pm25_rows(imin, imax, bpmin, bpmax) VALUES (0.0, 50.0, 0.0, 12.0), (51.0, 100.0, 12.1, 35.4), (101.0, 150.0, 35.5, 55.4), (151.0, 200.0, 55.5, 150.4), (201.0, 300.0, 150.5, 250.4), (301.0, 400.0, 250.5, 350.4), (401.0, 500.0, 350.5, 500.4);

-- Create table with values to calculate the p2 AQI
CREATE TABLE pm10_rows(imin double precision NOT NULL, imax double precision NOT NULL, bpmin double precision NOT NULL, bpmax double precision NOT NULL);
INSERT INTO pm10_rows(imin, imax, bpmin, bpmax) VALUES (0.0, 50.0, 0.0, 54.0), (51.0, 100.0, 55.0, 154.0), (101.0, 150.0, 155.0, 254.0), (151.0, 200.0, 255.0, 354.0), (201.0, 300.0, 355.0, 424.0), (301.0, 400.0, 425.0, 504.0), (401.0, 500.0, 505.0, 604.0);

-- Calculate p1 AQI using the pm25_rows table
CREATE OR REPLACE FUNCTION aqi_p1("p1" NUMERIC)
RETURNS DOUBLE PRECISION
LANGUAGE SQL
AS $$
SELECT ((imax - imin) / (bpmax - bpmin) * ("p1" - bpmin) + imin) as aqi
FROM pm25_rows
WHERE "p1" between bpmin AND bpmax;
$$;


-- Calculate p2 AQI using the pm10_rows table
-- (same as the p1 function)
CREATE OR REPLACE FUNCTION aqi_p2("p2" NUMERIC)
RETURNS DOUBLE PRECISION
LANGUAGE SQL
AS $$
SELECT ((imax - imin) / (bpmax - bpmin) * ("p2" - bpmin) + imin) as aqi
FROM pm10_rows
WHERE "p2" between bpmin AND bpmax;
$$;

-- Calculates the aqi by using two particle concentrations
CREATE OR REPLACE FUNCTION aqi("p1" NUMERIC, "p2" NUMERIC)
RETURNS DOUBLE PRECISION
LANGUAGE SQL
AS $$
SELECT greatest(aqi_p1("p1"), aqi_p2("p2")) as aqi
$$;

