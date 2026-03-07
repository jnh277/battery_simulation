# Battery Simulation in Rust

A battery load-following simulator written in Rust with Python bindings.

## Project Structure

```
battery_sim/
├── Cargo.toml              # Workspace definition
├── Cargo.lock              # Shared dependency lock
├── pyproject.toml          # Python package config (maturin + uv)
├── .python-version         # Python version pin for uv
│
└── crates/
    ├── battery_sim/        # Pure Rust library
    │   ├── Cargo.toml
    │   ├── data/           # Test data
    │   └── src/
    │       ├── lib.rs      # Library root
    │       ├── main.rs     # Binary entry point
    │       ├── battery.rs  # Battery model
    │       ├── simulation.rs
    │       ├── types.rs    # Energy, Power, Duration types
    │       └── data.rs     # CSV parsing
    │
    └── battery_sim_py/     # Python bindings (PyO3)
        ├── Cargo.toml
        └── src/
            └── lib.rs      # Python module definition
```

This is a [Cargo workspace](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html) with two crates:

- **`crates/battery_sim`** - Pure Rust library and binary. Has no Python dependencies and can be used standalone.
- **`crates/battery_sim_py`** - Python bindings using PyO3. Depends on `battery_sim` and exposes it to Python.

The workspace approach keeps the Rust library clean while allowing Python bindings to live in the same repository. Both crates share a single `target/` directory for faster builds.

## Rust Usage

### Run the binary

```bash
cargo run -p battery_sim
```

### Run tests

```bash
cargo test -p battery_sim
```

### Use as a library

```rust
use battery_sim::battery::Battery;
use battery_sim::simulation::simulate_load_following;
use battery_sim::types::{TelemetryPoint, AsEfficiency};
use battery_sim::{kwh, kw, hour};

// Create a battery
let battery = Battery::new(
    kwh!(100.0),      // capacity
    kw!(50.0),        // max power
    0.81.fraction(),  // round-trip efficiency
).expect("valid battery");

// Set initial state
let initial_state = battery.init_state(kwh!(50.0), Power::zero())
    .expect("valid state");

// Create telemetry points
let telemetry = vec![
    TelemetryPoint::new(hour!(1.0), kw!(10.0), kw!(3.0)),  // solar, load
    TelemetryPoint::new(hour!(1.0), kw!(5.0), kw!(8.0)),
];

// Run simulation
let states = simulate_load_following(telemetry, battery, initial_state)
    .expect("simulation succeeds");

for state in states {
    println!("SoC: {:.2} kWh, Power: {:.2} kW",
        state.state_of_charge_kwh(),
        state.power_kw());
}
```

## Python Usage

### Setup

Requires [UV](https://docs.astral.sh/uv/) for package management.

```bash
# Install dependencies and build the extension
uv sync

# Or with dev dependencies (pandas, pytest)
uv sync --extra dev
```

### Run Python code

```bash
uv run python scripts/example.py
```

### Example

```python
import numpy as np
import pandas as pd
from battery_sim import Battery, simulate_load_following

# Create battery object
battery = Battery(capacity_kwh=100.0, max_power_kw=50.0, efficiency=0.81)
print(battery)  # Battery(capacity=100.0 kWh, max_power=50.0 kW, efficiency=81.0%)

# Access properties
print(battery.capacity_kwh)  # 100.0
print(battery.max_power_kw)  # 50.0
print(battery.efficiency)    # 0.81

# Create telemetry data
df = pd.DataFrame({
    'duration': [1.0, 1.0, 1.0],
    'solar_power': [10.0, 5.0, 2.0],
    'load_power': [3.0, 5.0, 8.0],
})

# Run simulation
soc, power = simulate_load_following(
    duration_hours=df['duration'].values,
    solar_power_kw=df['solar_power'].values,
    load_power_kw=df['load_power'].values,
    battery=battery,
    initial_soc_kwh=50.0,
    initial_power_kw=0.0,
)

# Results are numpy arrays
results = pd.DataFrame({
    'state_of_charge_kwh': soc,
    'power_kw': power,
})
print(results)
```

### Error Handling

The `Battery` constructor and `simulate_load_following` raise `ValueError` for invalid inputs and `RuntimeError` for simulation failures:

```python
# Invalid battery parameters raise ValueError
try:
    battery = Battery(capacity_kwh=-100.0, max_power_kw=50.0, efficiency=0.81)  # Invalid!
except ValueError as e:
    print(f"Invalid input: {e}")

# Invalid efficiency
try:
    battery = Battery(capacity_kwh=100.0, max_power_kw=50.0, efficiency=1.5)  # Invalid!
except ValueError as e:
    print(f"Invalid input: {e}")
```

## Development

### Rebuild after Rust changes

```bash
uv sync
# or
uv run maturin develop
```

### Run Python tests

```bash
uv run pytest
```
