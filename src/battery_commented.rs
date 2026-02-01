pub struct BatteryState {
    // GARRY: For correctness, this can be its own struct. Then we can make it impossible to
    // achieve a negative state of charge. We also can make a "unit" struct to represent the
    // underlying energy value here.
    //
    // Another point here is that we don't know the "maximum" capacity of the battery. This means
    // that we can construct this struct with arbitrary state of charge that could be greater than
    // what we can hold. Perhaps it is worth considering a different design to avoid this?
    //
    // Finally, both of these fields are marked as public. This means I can do:
    // `BatteryState { state_of_charge_kwh: 10000000000.0, power_kw: 100000000.0 }`, with any set
    // of numbers, disregarding what my actual limits are or any other constraints. Encapsulating
    // this behind private fields with simple getters will solve this problem, especially provided
    // we already have a public constructor.
    pub state_of_charge_kwh: f64, // the current energy that the battery has
    pub power_kw: f64,            // the battery power
}

#[derive(Debug, Clone)]
pub struct Battery {
    // GARRY: same suggestions here – units, possible negative values, field encapsulation.
    pub capacity_kwh: f64, // The maximum amount of energy the battery can store
    pub max_power_kw: f64, // the maximum power the battery can charge or discharge at
    pub round_trip_efficiency: f64, // the round trip efficiency of the battery between 0 and 1
}

impl BatteryState {
    pub fn new(state_of_charge_kwh: f64, power_kw: f64) -> BatteryState {
        BatteryState {
            state_of_charge_kwh,
            power_kw,
        }
    }
}

impl Battery {
    pub fn new(capacity_kwh: f64, max_power_kw: f64, round_trip_efficiency: f64) -> Self {
        Battery {
            capacity_kwh,
            max_power_kw,
            round_trip_efficiency,
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
        // GARRY: this can result in negative available capacity. Depending on the design we should
        // either make sure we check here and return 0 or apply the suggestion from before where we
        // guarantee that state_of_charge_kwh can not be greater than capacity.
        let capacity_available = self.capacity_kwh - battery_state.state_of_charge_kwh;

        // GARRY: This is susceptible for div-by-zero: duration_hours is arbitrary f64 and there
        // are no constraints imposed on efficiency, effectively both can be 0.
        let power_to_fill: f64 = capacity_available / duration_hours / self.efficiency();
        self.max_power_kw.min(power_to_fill)
    }

    pub fn max_achievable_discharge_power(
        &self,
        battery_state: &BatteryState,
        duration_hours: f64,
    ) -> f64 {
        let power_to_empty: f64 =
            battery_state.state_of_charge_kwh / duration_hours * self.efficiency();
        self.max_power_kw.min(power_to_empty)
    }

    pub fn charge(
        &self,
        // GARRY: Here, depending on the design and ergonomics that we want to achieve, but it
        // might be a good idea to take ownership of the `BatteryState` instead of taking a
        // reference and then return back a new `BatteryState` after computation. In this case
        // users can't mix which one is which.
        battery_state: &BatteryState,
        power_kw: f64,
        duration_hours: f64,
    ) -> BatteryState {
        let actual_power: f64 =
            // GARRY: there is no need to reference `&battery_state` again, it is already behind a
            // reference, so we can just pass in `battery_state`.
            power_kw.min(self.max_achievable_charge_power(&battery_state, duration_hours));
        // GARRY: Here we need to check for overflow. Essentially additions and multiplications can
        // result in `Infinity` and if this happens we should fail the operation. This also means
        // we need to change this function to `Result<BatteryState, Error>` or
        // `Result<BatteryState, BatteryState>` depending on what we want to achieve. In the latter
        // case we can return `Ok(new_battery_state)` and `Err(old_battery_state)` if there is a
        // recovery mechanism for the caller of this function. Otherwise we just go for the former.
        let state_of_charge_kwh: f64 = (battery_state.state_of_charge_kwh
            + actual_power * duration_hours * self.efficiency())
        .min(self.capacity_kwh);

        BatteryState {
            state_of_charge_kwh,
            power_kw: actual_power,
        }
    }

    // GARRY: very same comments that are applicable to the `charge()` method above.
    pub fn discharge(
        &self,
        battery_state: &BatteryState,
        power_kw: f64,
        duration_hours: f64,
    ) -> BatteryState {
        let actual_power: f64 =
            power_kw.min(self.max_achievable_discharge_power(&battery_state, duration_hours));
        let state_of_charge_kwh: f64 = (battery_state.state_of_charge_kwh
            - actual_power * duration_hours / self.efficiency())
        .max(0.0);

        BatteryState {
            state_of_charge_kwh,
            power_kw: -actual_power,
        }
    }

    pub fn step(
        &self,
        // GARRY: if we were to take ownership in the `charge()`/`discharge()` methods, we would
        // need to take ownership here as well, and propagate any errors from these methods.
        battery_state: &BatteryState,
        power_kw: f64,
        duration_hours: f64,
    ) -> BatteryState {
        if power_kw < 0. {
            // GARRY: no need to reference here and down below either.
            self.discharge(&battery_state, -power_kw, duration_hours)
        } else if power_kw > 0. {
            self.charge(&battery_state, power_kw, duration_hours)
        } else {
            BatteryState {
                state_of_charge_kwh: battery_state.state_of_charge_kwh,
                power_kw: 0.0,
            }
        }
    }
}
