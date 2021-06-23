-- calculates the AQI for the last 24 hours for a given city
WITH current(cityname, p1, p2) AS
    (SELECT l.city, round(avg(p1)::numeric,3) as mp1, round(avg(p2)::numeric,3) as mp2, count(*) as eventcount
    FROM locations l, meas_current m
    WHERE ST_Contains(l.geog, m.location) AND at BETWEEN (to_timestamp(1583128500-24*60*60) AT TIME ZONE 'GMT') AND (to_timestamp(1583128499) AT TIME ZONE 'GMT')
    GROUP BY city
    ORDER BY mp1 DESC)
SELECT
    current.cityname,
	current.eventcount,
	current.p1,
	current.p2,
	aqi_p1(current.p1) as aqip1,
	aqi_p2(current.p2) as aqip2,
    round(aqi(current.p1, current.p2)::numeric, 3) as current_aqi
FROM current
WHERE current.cityname = 'Markdorf';
