use crate::types::{Duration, Power, TelemetryPoint};
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize)]
struct CsvRow {
    duration_hour: Duration,
    solar_power_kw: Power,
    load_power_kw: Power,
}

impl From<CsvRow> for TelemetryPoint {
    fn from(value: CsvRow) -> Self {
        Self::new(
            value.duration_hour,
            value.solar_power_kw,
            value.load_power_kw,
        )
    }
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

pub fn read_telemetry_csv<P: AsRef<Path>>(path: P) -> Result<Vec<TelemetryPoint>, CsvParseError> {
    let mut reader = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_path(path)?;

    let mut telemetry = Vec::new();
    for result in reader.deserialize() {
        let row: CsvRow = result?;

        telemetry.push(row.into());
    }
    Ok(telemetry)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{hour, kw};
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

