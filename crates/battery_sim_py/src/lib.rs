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
// Python Function
// ============================================================================

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
) -> PyResult<(Bound<'py, PyArray1<f64>>, Bound<'py, PyArray1<f64>>)> {
    // 1. Validate array lengths match
    let n = duration_hours.len()?;
    let solar_len = solar_power_kw.len()?;
    let load_len = load_power_kw.len()?;

    if solar_len != n || load_len != n {
        return Err(PyValueError::new_err(
            "duration_hours, solar_power_kw, and load_power_kw must have the same length"
        ));
    }

    // 2. Get slices from numpy arrays
    let duration = duration_hours.as_slice()
        .map_err(|e| PyValueError::new_err(format!("Failed to read duration array: {}", e)))?;
    let solar = solar_power_kw.as_slice()
        .map_err(|e| PyValueError::new_err(format!("Failed to read solar_power array: {}", e)))?;
    let load = load_power_kw.as_slice()
        .map_err(|e| PyValueError::new_err(format!("Failed to read load_power array: {}", e)))?;

    // 3. Build TelemetryPoints with validation
    let telemetry: Vec<TelemetryPoint> = duration.iter()
        .zip(solar.iter())
        .zip(load.iter())
        .map(|((&d, &s), &l)| -> PyResult<TelemetryPoint> {
            let duration = Duration::from_hour(d)
                .map_err(|v| PyValueError::new_err(format!("invalid duration: {}", v)))?;
            let solar_power = Power::from_kw(s)
                .map_err(|v| PyValueError::new_err(format!("invalid solar_power: {}", v)))?;
            let load_power = Power::from_kw(l)
                .map_err(|v| PyValueError::new_err(format!("invalid load_power: {}", v)))?;
            Ok(TelemetryPoint::new(duration, solar_power, load_power))
        })
        .collect::<PyResult<Vec<_>>>()?;

    // 4. Build initial state (validates internally)
    let initial_soc = Energy::from_kwh(initial_soc_kwh)
        .map_err(|v| PyValueError::new_err(format!("invalid initial_soc: {}", v)))?;
    let initial_power = Power::from_kw(initial_power_kw)
        .map_err(|v| PyValueError::new_err(format!("invalid initial_power: {}", v)))?;

    let initial_state = battery.inner.init_state(initial_soc, initial_power)
        .map_err(|e| PyValueError::new_err(e.to_string()))?;

    // 5. Run simulation
    let states = simulate_load_following(telemetry, battery.inner.clone(), initial_state)
        .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;

    // 6. Convert results to numpy arrays
    let soc: Vec<f64> = states.iter().map(|s| s.state_of_charge_kwh()).collect();
    let power: Vec<f64> = states.iter().map(|s| s.power_kw()).collect();

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
