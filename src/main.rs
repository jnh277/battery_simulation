

use battery_sim::battery::{Battery, BatteryState};
use battery_sim::types::{AsPower, AsEnergy, AsEfficiency, AsDuration, Power};
use battery_sim::control::LoadFollowing;

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


    let generation: Power = 5.0.kw();
    let consumption: Power = 3.0.kw();

    let controller: LoadFollowing = LoadFollowing::new();

    let target: Power = controller.decide(generation, consumption);

    let new_state_3: BatteryState = battery.charge(&new_state_2, target, 0.5.hour()).expect("ok");

    println!("Target power {:.2}kW", target);
    println!("State of charge {:.2}kWh", new_state_3.state_of_charge());
    println!("Achieved power {:.2}kW", new_state_3.power());


}
