/// Represents an AQI value calculated from
/// particle concentrations
#[derive(Debug, Copy, Clone)]
pub struct AQIValue(f32);

use std::ops::RangeInclusive;

// Code based on data & formula
// from https://www.airnow.gov/sites/default/files/2018-05/aqi-technical-assistance-document-may2016.pdf
// pages 14/15

// The data for these table Rows
// is based on Table 6
#[allow(non_snake_case)]
struct AQITableRow {
    // AQI range ("..equal to this AQI")
    I: RangeInclusive<f32>,
    // range for the values ("These Breakpoints..")
    BP: RangeInclusive<f32>,
}

/// The values for PM2.5, 24-hour 
const PM25_ROWS : [AQITableRow; 7] = [
    AQITableRow {
        I: 0.0..=50.0,
        BP: 0.0..=12.0,
    },
    AQITableRow {
        I: 51.0..=100.0,
        BP: 12.1..=35.4,
    },
    AQITableRow {
        I: 101.0..=150.0,
        BP: 35.5..=55.4,
    },
    AQITableRow {
        I: 151.0..=200.0,
        BP: 55.5..=150.4,
    },
    AQITableRow {
        I: 201.0..=300.0,
        BP: 150.5..=250.4,
    },
    AQITableRow {
        I: 301.0..=400.0,
        BP: 250.5..=350.4,
    },
    AQITableRow {
        I: 401.0..=500.0,
        BP: 350.5..=500.4,
    },
];

/// The values for PM10, 24-hour 
const PM10_ROWS : [AQITableRow; 7] = [
    AQITableRow {
        I: 0.0..=50.0,
        BP: 0.0..=54.0
    },
    AQITableRow {
        I: 51.0..=100.0,
        BP: 55.0..=154.0,
    },
    AQITableRow {
        I: 101.0..=150.0,
        BP: 155.0..=254.0,
    },
    AQITableRow {
        I: 151.0..=200.0,
        BP: 255.0..=354.0,
    },
    AQITableRow {
        I: 201.0..=300.0,
        BP: 355.0..=424.0,
    },
    AQITableRow {
        I: 301.0..=400.0,
        BP: 425.0..=504.0,
    },
    AQITableRow {
        I: 401.0..=500.0,
        BP: 505.0..=604.0,
    },
];


/// calculates the AQI value using the given lookup table and concentration.
/// and returns it.
/// returns None if C_p is outside the valid Range
/// Variable names correspond to those in Eqation 1.
#[allow(non_snake_case)]
#[inline]
fn aqi_from_table(table: &[AQITableRow], C_p: f32) -> Option<AQIValue> {
    for row in table {
        if row.BP.contains(&C_p) {
            let I_Lo = row.I.start();
            let I_Hi = row.I.end();
            let BP_Lo = row.BP.start();
            let BP_Hi = row.BP.end();
            let aqi = (I_Hi - I_Lo) / (BP_Hi - BP_Lo) * (C_p - BP_Lo) + I_Lo;
            debug_assert!(aqi >= 0.0);
            return Some(AQIValue(aqi));
        }
    }
    return None;
}

impl AQIValue {
    /// returns the rounded AQI value
    pub fn get(self) -> u16 {
        self.0.round() as u16
    }

    /// returns the AQI value, rounded to 3 digits
    /// and multiplited by 1000
    pub fn get_asdebs(self) -> i32 {
        (self.0 * 1000.0).round() as i32
    }

    /// calculate the AQI from a PM2.5 concentration (p2)
    pub fn from_pm25(pm25: f32) -> Option<AQIValue> {
        let pm25 = (pm25 * 10.0).round() / 10.0;
        aqi_from_table(&PM25_ROWS, pm25)
    }
    
    /// calculate the AQI from a PM10 concentration (p1)
    pub fn from_pm10(pm10: f32) -> Option<AQIValue> {
        let pm10 = pm10.round();
        aqi_from_table(&PM10_ROWS, pm10)
    }
}

#[cfg(test)]
mod tests {
    use super::AQIValue;

    #[test]
    fn test_all_pm25_aqis_work() {
        for i in 0..=5004 {
            assert!(AQIValue::from_pm25(i as f32 / 10.0).is_some());
        }
    }

    #[test]
    fn test_all_pm10_aqis_work() {
        for i in 0..=6040 {
            assert!(AQIValue::from_pm10(i as f32 / 10.0).is_some());
        }
    }

    #[test]
    fn test_valid_pm25_aqis() {
        assert_eq!(AQIValue::from_pm25(0.0).map(AQIValue::get), Some(0));
        assert_eq!(AQIValue::from_pm25(55.549).map(AQIValue::get), Some(151));
        assert_eq!(AQIValue::from_pm25(500.4).map(AQIValue::get), Some(500));
        assert_eq!(AQIValue::from_pm25(55.56).map(AQIValue::get_asdebs), Some(151052));
    }

    #[test]
    fn test_valid_pm10_aqis() {
        assert_eq!(AQIValue::from_pm10(0.0).map(AQIValue::get), Some(0));
        assert_eq!(AQIValue::from_pm10(155.049).map(AQIValue::get), Some(101));
        assert_eq!(AQIValue::from_pm10(604.49).map(AQIValue::get), Some(500));
    }
}
