-- calculate aqi improvement in time
-- 31536000 = 60*60*24*365 (1 year)
-- 1583020800 = TZ=GMT date "+%s" --date="2020-03-01 00:00:00"
WITH current(cityname, p1, p2) AS
    (SELECT l.city, round(avg(p1)::numeric,3) as mp1, round(avg(p2)::numeric,3) as mp2
    FROM locations l, meas_current m
    WHERE ST_Contains(l.geog, m.location) AND at BETWEEN (to_timestamp(1583020800) AT TIME ZONE 'GMT') AND (to_timestamp(1583020800-1+60*60*24*5) AT TIME ZONE 'GMT')
    GROUP BY city
    ORDER BY mp1 DESC),
lastyear(cityname, p1, p2) AS
    (SELECT l.city, round(avg(p1)::numeric,3) as mp1, round(avg(p2)::numeric,3) as mp2
    FROM locations l, meas_lastyear m
    WHERE ST_Contains(l.geog, m.location) AND at BETWEEN (to_timestamp(1583020800-31536000) AT TIME ZONE 'GMT') AND (to_timestamp(1583020800-1-31536000+60*60*24*5) AT TIME ZONE 'GMT')
    GROUP BY city
    ORDER BY mp1 DESC)
SELECT
    current.cityname,
    round(aqi(current.p1, current.p2)::numeric, 3) as current_aqi,
    round(aqi(lastyear.p1, lastyear.p2)::numeric, 3) as lastyear_aqi,
    round((aqi(current.p1, current.p2) - aqi(lastyear.p1, lastyear.p2))::numeric, 3) as improvement
FROM current, lastyear
WHERE current.cityname = lastyear.cityname
ORDER BY improvement ASC;
