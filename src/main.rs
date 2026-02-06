

use battery_sim::battery::{Battery, BatteryState};
use battery_sim::types::{AsEfficiency, Power, Energy, Duration, TelemetryPoint};
use battery_sim::{kwh, kw, hour};

fn main() {
    let battery = Battery::new(
        kwh!(10.0),
        kw!(5.0),   // 5 kW max power
        0.90.fraction(),   // 90% round-trip efficiency
    ).expect("OK");
    let state = battery.init_state(
        Energy::zero(),
        Power::zero()
    ).expect("ok");


    println!("Initial SoC: {:.1}", state.state_of_charge());

    let new_state_1: BatteryState = battery.charge(&state, kw!(3.0), hour!(0.5)).expect("ok");

    println!("New state of charge {:.2}kWh", new_state_1.state_of_charge());
    println!("Achieved charge power {:.2}kW", new_state_1.power());

    let new_state_2: BatteryState = battery.step(&new_state_1, kw!(-5.0), hour!(0.5)).expect("ok");

    println!("State of charge {:.2}kWh", new_state_2.state_of_charge());
    println!("Achieved power: {:.2}kW", new_state_2.power());


    let generation: Power = kw!(5.0);
    let consumption: Power = kw!(3.0);

    let telemetry_point: TelemetryPoint = TelemetryPoint::new(
        hour!(0.5),
        generation,
        consumption
    );

    // let new_state_3: BatteryState = battery.charge(&new_state_2, target, hour!(0.5)).expect("ok");
    let new_state_3: BatteryState = battery.load_follow_step(
        &new_state_2,
        &telemetry_point,
    ).expect("Ok");

    println!("Excess gen {:.2}kW", telemetry_point.excess_pv());
    println!("State of charge {:.2}kWh", new_state_3.state_of_charge());
    println!("Achieved power {:.2}kW", new_state_3.power());


}
