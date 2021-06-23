-- Aggregates the p1/p2 values over 24 hours in steps of
-- 5 minutes, like the preaggregation stage/ParticleAggregate
-- of the q1 binary. Used to compare a cities values to
-- those of the q1 binary.
WITH testcity(city,geog) AS
	(SELECT city,geog FROM locations WHERE city='Markdorf'),
tstart(t) AS (SELECT * FROM generate_series(1583128500-24*60*60, 1583128499, 300))
SELECT 
	testcity.city as city,
	tstart.t as startts,
	tstart.t+299 as endts,
	sum(m.p1) as p1sum,
	sum(m.p2) as p2sum,
	count(*) as eventcount
FROM meas_current m, testcity, tstart
WHERE ST_Contains(testcity.geog, m.location)
	AND at BETWEEN (to_timestamp(tstart.t) AT TIME ZONE 'GMT') AND (to_timestamp(tstart.t+299) AT TIME ZONE 'GMT')
GROUP BY testcity.city,tstart.t
ORDER BY startts ASC;
