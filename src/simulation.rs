use crate::battery::{BatteryState, Battery, BatteryError};
use crate::types::{TelemetryPoint};


#[derive(Debug, thiserror::Error)]
pub enum SimulationError {
    #[error("Simulating load following failed on step {1}.")]
    ErrorSimulatingLoadFollowing(#[source] BatteryError, usize)
}

pub fn simulate_load_following(
    telemetry_points: Vec<TelemetryPoint>,
    battery: Battery,
    initial_state: BatteryState,
) -> Result<Vec<BatteryState>, SimulationError> {

    let states: Vec<BatteryState> = telemetry_points.iter().enumerate().try_fold(
        vec![initial_state],
        |mut states, (i, point)| {
            let new_state = battery.load_follow_step(&states[i], point)
                .map_err(|e| SimulationError::ErrorSimulatingLoadFollowing(e, i))?;
            states.push(new_state);
            Ok(states)
        }
    )?;

    Ok(states)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{AsEfficiency, Power, Energy, Duration};
    use crate::{hour, kw, kwh};
    use approx::assert_abs_diff_eq;
    const EPSILON: f64 = 1e-9;

    fn test_battery() -> Battery {
        Battery::new(kwh!(100.0), kw!(50.0), 0.81.fraction())
            .expect("battery should be valid")
    }

    #[test]
    fn test_simulate_load_following_empty_telemetry() {
        let battery = test_battery();
        let initial_state = battery.init_state(kwh!(50.0), Power::zero())
            .expect("valid state");

        let states = simulate_load_following(vec![], battery, initial_state)
            .expect("simulation should succeed");

        assert_eq!(states.len(), 1);
        assert_abs_diff_eq!(states[0].state_of_charge().as_kwh(), 50.0, epsilon = EPSILON);
    }

    #[test]
    fn test_simulate_load_following_single_charge_step() {
        let battery = test_battery();
        let initial_state = battery.init_state(kwh!(50.0), Power::zero())
            .expect("valid state");

        // Solar 10 kW, Load 3 kW -> excess 7 kW charges battery
        let telemetry = vec![
            TelemetryPoint::new(hour!(1.0), kw!(10.0), kw!(3.0)),
        ];

        let states = simulate_load_following(telemetry, battery, initial_state)
            .expect("simulation should succeed");

        assert_eq!(states.len(), 2);
        // Charged at 7 kW for 1 hour with 90% efficiency: 50 + 7 * 1 * 0.9 = 56.3 kWh
        assert_abs_diff_eq!(states[1].state_of_charge().as_kwh(), 56.3, epsilon = EPSILON);
    }

    #[test]
    fn test_simulate_load_following_multiple_steps() {
        let battery = test_battery();
        let initial_state = battery.init_state(kwh!(50.0), Power::zero())
            .expect("valid state");

        let telemetry = vec![
            TelemetryPoint::new(hour!(1.0), kw!(10.0), kw!(3.0)),  // +7 kW charge
            TelemetryPoint::new(hour!(1.0), kw!(5.0), kw!(5.0)),   // 0 kW no change
            TelemetryPoint::new(hour!(1.0), kw!(2.0), kw!(9.0)),   // -7 kW discharge
        ];

        let states = simulate_load_following(telemetry, battery, initial_state)
            .expect("simulation should succeed");

        assert_eq!(states.len(), 4);
        // Step 1: 50 + 7 * 0.9 = 56.3 kWh
        assert_abs_diff_eq!(states[1].state_of_charge().as_kwh(), 56.3, epsilon = EPSILON);
        // Step 2: no change
        assert_abs_diff_eq!(states[2].state_of_charge().as_kwh(), 56.3, epsilon = EPSILON);
        // Step 3: 56.3 - 7 / 0.9 = 48.52... kWh
        let expected = 56.3 - (7.0 / 0.9);
        assert_abs_diff_eq!(states[3].state_of_charge().as_kwh(), expected, epsilon = EPSILON);
    }
}