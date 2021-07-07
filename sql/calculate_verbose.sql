-- calculate aqi improvement in time
-- for a specific city, with extra information
-- about event count and the unrounded p1/p2 values
-- 31536000 = 60*60*24*365 (1 year)
-- 1583020800 = TZ=GMT date "+%s" --date="2020-03-01 00:00:00"
WITH current(cityname, p1, p2) AS
    (SELECT l.city, round(avg(p1)::numeric,3) as mp1, round(avg(p2)::numeric,3) as mp2, count(*) as eventcount
    FROM locations l, meas_current m
    WHERE ST_Contains(l.geog, m.location) AND at BETWEEN (to_timestamp(1583020800) AT TIME ZONE 'GMT') AND (to_timestamp(1583020800-1+60*60*24*5) AT TIME ZONE 'GMT')
	AND m.p1 > 0 AND m.p2 > 0
    GROUP BY city
    ORDER BY mp1 DESC),
lastyear(cityname, p1, p2) AS
    (SELECT l.city, round(avg(p1)::numeric,3) as mp1, round(avg(p2)::numeric,3) as mp2, count(*) as eventcount
    FROM locations l, meas_lastyear m
    WHERE ST_Contains(l.geog, m.location) AND at BETWEEN (to_timestamp(1583020800-31536000) AT TIME ZONE 'GMT') AND (to_timestamp(1583020800-1-31536000+60*60*24*5) AT TIME ZONE 'GMT')
	AND m.p1 > 0 AND m.p2 > 0
    GROUP BY city
    ORDER BY mp1 DESC)
SELECT
    current.cityname,
	current.eventcount as current_eventcount,
	lastyear.eventcount as lastyear_eventcount,
	current.p1 as current_fiveday_p1,
	current.p2 as current_fiveday_p2,
	lastyear.p1 as lastyear_fiveday_p1,
	lastyear.p2 as lastyear_fiveday_p2,
    round(aqi(current.p1, current.p2)::numeric, 3) as current_aqi,
    round(aqi(lastyear.p1, lastyear.p2)::numeric, 3) as lastyear_aqi,
    round((aqi(current.p1, current.p2) - aqi(lastyear.p1, lastyear.p2))::numeric, 3) as improvement
FROM current, lastyear
WHERE current.cityname = lastyear.cityname AND current.cityname = 'Clausthal-Zellerfeld, Oberschulenberg'
ORDER BY improvement ASC;
