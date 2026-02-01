mod battery;
mod types;

use battery::{Battery, BatteryState};

fn main() {
    let battery = Battery::new(
        10.0,  // 10 kWh capacity (like a Powerwall)
        5.0,   // 5 kW max power
        0.90,   // 90% round-trip efficiency
    ).expect("OK");
    let state = battery.init_state(
        0.0,
        0.0
    ).expect("ok");


    println!("Initial SoC: {:.1}kWh", state.state_of_charge_kwh());

    let new_state_1: BatteryState = battery.charge(&state,3., 0.5).expect("ok");

    println!("New state of charge {:.2}kWh", new_state_1.state_of_charge_kwh());
    println!("Achieved charge power {:.2}kW", new_state_1.power_kw());

    let new_state_2: BatteryState = battery.step(&new_state_1,-5., 0.5).expect("ok");

    println!("State of charge {:.2}kWh", new_state_2.state_of_charge_kwh());
    println!("Achieved power: {:.2}kW", new_state_2.power_kw());

}
