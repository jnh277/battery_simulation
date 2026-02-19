use std::path::Path;
use serde::Deserialize;
use crate::types::{TelemetryPoint, Duration, Power};

#[derive(Debug, Deserialize)]
struct CsvRow {
    duration_hour: f64,
    solar_power_kw: f64,
    load_power_kw: f64,
}

#[derive(Debug, thiserror::Error)]
pub enum CsvParseError {
    #[error("CSV error: {0}")]
    Csv(#[from] csv::Error),
    #[error("Invalid duration value at row {row}: {value}")]
    InvalidDuration { row: usize, value: f64 },
    #[error("Invalid solar power value at row {row}: {value}")]
    InvalidSolarPower { row: usize, value: f64 },
    #[error("Invalid load power value at row {row}: {value}")]
    InvalidLoadPower { row: usize, value: f64 },
}

pub fn read_telemetry_csv<P: AsRef<Path>> (path: P) -> Result<Vec<TelemetryPoint>, CsvParseError> {
    let mut reader = csv::ReaderBuilder::new()
    .trim(csv::Trim::All)
    .from_path(path)?;

    let mut telemetry = Vec::new();
    for (idx, result) in reader.deserialize().enumerate() {
        let row: CsvRow = result?;
        let row_num = idx + 2;

        let duration: Duration = Duration::from_hour(row.duration_hour)
            .map_err(|value:f64| CsvParseError::InvalidDuration { row: row_num, value })?;
        let load_power: Power = Power::from_kw(row.load_power_kw)
            .map_err(|value:f64| CsvParseError::InvalidLoadPower { row: row_num, value })?;
        let solar_power: Power = Power::from_kw(row.solar_power_kw)
            .map_err(|value:f64| CsvParseError::InvalidSolarPower { row: row_num, value })?;

        telemetry.push(TelemetryPoint::new(duration, solar_power, load_power));

    }
    Ok(telemetry)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{kw, hour};
    #[test]
    fn test_read_telemetry_csv() {
        let telemetry = read_telemetry_csv("data/test_data.csv").expect("Should read telemetry");
        assert_eq!(telemetry.len(), 5);
        assert_eq!(telemetry[0].solar_power(), kw!(5.));
        assert_eq!(telemetry[0].duration(), hour!(0.5));
        assert_eq!(telemetry[0].load_power(), kw!(1.0));
        assert_eq!(telemetry[4].solar_power(), kw!(0.));
        assert_eq!(telemetry[4].duration(), hour!(0.5));
        assert_eq!(telemetry[4].load_power(), kw!(4.5));
    }
}