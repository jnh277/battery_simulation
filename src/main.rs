mod battery;
mod types;

use battery::{Battery, BatteryState};
use types::{AsPower, AsEnergy, AsEfficiency, AsDuration};

fn main() {
    let battery = Battery::new(
        10.0.kwh(),  // 10 kWh capacity (like a Powerwall)
        5.0.kw(),   // 5 kW max power
        0.90.fraction(),   // 90% round-trip efficiency
    ).expect("OK");
    let state = battery.init_state(
        0.0.kwh(),
        0.0.kw()
    ).expect("ok");


    println!("Initial SoC: {:.1}", state.state_of_charge());

    let new_state_1: BatteryState = battery.charge(&state,3.0.kw(), 0.5.hour()).expect("ok");

    println!("New state of charge {:.2}kWh", new_state_1.state_of_charge());
    println!("Achieved charge power {:.2}kW", new_state_1.power());

    let new_state_2: BatteryState = battery.step(&new_state_1,-5.0.kw(), 0.5.hour()).expect("ok");

    println!("State of charge {:.2}kWh", new_state_2.state_of_charge());
    println!("Achieved power: {:.2}kW", new_state_2.power());

}
