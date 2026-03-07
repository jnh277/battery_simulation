"""
Example script demonstrating the battery_sim Python bindings.

Run with: uv run python scripts/example.py
"""

import numpy as np
import pandas as pd
from battery_sim import Battery, simulate_load_following


def main():
    # -------------------------------------------------------------------------
    # Example 1: Basic usage with numpy arrays
    # -------------------------------------------------------------------------
    print("=" * 60)
    print("Example 1: Basic simulation with numpy arrays")
    print("=" * 60)

    # Telemetry data: 5 time steps of 1 hour each
    duration_hours = np.array([1.0, 1.0, 1.0, 1.0, 1.0])
    solar_power_kw = np.array([0.0, 5.0, 10.0, 8.0, 2.0])  # Solar generation
    load_power_kw = np.array([3.0, 3.0, 4.0, 6.0, 5.0])    # Load consumption

    # Create battery object
    battery = Battery(capacity_kwh=20.0, max_power_kw=10.0, efficiency=0.90)
    print(f"\nCreated: {battery}")
    print(f"  capacity_kwh: {battery.capacity_kwh}")
    print(f"  max_power_kw: {battery.max_power_kw}")
    print(f"  efficiency: {battery.efficiency}")

    # Initial state
    initial_soc_kwh = 10.0   # Start at 50% state of charge
    initial_power_kw = 0.0

    # Run simulation
    soc, power = simulate_load_following(
        duration_hours=duration_hours,
        solar_power_kw=solar_power_kw,
        load_power_kw=load_power_kw,
        battery=battery,
        initial_soc_kwh=initial_soc_kwh,
        initial_power_kw=initial_power_kw,
    )

    # Display results
    print(f"\nSimulation results:")
    print(f"Initial SoC: {initial_soc_kwh} kWh\n")

    print("Step | Solar | Load | Excess | Battery Power | SoC")
    print("-" * 55)
    for i in range(len(duration_hours)):
        excess = solar_power_kw[i] - load_power_kw[i]
        print(f"  {i}  | {solar_power_kw[i]:5.1f} | {load_power_kw[i]:4.1f} | {excess:6.1f} | {power[i+1]:13.2f} | {soc[i+1]:.2f}")

    # -------------------------------------------------------------------------
    # Example 2: Error handling
    # -------------------------------------------------------------------------
    print("\n" + "=" * 60)
    print("Example 2: Error handling")
    print("=" * 60)

    # Invalid capacity (negative)
    try:
        Battery(capacity_kwh=-10.0, max_power_kw=5.0, efficiency=0.9)  # Invalid!
    except ValueError as e:
        print(f"\nCaught ValueError for negative capacity: {e}")

    # Invalid efficiency (> 1)
    try:
        Battery(capacity_kwh=10.0, max_power_kw=5.0, efficiency=1.5)  # Invalid!
    except ValueError as e:
        print(f"Caught ValueError for invalid efficiency: {e}")

    # Mismatched array lengths
    try:
        valid_battery = Battery(capacity_kwh=10.0, max_power_kw=5.0, efficiency=0.9)
        simulate_load_following(
            duration_hours=np.array([1.0, 1.0]),
            solar_power_kw=np.array([5.0]),  # Wrong length!
            load_power_kw=np.array([3.0, 3.0]),
            battery=valid_battery,
            initial_soc_kwh=5.0,
            initial_power_kw=0.0,
        )
    except ValueError as e:
        print(f"Caught ValueError for mismatched arrays: {e}")

    print("\n" + "=" * 60)
    print("All examples completed successfully!")
    print("=" * 60)


if __name__ == "__main__":
    main()
