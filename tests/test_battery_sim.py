"""Tests for battery_sim Python bindings."""

import numpy as np
import pytest

from battery_sim import Battery, simulate_load_following


# ============================================================================
# PyBattery Tests
# ============================================================================


class TestBatteryCreation:
    """Tests for Battery construction."""

    def test_battery_creation_valid(self):
        """Test normal construction with valid parameters."""
        battery = Battery(capacity_kwh=100.0, max_power_kw=50.0, efficiency=0.9)
        assert battery.capacity_kwh == 100.0
        assert battery.max_power_kw == 50.0
        assert battery.efficiency == 0.9

    def test_battery_creation_invalid_capacity(self):
        """Test that negative/zero capacity raises ValueError."""
        with pytest.raises(ValueError):
            Battery(capacity_kwh=-100.0, max_power_kw=50.0, efficiency=0.9)

        with pytest.raises(ValueError):
            Battery(capacity_kwh=0.0, max_power_kw=50.0, efficiency=0.9)

    def test_battery_creation_invalid_max_power(self):
        """Test that negative/zero max_power raises ValueError."""
        with pytest.raises(ValueError):
            Battery(capacity_kwh=100.0, max_power_kw=-50.0, efficiency=0.9)

        with pytest.raises(ValueError):
            Battery(capacity_kwh=100.0, max_power_kw=0.0, efficiency=0.9)

    def test_battery_creation_invalid_efficiency(self):
        """Test that efficiency < 0 or > 1 raises ValueError."""
        with pytest.raises(ValueError, match="invalid efficiency"):
            Battery(capacity_kwh=100.0, max_power_kw=50.0, efficiency=-0.1)

        with pytest.raises(ValueError, match="invalid efficiency"):
            Battery(capacity_kwh=100.0, max_power_kw=50.0, efficiency=1.5)

        with pytest.raises(ValueError, match="invalid efficiency"):
            Battery(capacity_kwh=100.0, max_power_kw=50.0, efficiency=0.0)


class TestBatteryGetters:
    """Tests for Battery property getters."""

    def test_battery_getters(self):
        """Test capacity_kwh, max_power_kw, efficiency getters."""
        battery = Battery(capacity_kwh=200.0, max_power_kw=100.0, efficiency=0.85)
        assert battery.capacity_kwh == 200.0
        assert battery.max_power_kw == 100.0
        assert battery.efficiency == 0.85

    def test_battery_repr(self):
        """Test string representation."""
        battery = Battery(capacity_kwh=100.0, max_power_kw=50.0, efficiency=0.9)
        repr_str = repr(battery)
        assert "Battery" in repr_str
        assert "100.0 kWh" in repr_str
        assert "50.0 kW" in repr_str
        assert "90.0%" in repr_str


# ============================================================================
# simulate_load_following Tests
# ============================================================================


class TestSimulationBasic:
    """Basic simulation tests."""

    def test_simulation_basic(self):
        """Test simple charge/discharge scenario."""
        battery = Battery(capacity_kwh=100.0, max_power_kw=50.0, efficiency=0.9)

        # 3 time steps of 1 hour each
        duration = np.array([1.0, 1.0, 1.0])
        # Solar generates 30 kW
        solar = np.array([30.0, 30.0, 30.0])
        # Load consumes 20 kW (net +10 kW to battery)
        load = np.array([20.0, 20.0, 20.0])

        soc, power = simulate_load_following(
            duration_hours=duration,
            solar_power_kw=solar,
            load_power_kw=load,
            battery=battery,
            initial_soc_kwh=50.0,
            initial_power_kw=0.0,
        )

        assert len(soc) == 3
        assert len(power) == 3
        # Battery should be charging (positive power)
        assert all(p > 0 for p in power)

    def test_simulation_empty_arrays_returns_value_error(self):
        """Test empty input arrays produce empty output."""
        battery = Battery(capacity_kwh=100.0, max_power_kw=50.0, efficiency=0.9)

        duration = np.array([])
        solar = np.array([])
        load = np.array([])

        with pytest.raises(ValueError, match="arrays must be at least 1 long"):
            simulate_load_following(
                duration_hours=duration,
                solar_power_kw=solar,
                load_power_kw=load,
                battery=battery,
                initial_soc_kwh=50.0,
                initial_power_kw=0.0,
            )


class TestSimulationBehavior:
    """Tests for simulation charging/discharging behavior."""

    def test_simulation_charging_increases_soc(self):
        """Verify charging behavior increases state of charge."""
        battery = Battery(capacity_kwh=100.0, max_power_kw=50.0, efficiency=0.9)

        # Solar > Load means battery charges
        duration = np.array([1.0, 1.0, 1.0])
        solar = np.array([40.0, 40.0, 40.0])
        load = np.array([10.0, 10.0, 10.0])

        soc, power = simulate_load_following(
            duration_hours=duration,
            solar_power_kw=solar,
            load_power_kw=load,
            battery=battery,
            initial_soc_kwh=20.0,
            initial_power_kw=0.0,
        )

        # SoC should increase over time
        assert soc[0] > 20.0
        assert soc[1] > soc[0]
        assert soc[2] > soc[1]

        # Power should be positive (charging)
        assert all(p > 0 for p in power)

    def test_simulation_discharging_decreases_soc(self):
        """Verify discharging behavior decreases state of charge."""
        battery = Battery(capacity_kwh=100.0, max_power_kw=50.0, efficiency=0.9)

        # Load > Solar means battery discharges
        duration = np.array([1.0, 1.0, 1.0])
        solar = np.array([10.0, 10.0, 10.0])
        load = np.array([40.0, 40.0, 40.0])

        soc, power = simulate_load_following(
            duration_hours=duration,
            solar_power_kw=solar,
            load_power_kw=load,
            battery=battery,
            initial_soc_kwh=80.0,
            initial_power_kw=0.0,
        )

        # SoC should decrease over time
        assert soc[0] < 80.0
        assert soc[1] < soc[0]
        assert soc[2] < soc[1]

        # Power should be negative (discharging)
        assert all(p < 0 for p in power)

    def test_simulation_power_sign_convention(self):
        """Verify positive=charge, negative=discharge convention."""
        battery = Battery(capacity_kwh=100.0, max_power_kw=50.0, efficiency=0.9)

        # Charging scenario: solar > load
        duration = np.array([1.0])
        solar = np.array([50.0])
        load = np.array([10.0])

        _, power_charge = simulate_load_following(
            duration_hours=duration,
            solar_power_kw=solar,
            load_power_kw=load,
            battery=battery,
            initial_soc_kwh=50.0,
            initial_power_kw=0.0,
        )
        assert power_charge[0] > 0, "Charging power should be positive"

        # Discharging scenario: load > solar
        solar = np.array([10.0])
        load = np.array([50.0])

        _, power_discharge = simulate_load_following(
            duration_hours=duration,
            solar_power_kw=solar,
            load_power_kw=load,
            battery=battery,
            initial_soc_kwh=50.0,
            initial_power_kw=0.0,
        )
        assert power_discharge[0] < 0, "Discharging power should be negative"
