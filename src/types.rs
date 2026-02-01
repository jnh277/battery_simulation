use std::fmt;
use std::ops::{Add, Sub, Mul, Div, Neg};

/* --------------- ENERGY ------------------- */

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Energy(f64);

impl Energy {
    pub fn from_kwh(energy_kwh: f64) -> Result<Self, f64> {
        if energy_kwh.is_infinite() || energy_kwh.is_nan() {
            Err(energy_kwh)
        } else {
            Ok(Self(energy_kwh))
        }
    }

    pub fn to_kwh(&self) -> f64 {
        self.0
    }

    pub fn min(self, other: Energy) -> Energy {
        Energy(self.0.min(other.0))
    }

    pub fn max(self, other: Energy) -> Energy {
        Energy(self.0.max(other.0))
    }
}


impl Add for Energy {
    type Output = Energy;
    fn add(self, rhs: Energy) -> Energy {
        Energy(self.0 + rhs.0)
    }
}

impl Sub for Energy {
    type Output = Energy;
    fn sub(self, rhs: Energy) -> Energy {
        Energy(self.0 - rhs.0)
    }
}

pub trait AsEnergy {
    fn mwh(self) -> Energy;

    fn kwh(self) -> Energy;

    fn wh(self) -> Energy;
}

impl AsEnergy for f64 {
    fn mwh(self) -> Energy {
        Energy::from_kwh(self * 1_000.).expect("Invalid energy value")
    }
    fn kwh(self) -> Energy {
        Energy::from_kwh(self).expect("Invalid energy value")
    }
    fn wh(self) -> Energy {
        Energy::from_kwh(self / 1_000.).expect("Invalid energy value")
    }
}

/* --------------- POWER ------------------- */

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Power(f64);

impl Power {
    pub fn from_kw(power_kw: f64) -> Result<Self, f64> {
        if power_kw.is_infinite() || power_kw.is_nan() {
            Err(power_kw)
        } else {
            Ok(Self(power_kw))
        }
    }

    pub fn to_kw(&self) -> f64 {
        self.0
    }

    pub fn abs(self) -> Power {
        Power(self.0.abs())
    }

    pub fn min(self, other: Power) -> Power {
        Power(self.0.min(other.0))
    }
}


impl Neg for Power {
    type Output = Power;

    fn neg(self) -> Power {
        Power(-self.0)
    }
}

pub trait AsPower {
    fn mw(self) -> Power;

    fn kw(self) -> Power;

    fn watt(self) -> Power;
}

impl AsPower for f64 {
    fn mw(self) -> Power {
        Power::from_kw(self * 1_000.).expect("Invalid power value")
    }
    fn kw(self) -> Power {
        Power::from_kw(self).expect("Invalid power value")
    }
    fn watt(self) -> Power {
        Power::from_kw(self / 1_000.).expect("Invalid power value")
    }
}

/* --------------- DURATION ------------------- */

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Duration(f64);

impl Duration {
    pub fn from_hour(duration_hour: f64) -> Result<Self, f64> {
        if duration_hour.is_infinite() || duration_hour.is_nan() {
            Err(duration_hour)
        } else if duration_hour <= 1e-15 { // force duration greater than 0
            Err(duration_hour)
        } else {
            Ok(Self(duration_hour))
        }
    }

    pub fn to_hour(&self) -> f64 {
        self.0
    }
}

pub trait AsDuration {
    fn hour(self) -> Duration;

    fn minute(self) -> Duration;

    fn second(self) -> Duration;
}

impl AsDuration for f64 {
    fn hour(self) -> Duration {
        Duration::from_hour(self).expect("Invalid duration value")
    }
    fn minute(self) -> Duration {
        Duration::from_hour(self / 60.).expect("Invalid duration value")
    }
    fn second(self) -> Duration {
        Duration::from_hour(self / 3600.).expect("Invalid duration value")
    }
}


#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Efficiency(f64);

impl Efficiency {
    pub fn from_fraction(fraction: f64) -> Result<Self, f64> {
        if fraction.is_infinite() || fraction.is_nan() {
            Err(fraction)
        } else if fraction < 0.0 || fraction > 1.0 { // force duration greater than 0
            Err(fraction)
        } else {
            Ok(Self(fraction))
        }
    }

    pub fn to_fraction(&self) -> f64 {
        self.0
    }

    pub fn sqrt(self) -> Efficiency {
        Efficiency(self.0.sqrt())
    }
}


pub trait AsEfficiency {
    fn fraction(self) -> Efficiency;

    fn percent(self) -> Efficiency;
}

impl AsEfficiency for f64 {
    fn fraction(self) -> Efficiency {
        Efficiency::from_fraction(self).expect("Invalid fraction value")
    }
    fn percent(self) -> Efficiency {
        Efficiency::from_fraction(self / 300.).expect("Invalid percent value")
    }
}

/* ----- Implenting display for our types ---- */

macro_rules! impl_display_with_unit {
    ($type:ty, $unit:expr) => {
        impl fmt::Display for $type {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                let precision = f.precision().unwrap_or(2);
                write!(f, "{:.prec$} {}", self.0, $unit, prec = precision)
            }
        }
    };
}

// Use the macro for each type
impl_display_with_unit!(Energy, "kWh");
impl_display_with_unit!(Power, "kW");
impl_display_with_unit!(Duration, "hours");
impl_display_with_unit!(Efficiency, "kg");


/* Type conversion */
// Power * Duration = Energy
impl Mul<Duration> for Power {
    type Output = Energy;

    fn mul(self, rhs: Duration) -> Energy {
        // Power (kW) * Duration (hours) = Energy (kWh)
        Energy::from_kwh(self.0 * rhs.0)
            .expect("Power * Duration should produce valid Energy")
    }
}

// Duration * Power = Energy (commutative)
impl Mul<Power> for Duration {
    type Output = Energy;

    fn mul(self, rhs: Power) -> Energy {
        Energy::from_kwh(self.0 * rhs.0)
            .expect("Duration * Power should produce valid Energy")
    }
}

impl Div<Duration> for Energy {
    type Output = Power;

    fn div(self, rhs: Duration) -> Power {
        Power::from_kw(self.0 / rhs.0)
            .expect("Energy / Duration should produce valid Power")
    }
}

impl Div<Efficiency> for Power {
    type Output = Power;

    fn div(self, rhs: Efficiency) -> Power {
        Power::from_kw(self.0 / rhs.0)
            .expect("Power / Efficiency should produce valid Power")
    }
}

impl Mul<Efficiency> for Power {
    type Output = Power;

    fn mul(self, rhs: Efficiency) -> Power {
        Power::from_kw(self.0 * rhs.0)
            .expect("Power * Efficiency should produce valid Power")
    }
}

impl Mul<Power> for Efficiency {
    type Output = Power;

    fn mul(self, rhs: Power) -> Power {
        Power::from_kw(self.0 * rhs.0)
            .expect("Efficiency * Power should produce valid Power")
    }
}


impl Div<Efficiency> for Energy {
    type Output = Energy;

    fn div(self, rhs: Efficiency) -> Energy {
        Energy::from_kwh(self.0 / rhs.0)
            .expect("Energy / Efficiency should produce valid Energy")
    }
}

impl Mul<Efficiency> for Energy {
    type Output = Energy;

    fn mul(self, rhs: Efficiency) -> Energy {
        Energy::from_kwh(self.0 * rhs.0)
            .expect("Energy * Efficiency should produce valid Energy")
    }
}

impl Mul<Energy> for Efficiency {
    type Output = Energy;

    fn mul(self, rhs: Energy) -> Energy {
        Energy::from_kwh(self.0 * rhs.0)
            .expect("Efficiency * Power should produce valid Power")
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use approx::{assert_abs_diff_eq};
    const EPSILON: f64 = 1e-12;

    #[test]
    fn test_energy_from_kw_accepts_finite_values() {
        let e:Energy = Energy::from_kwh(123.45).expect("finite values should be accepted");
        assert_abs_diff_eq!(e.0, 123.45, epsilon=EPSILON);

        let e:Energy = Energy::from_kwh(-10.0).expect("finite negative values are allowed for Energy");
        assert_abs_diff_eq!(e.0, -10.0, epsilon=EPSILON);

        let e:Energy = Energy::from_kwh(0.0).expect("zero should be accepted");
        assert_abs_diff_eq!(e.0, 0.0, epsilon=EPSILON);
    }

    #[test]
    fn test_energy_to_kw() {
        let e:Energy = Energy::from_kwh(123.45).expect("finite values should be accepted");
        let val:f64 = e.to_kwh();
        assert_abs_diff_eq!(val, 123.45, epsilon=EPSILON);
    }

    #[test]
    fn test_energy_from_kw_rejects_nan() {
        let err = Energy::from_kwh(f64::NAN).unwrap_err();
        assert!(err.is_nan());
    }

    #[test]
    fn test_energy_from_kw_rejects_infinity() {
        let err = Energy::from_kwh(f64::INFINITY).unwrap_err();
        assert!(err.is_infinite());
        assert!(err.is_sign_positive());

        let err = Energy::from_kwh(f64::NEG_INFINITY).unwrap_err();
        assert!(err.is_infinite());
        assert!(err.is_sign_negative());
    }

    #[test]
    fn test_energy_add_energy() {
        let e:Energy = Energy::from_kwh(4.1).expect("4.1 should be accepted");
        let e2:Energy = Energy::from_kwh(-5.1).expect("-5.1 should be accepted");
        let e3:Energy = e + e2;
        assert_abs_diff_eq!(e3.0, 4.1 - 5.1, epsilon=EPSILON);
    }

    #[test]
    fn test_energy_sub_energy() {
        let e:Energy = Energy::from_kwh(4.1).expect("4.1 should be accepted");
        let e2:Energy = Energy::from_kwh(-5.1).expect("-5.1 should be accepted");
        let e3:Energy = e - e2;
        assert_abs_diff_eq!(e3.0, 4.1 + 5.1, epsilon=EPSILON);
    }

    #[test]
    fn test_as_energy() {
        let e:Energy = 1.5.kwh();
        assert_abs_diff_eq!(e.0, 1.5, epsilon=EPSILON);

        let e:Energy = (-5.1).kwh();
        assert_abs_diff_eq!(e.0, -5.1, epsilon=EPSILON);

        let e:Energy = 4.2.mwh();
        assert_abs_diff_eq!(e.0, 4200., epsilon=EPSILON);

        let e:Energy = (-5.1).mwh();
        assert_abs_diff_eq!(e.0, -5100., epsilon=EPSILON);

        let e:Energy = (-4.2).wh();
        assert_abs_diff_eq!(e.0, -4.2e-3, epsilon=EPSILON);

        let e:Energy = 4.2.wh();
        assert_abs_diff_eq!(e.0, 4.2e-3, epsilon=EPSILON);

    }

    #[test]
    fn test_energy_comparison() {
        let e1 = Energy::from_kwh(10.0).unwrap();
        let e2 = Energy::from_kwh(20.0).unwrap();
        let e3 = Energy::from_kwh(10.0).unwrap();

        assert!(e1 < e2);
        assert!(e1 <= e2);
        assert!(e2 > e1);
        assert!(e2 >= e1);
        assert!(e1 == e3);
        assert!(e1 != e2);
    }
}
