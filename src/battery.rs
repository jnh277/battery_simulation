use crate::types::{AsEnergy, Energy};

pub struct BatteryState {
    state_of_charge: Energy, // the current energy that the battery has
    power_kw: f64,           // the battery power
}

impl BatteryState {
    fn new(state_of_charge: Energy, power_kw: f64) -> BatteryState {
        BatteryState {
            state_of_charge,
            power_kw,
        }
    }

    pub fn state_of_charge_kwh(&self) -> f64 {
        self.state_of_charge.to_kwh()
    }

    pub fn power_kw(&self) -> f64 {
        self.power_kw
    }
}

#[derive(Debug, Clone)]
pub struct Battery {
    capacity: Energy,           // The maximum amount of energy the battery can store
    max_power_kw: f64,          // the maximum power the battery can charge or discharge at
    round_trip_efficiency: f64, // the round trip efficiency of the battery between 0 and 1
}

#[derive(Debug)]
pub enum BatteryError {
    InvalidCapacity(String),
    InvalidPower(String),
    InvalidEfficiency(String),
    ErrorCharging(String),
    ErrorDischarging(String),
}

#[derive(Debug)]
pub enum BatteryStateError {
    InvalidStateOfCharge(String),
    InvalidPower(String)
}


impl Battery {
    pub fn new(
        capacity_kwh: f64,
        max_power_kw: f64,
        round_trip_efficiency: f64,
    ) -> Result<Battery, BatteryError> {
        let capacity: Energy = capacity_kwh.kwh();

        if capacity < 0.0.kwh() {
            return Err(BatteryError::InvalidCapacity(
                "Must be greater than 0".to_string(),
            ));
        }

        Ok(Battery {
            capacity,
            max_power_kw,
            round_trip_efficiency,
        })
    }

    pub fn init_state(
        &self,
        state_of_charge_kwh: f64,
        power_kw: f64,
    ) -> Result<BatteryState, BatteryStateError> {
        let state_of_charge: Energy = state_of_charge_kwh.kwh();
        if state_of_charge < 0.0.kwh() {
            Err(BatteryStateError::InvalidStateOfCharge("State of charge must be greater than 0".to_string()))
        } else if state_of_charge > self.capacity {
            Err(BatteryStateError::InvalidStateOfCharge("State of charge must be less than capacity.".to_string()))
        } else {
            Ok(BatteryState::new(state_of_charge, power_kw))
        }
    }
    pub fn efficiency(&self) -> f64 {
        self.round_trip_efficiency.sqrt()
    }

    pub fn max_achievable_charge_power(
        &self,
        battery_state: &BatteryState,
        duration_hours: f64,
    ) -> f64 {
        let capacity_available = self.capacity - battery_state.state_of_charge;
        let power_to_fill: f64 = capacity_available.to_kwh() / duration_hours / self.efficiency();
        self.max_power_kw.min(power_to_fill)
    }

    pub fn max_achievable_discharge_power(
        &self,
        battery_state: &BatteryState,
        duration_hours: f64,
    ) -> f64 {
        let power_to_empty: f64 =
            battery_state.state_of_charge.to_kwh() / duration_hours * self.efficiency();
        self.max_power_kw.min(power_to_empty)
    }

    pub fn charge(
        &self,
        battery_state: &BatteryState,
        power_kw: f64,
        duration_hours: f64,
    ) -> Result<BatteryState, BatteryError> {
        let actual_power: f64 =
            power_kw.min(self.max_achievable_charge_power(&battery_state, duration_hours));
        let state_of_charge_kwh: f64 = (battery_state.state_of_charge.to_kwh()
            + actual_power * duration_hours * self.efficiency())
        .min(self.capacity.to_kwh());

        match self.init_state(
            state_of_charge_kwh,
            actual_power,
        ){
            Ok(state) => Ok(state),
            Err(_) => Err(BatteryError::ErrorCharging("Error during charge".to_string()))
        }
    }

    pub fn discharge(
        &self,
        battery_state: &BatteryState,
        power_kw: f64,
        duration_hours: f64,
    ) -> Result<BatteryState, BatteryError> {
        let actual_power: f64 =
            power_kw.min(self.max_achievable_discharge_power(&battery_state, duration_hours));
        let state_of_charge_kwh: f64 = (battery_state.state_of_charge.to_kwh()
            - actual_power * duration_hours / self.efficiency())
        .max(0.0);

        match self.init_state(
            state_of_charge_kwh,
            actual_power,
        ){
            Ok(state) => Ok(state),
            Err(_) => Err(BatteryError::ErrorDischarging("Error during discharge".to_string()))
        }
    }

    pub fn step(
        &self,
        battery_state: &BatteryState,
        power_kw: f64,
        duration_hours: f64,
    ) -> Result<BatteryState, BatteryError> {
        if power_kw < 0. {
            match self.discharge(battery_state, -power_kw, duration_hours) {
                Ok(state) => Ok(state),
                Err(e) => Err(e),
            }
        } else if power_kw > 0. {
            match self.charge(battery_state, power_kw, duration_hours) {
                Ok(state) => Ok(state),
                Err(e) => Err(e),
            }
        } else {
            Ok(BatteryState{
                state_of_charge: battery_state.state_of_charge,
                power_kw: 0.0
            })
        }
    }
}
