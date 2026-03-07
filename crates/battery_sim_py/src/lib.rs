use pyo3::prelude::*;
use pyo3::exceptions::{PyValueError, PyRuntimeError};
use numpy::{PyReadonlyArray1, PyArray1};

// Import from the core library - use :: prefix to avoid ambiguity with the pymodule name
use ::battery_sim::battery::Battery;
use ::battery_sim::simulation::simulate_load_following;
use ::battery_sim::types::{Duration, Power, Energy, Efficiency, TelemetryPoint};

// ============================================================================
// PyBattery Class
// ============================================================================

/// A battery with configurable capacity, max power, and round-trip efficiency.
#[pyclass(name = "Battery")]
pub struct PyBattery {
    inner: Battery,
}

#[pymethods]
impl PyBattery {
    /// Create a new Battery.
    ///
    /// Parameters
    /// ----------
    /// capacity_kwh : float
    ///     Battery capacity in kWh. Must be positive.
    /// max_power_kw : float
    ///     Maximum charge/discharge power in kW. Must be positive.
    /// efficiency : float
    ///     Round-trip efficiency as a fraction (0 < efficiency <= 1).
    ///
    /// Raises
    /// ------
    /// ValueError
    ///     If any parameter is invalid.
    #[new]
    fn new(capacity_kwh: f64, max_power_kw: f64, efficiency: f64) -> PyResult<Self> {
        let efficiency = Efficiency::from_fraction(efficiency)
            .map_err(|v| PyValueError::new_err(format!("invalid efficiency: {}", v)))?;
        let capacity = Energy::from_kwh(capacity_kwh)
            .map_err(|v| PyValueError::new_err(format!("invalid capacity: {}", v)))?;
        let max_power = Power::from_kw(max_power_kw)
            .map_err(|v| PyValueError::new_err(format!("invalid max_power: {}", v)))?;

        let inner = Battery::new(capacity, max_power, efficiency)
            .map_err(|e| PyValueError::new_err(e.to_string()))?;

        Ok(PyBattery { inner })
    }

    /// Battery capacity in kWh.
    #[getter]
    fn capacity_kwh(&self) -> f64 {
        self.inner.capacity().as_kwh()
    }

    /// Maximum charge/discharge power in kW.
    #[getter]
    fn max_power_kw(&self) -> f64 {
        self.inner.max_power().as_kw()
    }

    /// Round-trip efficiency as a fraction.
    #[getter]
    fn efficiency(&self) -> f64 {
        self.inner.round_trip_efficiency().as_fraction()
    }

    fn __repr__(&self) -> String {
        format!(
            "Battery(capacity={:.1} kWh, max_power={:.1} kW, efficiency={:.1}%)",
            self.capacity_kwh(),
            self.max_power_kw(),
            self.efficiency() * 100.0
        )
    }
}

// ============================================================================
// Helper Functions (internal, pure Rust)
// ============================================================================

use ::battery_sim::battery::BatteryState;

/// Validates that all three input arrays have the same length.
fn validate_array_lengths(
    duration_len: usize,
    solar_len: usize,
    load_len: usize,
) -> Result<(), String> {
    if solar_len != duration_len || load_len != duration_len {
        return Err(
            "duration_hours, solar_power_kw, and load_power_kw must have the same length".to_string()
        );
    }
    if duration_len < 1 {
        return Err(
            "arrays must be at least 1 long.".to_string()
        )
    }
    Ok(())
}

/// Converts raw f64 slices to validated TelemetryPoint vector.
fn build_telemetry_points(
    duration: &[f64],
    solar: &[f64],
    load: &[f64],
) -> Result<Vec<TelemetryPoint>, String> {
    duration.iter()
        .zip(solar.iter())
        .zip(load.iter())
        .map(|((&d, &s), &l)| -> Result<TelemetryPoint, String> {
            let duration = Duration::from_hour(d)
                .map_err(|v| format!("invalid duration: {}", v))?;
            let solar_power = Power::from_kw(s)
                .map_err(|v| format!("invalid solar_power: {}", v))?;
            let load_power = Power::from_kw(l)
                .map_err(|v| format!("invalid load_power: {}", v))?;
            Ok(TelemetryPoint::new(duration, solar_power, load_power))
        })
        .collect()
}

/// Converts f64 values to validated BatteryState.
fn build_initial_state(
    battery: &Battery,
    initial_soc_kwh: f64,
    initial_power_kw: f64,
) -> Result<BatteryState, String> {
    let initial_soc = Energy::from_kwh(initial_soc_kwh)
        .map_err(|v| format!("invalid initial_soc: {}", v))?;
    let initial_power = Power::from_kw(initial_power_kw)
        .map_err(|v| format!("invalid initial_power: {}", v))?;

    battery.init_state(initial_soc, initial_power)
        .map_err(|e| e.to_string())
}

/// Extracts (soc, power) vectors from BatteryState slice.
fn extract_results(states: &[BatteryState]) -> (Vec<f64>, Vec<f64>) {
    let soc: Vec<f64> = states[1..].iter().map(|s| s.state_of_charge_kwh()).collect();
    let power: Vec<f64> = states[1..].iter().map(|s| s.power_kw()).collect();
    (soc, power)
}

// ============================================================================
// Python Function
// ============================================================================
type DoublePyArray = PyArray1<f64>;

/// Simulate battery load following behavior.
///
/// Parameters
/// ----------
/// duration_hours : numpy.ndarray
///     Duration of each time step in hours.
/// solar_power_kw : numpy.ndarray
///     Solar power generation at each time step in kW.
/// load_power_kw : numpy.ndarray
///     Load power consumption at each time step in kW.
/// battery : Battery
///     Battery object with capacity, max power, and efficiency.
/// initial_soc_kwh : float
///     Initial state of charge in kWh.
/// initial_power_kw : float
///     Initial power in kW.
///
/// Returns
/// -------
/// tuple[numpy.ndarray, numpy.ndarray]
///     Tuple of (state_of_charge_kwh, power_kw) arrays.
///
/// Raises
/// ------
/// ValueError
///     If inputs are invalid (mismatched array lengths, etc.)
/// RuntimeError
///     If simulation fails during execution.
#[pyfunction]
#[pyo3(name = "simulate_load_following")]
fn simulate_load_following_py<'py>(
    py: Python<'py>,
    duration_hours: PyReadonlyArray1<'py, f64>,
    solar_power_kw: PyReadonlyArray1<'py, f64>,
    load_power_kw: PyReadonlyArray1<'py, f64>,
    battery: &PyBattery,
    initial_soc_kwh: f64,
    initial_power_kw: f64,
) -> PyResult<(Bound<'py, DoublePyArray>, Bound<'py, DoublePyArray>)> {
    // 1. Validate array lengths match
    validate_array_lengths(
        duration_hours.len()?,
        solar_power_kw.len()?,
        load_power_kw.len()?,
    ).map_err(PyValueError::new_err)?;

    // 2. Get slices from numpy arrays
    let duration = duration_hours.as_slice()
        .map_err(|e| PyValueError::new_err(format!("Failed to read duration array: {}", e)))?;
    let solar = solar_power_kw.as_slice()
        .map_err(|e| PyValueError::new_err(format!("Failed to read solar_power array: {}", e)))?;
    let load = load_power_kw.as_slice()
        .map_err(|e| PyValueError::new_err(format!("Failed to read load_power array: {}", e)))?;

    // 3. Build TelemetryPoints with validation
    let telemetry = build_telemetry_points(duration, solar, load)
        .map_err(PyValueError::new_err)?;

    // 4. Build initial state
    let initial_state = build_initial_state(&battery.inner, initial_soc_kwh, initial_power_kw)
        .map_err(PyValueError::new_err)?;

    // 5. Run simulation
    let states = simulate_load_following(telemetry, battery.inner.clone(), initial_state)
        .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;

    // 6. Convert results to numpy arrays
    let (soc, power) = extract_results(&states);
    Ok((
        PyArray1::from_vec(py, soc),
        PyArray1::from_vec(py, power),
    ))
}

// ============================================================================
// Python Module
// ============================================================================

#[pymodule]
fn battery_sim(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyBattery>()?;
    m.add_function(wrap_pyfunction!(simulate_load_following_py, m)?)?;
    Ok(())
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -------------------------------------------------------------------------
    // validate_array_lengths tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_validate_array_lengths_equal() {
        assert!(validate_array_lengths(3, 3, 3).is_ok());
    }

    #[test]
    fn test_validate_array_lengths_mismatch_solar() {
        let result = validate_array_lengths(3, 2, 3);
        match result {
            Err(e) => assert!(e.contains("same length"), "expected 'same length', got: {}", e),
            Ok(_) => panic!("expected error"),
        }
    }

    #[test]
    fn test_validate_array_lengths_mismatch_load() {
        let result = validate_array_lengths(3, 3, 2);
        match result {
            Err(e) => assert!(e.contains("same length"), "expected 'same length', got: {}", e),
            Ok(_) => panic!("expected error"),
        }
    }

    #[test]
    fn test_validate_array_lengths_empty() {
        let result = validate_array_lengths(0, 0, 0);
        match result {
            Err(e) => assert!(e.contains("arrays must be at least 1 long."),
                              "expected 'arrays must be at least 1 long', got: {}", e),
            Ok(_) => panic!("expected error"),
        }
    }

    // -------------------------------------------------------------------------
    // build_telemetry_points tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_build_telemetry_valid() {
        let duration = [1.0, 0.5, 0.25];
        let solar = [10.0, 20.0, 30.0];
        let load = [5.0, 10.0, 15.0];

        let result = build_telemetry_points(&duration, &solar, &load);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 3);
    }

    #[test]
    fn test_build_telemetry_empty() {
        let result = build_telemetry_points(&[], &[], &[]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[test]
    fn test_build_telemetry_invalid_duration_negative() {
        let result = build_telemetry_points(&[-1.0, 1.0], &[10.0, 20.0], &[5.0, 10.0]);
        match result {
            Err(e) => assert!(e.contains("invalid duration"), "expected 'invalid duration', got: {}", e),
            Ok(_) => panic!("expected error"),
        }
    }

    #[test]
    fn test_build_telemetry_invalid_duration_nan() {
        let result = build_telemetry_points(&[f64::NAN, 1.0], &[10.0, 20.0], &[5.0, 10.0]);
        match result {
            Err(e) => assert!(e.contains("invalid duration"), "expected 'invalid duration', got: {}", e),
            Ok(_) => panic!("expected error"),
        }
    }

    #[test]
    fn test_build_telemetry_invalid_solar_nan() {
        let result = build_telemetry_points(&[1.0, 1.0], &[f64::NAN, 20.0], &[5.0, 10.0]);
        match result {
            Err(e) => assert!(e.contains("invalid solar_power"), "expected 'invalid solar_power', got: {}", e),
            Ok(_) => panic!("expected error"),
        }
    }

    #[test]
    fn test_build_telemetry_invalid_load_nan() {
        let result = build_telemetry_points(&[1.0, 1.0], &[10.0, 20.0], &[f64::NAN, 10.0]);
        match result {
            Err(e) => assert!(e.contains("invalid load_power"), "expected 'invalid load_power', got: {}", e),
            Ok(_) => panic!("expected error"),
        }
    }

    // -------------------------------------------------------------------------
    // build_initial_state tests
    // -------------------------------------------------------------------------

    fn make_test_battery() -> Battery {
        let capacity = Energy::from_kwh(100.0).unwrap();
        let max_power = Power::from_kw(50.0).unwrap();
        let efficiency = Efficiency::from_fraction(0.9).unwrap();
        Battery::new(capacity, max_power, efficiency).unwrap()
    }

    #[test]
    fn test_build_initial_state_valid() {
        let battery = make_test_battery();
        let result = build_initial_state(&battery, 50.0, 0.0);
        assert!(result.is_ok());

        let state = result.unwrap();
        assert!((state.state_of_charge_kwh() - 50.0).abs() < 1e-9);
        assert!((state.power_kw() - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_build_initial_state_at_capacity() {
        let battery = make_test_battery();
        assert!(build_initial_state(&battery, 100.0, 0.0).is_ok());
    }

    #[test]
    fn test_build_initial_state_at_zero() {
        let battery = make_test_battery();
        assert!(build_initial_state(&battery, 0.0, 0.0).is_ok());
    }

    #[test]
    fn test_build_initial_state_exceeds_capacity() {
        let battery = make_test_battery();
        let result = build_initial_state(&battery, 150.0, 0.0);
        assert!(result.is_err());
    }

    #[test]
    fn test_build_initial_state_negative_soc() {
        let battery = make_test_battery();
        let result = build_initial_state(&battery, -10.0, 0.0);
        assert!(result.is_err());
    }

    #[test]
    fn test_build_initial_state_nan_soc() {
        let battery = make_test_battery();
        let result = build_initial_state(&battery, f64::NAN, 0.0);
        assert!(result.is_err());
    }

    #[test]
    fn test_build_initial_state_nan_power() {
        let battery = make_test_battery();
        let result = build_initial_state(&battery, 50.0, f64::NAN);
        assert!(result.is_err());
    }

    // -------------------------------------------------------------------------
    // extract_results tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_extract_results_values() {
        let battery = make_test_battery();
        let state = build_initial_state(&battery, 50.0, 10.0).unwrap();

        let (soc, power) = extract_results(&[state, state]);
        assert_eq!(soc.len(), 1);
        assert_eq!(power.len(), 1);
        assert!((soc[0] - 50.0).abs() < 1e-9);
        assert!((power[0] - 10.0).abs() < 1e-9);
    }
}
