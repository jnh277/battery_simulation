use std::fmt::{Display, Formatter};
use crate::types::{AsEnergy, Energy, Power, AsPower, Duration, Efficiency, TelemetryPoint};

pub struct BatteryState {
    state_of_charge: Energy, // the current energy that the battery has
    power: Power,           // the battery power
}

impl BatteryState {
    fn new(state_of_charge: Energy, power: Power) -> BatteryState {
        BatteryState {
            state_of_charge,
            power,
        }
    }

    pub fn state_of_charge_kwh(&self) -> f64 {
        self.state_of_charge.as_kwh()
    }

    pub fn power_kw(&self) -> f64 {
        self.power.as_kw()
    }

    pub fn power(&self) -> Power {
        self.power
    }

    pub fn state_of_charge(&self) -> Energy {
        self.state_of_charge
    }
}

#[derive(Debug, Clone)]
pub struct Battery {
    capacity: Energy,           // The maximum amount of energy the battery can store
    max_power: Power,          // the maximum power the battery can charge or discharge at
    round_trip_efficiency: Efficiency, // the round trip efficiency of the battery between 0 and 1
}

#[derive(Debug, thiserror::Error)]
pub enum BatteryError {
    #[error("Capacity must be greater than 0.")]
    NonPositiveCapacity,
    #[error("Max Power must be greater than 0.")]
    NonPositiveMaxPower,
    #[error("Error during charge.")]
    ErrorCharging(#[source]BatteryStateError),
    #[error("Error during discharge.")]
    ErrorDischarging(#[source]BatteryStateError),
}

#[derive(Debug, thiserror::Error)]
pub enum BatteryStateError {
    #[error("State of charge must be greater than 0.")]
    NegativeStateOfCharge,
    #[error("State of charge {0} must be less than Capacity {1}.")]
    StateOfChargeGreaterThanCapacity(Energy, Energy),
    #[error("Power must be less than max power.")]
    PowerGreaterThanMax
}


impl Battery {
    pub fn new(
        capacity: Energy,
        max_power: Power,
        round_trip_efficiency: Efficiency,
    ) -> Result<Battery, BatteryError> {

        if capacity.as_kwh() <= 0.0 {
            return Err(BatteryError::NonPositiveCapacity);
        }

        if max_power <= Power::zero() {
            return Err(BatteryError::NonPositiveMaxPower)
        }


        Ok(Battery {
            capacity,
            max_power,
            round_trip_efficiency,
        })
    }

    pub fn init_state(
        &self,
        state_of_charge: Energy,
        power: Power,
    ) -> Result<BatteryState, BatteryStateError> {
        if state_of_charge.as_kwh() < 0.0 {
            Err(BatteryStateError::NegativeStateOfCharge)
        } else if state_of_charge > self.capacity {
            Err(BatteryStateError::StateOfChargeGreaterThanCapacity(state_of_charge, self.capacity))
        } else if power.abs() > self.max_power{
            Err(BatteryStateError::PowerGreaterThanMax)
        } else {
            Ok(BatteryState::new(state_of_charge, power))
        }
    }
    pub fn efficiency(&self) -> Efficiency {
        self.round_trip_efficiency.sqrt()
    }

    pub fn max_achievable_charge_power(
        &self,
        battery_state: &BatteryState,
        duration: Duration,
    ) -> Power {
        let capacity_available = self.capacity - battery_state.state_of_charge;
        let power_to_fill: Power = capacity_available / duration / self.efficiency();
        self.max_power.min(power_to_fill)
    }

    pub fn max_achievable_discharge_power(
        &self,
        battery_state: &BatteryState,
        duration: Duration,
    ) -> Power {
        let power_to_empty: Power =
            battery_state.state_of_charge / duration * self.efficiency();
        self.max_power.min(power_to_empty)
    }

    pub fn charge(
        &self,
        battery_state: &BatteryState,
        power: Power,
        duration: Duration,
    ) -> Result<BatteryState, BatteryError> {
        let actual_power: Power =
            power.min(self.max_achievable_charge_power(battery_state, duration));
        let state_of_charge: Energy = (battery_state.state_of_charge
            + actual_power * duration * self.efficiency())
        .min(self.capacity);
        self.init_state(state_of_charge, actual_power).map_err(BatteryError::ErrorCharging)
    }

    pub fn discharge(
        &self,
        battery_state: &BatteryState,
        power: Power,
        duration: Duration,
    ) -> Result<BatteryState, BatteryError> {
        let actual_power: Power =
            power.min(self.max_achievable_discharge_power(battery_state, duration));

        let state_of_charge: Energy = (battery_state.state_of_charge
            - actual_power * duration / self.efficiency())
        .max(Energy::zero());

        self.init_state(state_of_charge, actual_power).map_err(BatteryError::ErrorDischarging)
    }

    pub fn step(
        &self,
        battery_state: &BatteryState,
        power: Power,
        duration: Duration,
    ) -> Result<BatteryState, BatteryError> {
        if power < Power::zero() {
            match self.discharge(battery_state, - power, duration) {
                Ok(state) => Ok(state),
                Err(e) => Err(e),
            }
        } else if power > Power::zero() {
            match self.charge(battery_state, power, duration) {
                Ok(state) => Ok(state),
                Err(e) => Err(e),
            }
        } else {
            Ok(BatteryState{
                state_of_charge: battery_state.state_of_charge,
                power: Power::zero(),
            })
        }
    }

    pub fn load_follow_step(
        &self,
        battery_state: &BatteryState,
        telemetry_point: &TelemetryPoint,
    ) -> Result<BatteryState, BatteryError> {
        let desired_power: Power = telemetry_point.excess_pv();
        self.step(battery_state, desired_power, telemetry_point.duration())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;
    use crate::{hour, kw, kwh};
    use crate::types::{AsDuration, AsEfficiency};
    const EPSILON: f64 = 1e-9;

    /* --------------- BATTERY CONSTRUCTION TESTS ------------------- */

    #[test]
    fn test_battery_new_accepts_valid_values() {
        let battery = Battery::new(
            kwh!(100.0),
            kw!(50.0),
            0.9.fraction(),
        );
        assert!(battery.is_ok());
    }

    #[test]
    fn test_battery_new_rejects_zero_capacity() {
        let battery = Battery::new(
            Energy::zero(),
            kw!(50.0),
            0.9.fraction(),
        );
        assert!(matches!(battery, Err(BatteryError::NonPositiveCapacity)));
    }

    #[test]
    fn test_battery_new_rejects_negative_capacity() {
        let battery = Battery::new(
            kwh!(-10.0),
            kw!(50.0),
            0.9.fraction(),
        );
        assert!(matches!(battery, Err(BatteryError::NonPositiveCapacity)));
    }

    #[test]
    fn test_battery_new_rejects_zero_power() {
        let battery = Battery::new(
            kwh!(100.0),
            Power::zero(),
            0.9.fraction(),
        );
        assert!(matches!(battery, Err(BatteryError::NonPositiveMaxPower)));
    }

    #[test]
    fn test_battery_new_rejects_negative_power() {
        let battery = Battery::new(
            kwh!(100.0),
            kw!(-10.0),
            0.9.fraction(),
        );
        assert!(matches!(battery, Err(BatteryError::NonPositiveMaxPower)));
    }

    /* --------------- BATTERY STATE INITIALIZATION TESTS ------------------- */

    #[test]
    fn test_init_state_accepts_valid_values() {
        let battery = Battery::new(kwh!(100.0), kw!(50.0), 0.81.fraction())
            .expect("battery should be valid");
        let state = battery.init_state(kwh!(50.0), kw!(25.0));
        assert!(state.is_ok());
    }

    #[test]
    fn test_init_state_accepts_zero_soc() {
        let battery = Battery::new(kwh!(100.0), kw!(50.0), 0.81.fraction())
            .expect("battery should be valid");
        let state = battery.init_state(Energy::zero(), Power::zero());
        assert!(state.is_ok());
    }

    #[test]
    fn test_init_state_accepts_soc_at_capacity() {
        let battery = Battery::new(kwh!(100.0), kw!(50.0), 0.81.fraction())
            .expect("battery should be valid");
        let state = battery.init_state(kwh!(100.0), Power::zero());
        assert!(state.is_ok());
    }

    #[test]
    fn test_init_state_rejects_negative_soc() {
        let battery = Battery::new(kwh!(100.0), kw!(50.0), 0.81.fraction())
            .expect("battery should be valid");
        let state = battery.init_state(kwh!(-10.0), Power::zero());
        assert!(matches!(state, Err(BatteryStateError::NegativeStateOfCharge)));
    }

    #[test]
    fn test_init_state_rejects_soc_above_capacity() {
        let battery = Battery::new(kwh!(100.0), kw!(50.0), 0.81.fraction())
            .expect("battery should be valid");
        let state = battery.init_state(kwh!(150.0), Power::zero());
        assert!(matches!(state, Err(BatteryStateError::StateOfChargeGreaterThanCapacity(_,_))));
    }

    #[test]
    fn test_init_state_rejects_power_above_max() {
        let battery = Battery::new(kwh!(100.0), kw!(50.0), 0.81.fraction())
            .expect("battery should be valid");
        let state = battery.init_state(kwh!(50.0), kw!(60.0));
        assert!(matches!(state, Err(BatteryStateError::PowerGreaterThanMax)));
    }

    #[test]
    fn test_init_state_accepts_negative_power() {
        let battery = Battery::new(kwh!(100.0), kw!(50.0), 0.81.fraction())
            .expect("battery should be valid");
        let state = battery.init_state(kwh!(50.0), kw!(-25.0));
        assert!(state.is_ok());
    }

    #[test]
    fn test_init_state_rejects_negative_power_above_max() {
        let battery = Battery::new(kwh!(100.0), kw!(50.0), 0.81.fraction())
            .expect("battery should be valid");
        let state = battery.init_state(kwh!(50.0), kw!(-60.0));
        assert!(matches!(state, Err(BatteryStateError::PowerGreaterThanMax)));
    }

    /* --------------- EFFICIENCY TESTS ------------------- */

    #[test]
    fn test_efficiency_returns_sqrt_of_round_trip() {
        let battery = Battery::new(kwh!(100.0), kw!(50.0), 0.81.fraction())
            .expect("battery should be valid");
        let one_way = battery.efficiency().as_fraction();
        assert_abs_diff_eq!(one_way, 0.9, epsilon = EPSILON);
    }

    /* --------------- MAX ACHIEVABLE POWER TESTS ------------------- */

    #[test]
    fn test_max_achievable_charge_power_limited_by_capacity() {
        let battery = Battery::new(kwh!(100.0), kw!(50.0), 0.81.fraction())
            .expect("battery should be valid");
        // Battery at 95 kWh, only 5 kWh capacity left
        // With 90% efficiency, need 5/0.9 = 5.56 kWh input to store 5 kWh
        // Over 1 hour: 5.56 kW max charge power
        let state = battery.init_state(kwh!(95.0), Power::zero()).expect("valid state");
        let max_power = battery.max_achievable_charge_power(&state, hour!(1.0));
        let expected = (100.0 - 95.0) / 1.0 / 0.9; // ~5.56 kW
        assert_abs_diff_eq!(max_power.as_kw(), expected, epsilon = EPSILON);
    }

    #[test]
    fn test_max_achievable_charge_power_limited_by_max_power() {
        let battery = Battery::new(kwh!(100.0), kw!(50.0), 0.81.fraction())
            .expect("battery should be valid");
        // Empty battery, plenty of capacity - limited by max_power (50 kW)
        let state = battery.init_state(Energy::zero(), Power::zero()).expect("valid state");
        let max_power = battery.max_achievable_charge_power(&state, hour!(1.0));
        assert_abs_diff_eq!(max_power.as_kw(), 50.0, epsilon = EPSILON);
    }

    #[test]
    fn test_max_achievable_discharge_power_limited_by_soc() {
        let battery = Battery::new(kwh!(100.0), kw!(50.0), 0.81.fraction())
            .expect("battery should be valid");
        // Battery at 5 kWh, can only discharge that much
        // With 90% efficiency, output = 5 * 0.9 = 4.5 kWh over 1 hour = 4.5 kW
        let state = battery.init_state(kwh!(5.0), Power::zero()).expect("valid state");
        let max_power = battery.max_achievable_discharge_power(&state, hour!(1.0));
        let expected = 5.0 / 1.0 * 0.9; // 4.5 kW
        assert_abs_diff_eq!(max_power.as_kw(), expected, epsilon = EPSILON);
    }

    #[test]
    fn test_max_achievable_discharge_power_limited_by_max_power() {
        let battery = Battery::new(kwh!(100.0), kw!(50.0), 0.81.fraction())
            .expect("battery should be valid");
        // Full battery, plenty of energy - limited by max_power (50 kW)
        let state = battery.init_state(kwh!(100.0), Power::zero()).expect("valid state");
        let max_power = battery.max_achievable_discharge_power(&state, hour!(1.0));
        assert_abs_diff_eq!(max_power.as_kw(), 50.0, epsilon = EPSILON);
    }

    /* --------------- CHARGE TESTS ------------------- */

    #[test]
    fn test_charge_normal_operation() {
        let battery = Battery::new(kwh!(100.0), kw!(50.0), 0.81.fraction())
            .expect("battery should be valid");
        let state = battery.init_state(Energy::zero(), Power::zero()).expect("valid state");
        // Charge at 10 kW for 1 hour with 90% efficiency
        // Energy stored = 10 * 1 * 0.9 = 9 kWh
        let new_state = battery.charge(&state, kw!(10.0), hour!(1.0)).expect("charge should succeed");
        assert_abs_diff_eq!(new_state.state_of_charge().as_kwh(), 9.0, epsilon = EPSILON);
        assert_abs_diff_eq!(new_state.power().as_kw(), 10.0, epsilon = EPSILON);
    }

    #[test]
    fn test_charge_clamps_to_max_power() {
        let battery = Battery::new(kwh!(100.0), kw!(50.0), 0.81.fraction())
            .expect("battery should be valid");
        let state = battery.init_state(Energy::zero(), Power::zero()).expect("valid state");
        // Request 100 kW but max is 50 kW
        let new_state = battery.charge(&state, kw!(100.0), hour!(1.0)).expect("charge should succeed");
        assert_abs_diff_eq!(new_state.power().as_kw(), 50.0, epsilon = EPSILON);
    }

    #[test]
    fn test_charge_clamps_to_capacity() {
        let battery = Battery::new(kwh!(100.0), kw!(50.0), 0.81.fraction())
            .expect("battery should be valid");
        let state = battery.init_state(kwh!(90.0), Power::zero()).expect("valid state");
        // Try to charge 50 kW for 1 hour (would add 45 kWh), but only 10 kWh capacity left
        let new_state = battery.charge(&state, kw!(50.0), hour!(1.0)).expect("charge should succeed");
        assert_abs_diff_eq!(new_state.state_of_charge().as_kwh(), 100.0, epsilon = EPSILON);
    }

    #[test]
    fn test_charge_accounts_for_efficiency() {
        let battery = Battery::new(kwh!(100.0), kw!(50.0), 0.81.fraction())
            .expect("battery should be valid");
        let state = battery.init_state(Energy::zero(), Power::zero()).expect("valid state");
        // Charge at 20 kW for 2 hours with 90% efficiency
        // Energy stored = 20 * 2 * 0.9 = 36 kWh
        let new_state = battery.charge(&state, kw!(20.0), hour!(2.0)).expect("charge should succeed");
        assert_abs_diff_eq!(new_state.state_of_charge().as_kwh(), 36.0, epsilon = EPSILON);
    }

    /* --------------- DISCHARGE TESTS ------------------- */

    #[test]
    fn test_discharge_normal_operation() {
        let battery = Battery::new(kwh!(100.0), kw!(50.0), 0.81.fraction())
            .expect("battery should be valid");
        let state = battery.init_state(kwh!(50.0), Power::zero()).expect("valid state");
        // Discharge at 10 kW for 1 hour with 90% efficiency
        // Energy removed from battery = 10 / 0.9 = 11.11 kWh
        let new_state = battery.discharge(&state, kw!(10.0), hour!(1.0)).expect("discharge should succeed");
        let expected_soc = 50.0 - (10.0 / 0.9);
        assert_abs_diff_eq!(new_state.state_of_charge().as_kwh(), expected_soc, epsilon = EPSILON);
        assert_abs_diff_eq!(new_state.power().as_kw(), 10.0, epsilon = EPSILON);
    }

    #[test]
    fn test_discharge_clamps_to_max_power() {
        let battery = Battery::new(kwh!(100.0), kw!(50.0), 0.81.fraction())
            .expect("battery should be valid");
        let state = battery.init_state(kwh!(100.0), Power::zero()).expect("valid state");
        // Request 100 kW but max is 50 kW
        let new_state = battery.discharge(&state, kw!(100.0), hour!(1.0)).expect("discharge should succeed");
        assert_abs_diff_eq!(new_state.power().as_kw(), 50.0, epsilon = EPSILON);
    }

    #[test]
    fn test_discharge_clamps_to_zero_soc() {
        let battery = Battery::new(kwh!(100.0), kw!(50.0), 0.81.fraction())
            .expect("battery should be valid");
        let state = battery.init_state(kwh!(5.0), Power::zero()).expect("valid state");
        // Try to discharge 50 kW for 1 hour, but only 5 kWh available
        let new_state = battery.discharge(&state, kw!(50.0), hour!(1.0)).expect("discharge should succeed");
        assert!(new_state.state_of_charge().as_kwh() >= 0.0);
    }

    #[test]
    fn test_discharge_accounts_for_efficiency() {
        let battery = Battery::new(kwh!(100.0), kw!(50.0), 0.81.fraction())
            .expect("battery should be valid");
        let state = battery.init_state(kwh!(100.0), Power::zero()).expect("valid state");
        // Discharge at 18 kW for 2 hours with 90% efficiency
        // Energy removed from battery = 18 * 2 / 0.9 = 40 kWh
        let new_state = battery.discharge(&state, kw!(18.0), hour!(2.0)).expect("discharge should succeed");
        let expected_soc = 100.0 - (18.0 * 2.0 / 0.9);
        assert_abs_diff_eq!(new_state.state_of_charge().as_kwh(), expected_soc, epsilon = EPSILON);
    }

    /* --------------- STEP TESTS ------------------- */

    #[test]
    fn test_step_positive_power_charges() {
        let battery = Battery::new(kwh!(100.0), kw!(50.0), 0.81.fraction())
            .expect("battery should be valid");
        let state = battery.init_state(kwh!(50.0), Power::zero()).expect("valid state");
        let new_state = battery.step(&state, kw!(10.0), hour!(1.0)).expect("step should succeed");
        // Positive power should charge: 50 + 10 * 1 * 0.9 = 59 kWh
        assert_abs_diff_eq!(new_state.state_of_charge().as_kwh(), 59.0, epsilon = EPSILON);
    }

    #[test]
    fn test_step_negative_power_discharges() {
        let battery = Battery::new(kwh!(100.0), kw!(50.0), 0.81.fraction())
            .expect("battery should be valid");
        let state = battery.init_state(kwh!(50.0), Power::zero()).expect("valid state");
        let new_state = battery.step(&state, kw!(-10.0), hour!(1.0)).expect("step should succeed");
        // Negative power should discharge: 50 - 10 / 0.9 = 38.89 kWh
        let expected_soc = 50.0 - (10.0 / 0.9);
        assert_abs_diff_eq!(new_state.state_of_charge().as_kwh(), expected_soc, epsilon = EPSILON);
    }

    #[test]
    fn test_step_zero_power_maintains_soc() {
        let battery = Battery::new(kwh!(100.0), kw!(50.0), 0.81.fraction())
            .expect("battery should be valid");
        let state = battery.init_state(kwh!(50.0), kw!(10.0)).expect("valid state");
        let new_state = battery.step(&state, Power::zero(), hour!(1.0)).expect("step should succeed");
        assert_abs_diff_eq!(new_state.state_of_charge().as_kwh(), 50.0, epsilon = EPSILON);
        assert_abs_diff_eq!(new_state.power().as_kw(), 0.0, epsilon = EPSILON);
    }

    /* --------------- INTEGRATION / ROUND-TRIP TESTS ------------------- */

    #[test]
    fn test_round_trip_efficiency() {
        let battery = Battery::new(kwh!(100.0), kw!(50.0), 0.81.fraction())
            .expect("battery should be valid");
        let state = battery.init_state(kwh!(50.0), Power::zero()).expect("valid state");

        // Charge 10 kW for 1 hour: adds 10 * 0.9 = 9 kWh
        let after_charge = battery.charge(&state, kw!(10.0), hour!(1.0)).expect("charge should succeed");
        assert_abs_diff_eq!(after_charge.state_of_charge().as_kwh(), 59.0, epsilon = EPSILON);

        // Discharge 9 kW for 1 hour: removes 9 / 0.9 = 10 kWh from battery
        // But we only deliver 9 kWh to the grid
        let after_discharge = battery.discharge(&after_charge, kw!(9.0), hour!(1.0)).expect("discharge should succeed");
        let expected_soc = 59.0 - (9.0 / 0.9); // 59 - 10 = 49 kWh
        assert_abs_diff_eq!(after_discharge.state_of_charge().as_kwh(), expected_soc, epsilon = EPSILON);

        // Net effect: put in 10 kWh (at grid), got out 9 kWh (at grid)
        // But battery SOC went 50 -> 59 -> 49 (net -1 kWh in battery)
        // Round-trip: 9 kWh out / 10 kWh in = 0.9 * 0.9 = 0.81 = 81%
    }

    #[test]
    fn test_multiple_charge_cycles() {
        let battery = Battery::new(kwh!(100.0), kw!(50.0), 0.81.fraction())
            .expect("battery should be valid");
        let state = battery.init_state(Energy::zero(), Power::zero()).expect("valid state");

        // Charge 10 kW for 1 hour three times
        let state1 = battery.charge(&state, kw!(10.0), hour!(1.0)).expect("charge 1");
        let state2 = battery.charge(&state1, kw!(10.0), hour!(1.0)).expect("charge 2");
        let state3 = battery.charge(&state2, kw!(10.0), hour!(1.0)).expect("charge 3");

        // Each charge adds 10 * 0.9 = 9 kWh, total = 27 kWh
        assert_abs_diff_eq!(state3.state_of_charge().as_kwh(), 27.0, epsilon = EPSILON);
    }

    #[test]
    fn test_multiple_discharge_cycles() {
        let battery = Battery::new(kwh!(100.0), kw!(50.0), 0.81.fraction())
            .expect("battery should be valid");
        let state = battery.init_state(kwh!(100.0), Power::zero()).expect("valid state");

        // Discharge 9 kW for 1 hour three times
        let state1 = battery.discharge(&state, kw!(9.0), hour!(1.0)).expect("discharge 1");
        let state2 = battery.discharge(&state1, kw!(9.0), hour!(1.0)).expect("discharge 2");
        let state3 = battery.discharge(&state2, kw!(9.0), hour!(1.0)).expect("discharge 3");

        // Each discharge removes 9 / 0.9 = 10 kWh, total removed = 30 kWh
        assert_abs_diff_eq!(state3.state_of_charge().as_kwh(), 70.0, epsilon = EPSILON);
    }

    #[test]
    fn test_charge_discharge_sequence() {
        let battery = Battery::new(kwh!(100.0), kw!(50.0), 0.81.fraction())
            .expect("battery should be valid");
        let state = battery.init_state(kwh!(50.0), Power::zero()).expect("valid state");

        // Simulate a day: charge in morning, discharge in evening
        let after_charge = battery.step(&state, kw!(30.0), hour!(2.0)).expect("morning charge");
        // Added 30 * 2 * 0.9 = 54 kWh, SOC = 50 + 54 = 104, clamped to 100
        assert_abs_diff_eq!(after_charge.state_of_charge().as_kwh(), 100.0, epsilon = EPSILON);

        let after_discharge = battery.step(&after_charge, kw!(-40.0), hour!(1.0)).expect("evening discharge");
        // Removed 40 / 0.9 = 44.44 kWh, SOC = 100 - 44.44 = 55.56
        let expected = 100.0 - (40.0 / 0.9);
        assert_abs_diff_eq!(after_discharge.state_of_charge().as_kwh(), expected, epsilon = EPSILON);
    }

    /* --------------- ACCESSOR TESTS ------------------- */

    #[test]
    fn test_battery_state_accessors() {
        let battery = Battery::new(kwh!(100.0), kw!(50.0), 0.81.fraction())
            .expect("battery should be valid");
        let state = battery.init_state(kwh!(75.0), kw!(25.0)).expect("valid state");

        assert_abs_diff_eq!(state.state_of_charge().as_kwh(), 75.0, epsilon = EPSILON);
        assert_abs_diff_eq!(state.state_of_charge_kwh(), 75.0, epsilon = EPSILON);
        assert_abs_diff_eq!(state.power().as_kw(), 25.0, epsilon = EPSILON);
        assert_abs_diff_eq!(state.power_kw(), 25.0, epsilon = EPSILON);
    }
}
