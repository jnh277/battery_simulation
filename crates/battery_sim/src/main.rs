

use battery_sim::battery::{Battery};
use battery_sim::types::{AsEfficiency, Power, Energy, Duration, TelemetryPoint};
use battery_sim::{kwh, kw, hour};
use battery_sim::simulation::simulate_load_following;

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

    let telemetry_points: Vec<TelemetryPoint> = vec![
        TelemetryPoint::new(hour!(0.5), kw!(3.0), kw!(0.0)),
        TelemetryPoint::new(hour!(0.5), kw!(2.5), kw!(0.0)),
        TelemetryPoint::new(hour!(0.5), kw!(0.0), kw!(3.0)),
        TelemetryPoint::new(hour!(0.5), kw!(0.0), kw!(2.5)),
    ];

    let states = simulate_load_following(
        telemetry_points,
        battery,
        state
    ).expect("OK");

    for state in states {
        println!("battery power {:.2}", state.power());
    }
}
