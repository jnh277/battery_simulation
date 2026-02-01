use crate::types::{AsEnergy, Energy, Power, AsPower, Duration, AsDuration, Efficiency, AsEfficiency};

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
        self.state_of_charge.to_kwh()
    }

    pub fn power_kw(&self) -> f64 {
        self.power.to_kw()
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
        capacity: Energy,
        max_power: Power,
        round_trip_efficiency: Efficiency,
    ) -> Result<Battery, BatteryError> {

        if capacity <= 0.0.kwh() {
            return Err(BatteryError::InvalidCapacity(
                "Must be greater than 0".to_string(),
            ));
        }

        if max_power <= 0.0.kw() {
            return Err(BatteryError::InvalidPower(
                "Must be greater than 0".to_string(),
            ))
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
        if state_of_charge < 0.0.kwh() {
            Err(BatteryStateError::InvalidStateOfCharge("State of charge must be greater than 0".to_string()))
        } else if state_of_charge > self.capacity {
            Err(BatteryStateError::InvalidStateOfCharge("State of charge must be less than capacity.".to_string()))
        } else if power.abs() > self.max_power{
            Err(BatteryStateError::InvalidPower("Magnitude of power must be less than max_power.".to_string()))
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

        match self.init_state(
            state_of_charge,
            actual_power,
        ){
            Ok(state) => Ok(state),
            Err(_) => Err(BatteryError::ErrorCharging("Error during charge".to_string()))
        }
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
        .max(0.0.kwh());

        match self.init_state(
            state_of_charge,
            actual_power,
        ){
            Ok(state) => Ok(state),
            Err(_) => Err(BatteryError::ErrorDischarging("Error during discharge".to_string()))
        }
    }

    pub fn step(
        &self,
        battery_state: &BatteryState,
        power: Power,
        duration: Duration,
    ) -> Result<BatteryState, BatteryError> {
        if power < 0.0.kw() {
            match self.discharge(battery_state, - power, duration) {
                Ok(state) => Ok(state),
                Err(e) => Err(e),
            }
        } else if power > 0.0.kw() {
            match self.charge(battery_state, power, duration) {
                Ok(state) => Ok(state),
                Err(e) => Err(e),
            }
        } else {
            Ok(BatteryState{
                state_of_charge: battery_state.state_of_charge,
                power: 0.0.kw()
            })
        }
    }
}
