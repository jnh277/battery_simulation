use std::fmt;
use std::ops::{Add, Sub, Mul, Div, Neg};

const MIN_VALUE: f64 = 1e-10;
const MAX_VALUE: f64 = 1e6;  // this will be equivalent to 1 GIGA

/* --------------- ENERGY ------------------- */

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Energy(f64);

impl Energy {
    pub fn from_kwh(energy_kwh: f64) -> Result<Self, f64> {
        if energy_kwh.is_infinite() || energy_kwh.is_nan() || energy_kwh > MAX_VALUE  {
            Err(energy_kwh)
        } else {
            Ok(Self(energy_kwh))
        }
    }

    pub fn as_kwh(&self) -> f64 {
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
        if power_kw.is_infinite() || power_kw.is_nan() || power_kw > MAX_VALUE {
            Err(power_kw)
        } else {
            Ok(Self(power_kw))
        }
    }

    pub fn as_kw(&self) -> f64 {
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
        if duration_hour.is_infinite() || duration_hour.is_nan() || !(MIN_VALUE..=MAX_VALUE).contains(&duration_hour){
            Err(duration_hour)
        } else {
            Ok(Self(duration_hour))
        }
    }

    pub fn as_hour(&self) -> f64 {
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
        } else if fraction <= MIN_VALUE || fraction > 1.0 { // force efficiency in range (0, 1]
            Err(fraction)
        } else {
            Ok(Self(fraction))
        }
    }

    pub fn as_fraction(&self) -> f64 {
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
        Efficiency::from_fraction(self / 100.).expect("Invalid percent value")
    }
}

/* ----- Implementing display for our types ---- */

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
impl_display_with_unit!(Efficiency, "%");


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
        let val:f64 = e.as_kwh();
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
        let e1 = Energy::from_kwh(10.0).expect("10.0 should be valid");
        let e2 = Energy::from_kwh(20.0).expect("20.0 should be valid");
        let e3 = Energy::from_kwh(10.0).expect("10.0 should be valid");

        assert!(e1 < e2);
        assert!(e1 <= e2);
        assert!(e2 > e1);
        assert!(e2 >= e1);
        assert!(e1 == e3);
        assert!(e1 != e2);
    }

    #[test]
    fn test_energy_min() {
        let e1 = Energy::from_kwh(10.0).expect("10.0 should be valid");
        let e2 = Energy::from_kwh(20.0).expect("20.0 should be valid");
        assert_eq!(e1.min(e2), e1);
        assert_eq!(e2.min(e1), e1);
    }

    #[test]
    fn test_energy_max() {
        let e1 = Energy::from_kwh(10.0).expect("10.0 should be valid");
        let e2 = Energy::from_kwh(20.0).expect("20.0 should be valid");
        assert_eq!(e1.max(e2), e2);
        assert_eq!(e2.max(e1), e2);
    }

    #[test]
    fn test_energy_rejects_max_value() {
        let err = Energy::from_kwh(MAX_VALUE + 1.0).unwrap_err();
        assert!(err > MAX_VALUE);
    }

    /* --------------- POWER TESTS ------------------- */

    #[test]
    fn test_power_from_kw_accepts_finite_values() {
        let p = Power::from_kw(123.45).expect("finite values should be accepted");
        assert_abs_diff_eq!(p.0, 123.45, epsilon = EPSILON);

        let p = Power::from_kw(-10.0).expect("finite negative values are allowed for Power");
        assert_abs_diff_eq!(p.0, -10.0, epsilon = EPSILON);

        let p = Power::from_kw(0.0).expect("zero should be accepted");
        assert_abs_diff_eq!(p.0, 0.0, epsilon = EPSILON);
    }

    #[test]
    fn test_power_to_kw() {
        let p = Power::from_kw(123.45).expect("finite values should be accepted");
        let val = p.as_kw();
        assert_abs_diff_eq!(val, 123.45, epsilon = EPSILON);
    }

    #[test]
    fn test_power_from_kw_rejects_nan() {
        let err = Power::from_kw(f64::NAN).unwrap_err();
        assert!(err.is_nan());
    }

    #[test]
    fn test_power_from_kw_rejects_infinity() {
        let err = Power::from_kw(f64::INFINITY).unwrap_err();
        assert!(err.is_infinite());
        assert!(err.is_sign_positive());

        let err = Power::from_kw(f64::NEG_INFINITY).unwrap_err();
        assert!(err.is_infinite());
        assert!(err.is_sign_negative());
    }

    #[test]
    fn test_power_rejects_max_value() {
        let err = Power::from_kw(MAX_VALUE + 1.0).unwrap_err();
        assert!(err > MAX_VALUE);
    }

    #[test]
    fn test_power_abs() {
        let p = Power::from_kw(-50.0).expect("-50.0 should be valid");
        assert_abs_diff_eq!(p.abs().as_kw(), 50.0, epsilon = EPSILON);

        let p = Power::from_kw(50.0).expect("50.0 should be valid");
        assert_abs_diff_eq!(p.abs().as_kw(), 50.0, epsilon = EPSILON);
    }

    #[test]
    fn test_power_min() {
        let p1 = Power::from_kw(10.0).expect("10.0 should be valid");
        let p2 = Power::from_kw(20.0).expect("20.0 should be valid");
        assert_eq!(p1.min(p2), p1);
        assert_eq!(p2.min(p1), p1);
    }

    #[test]
    fn test_power_neg() {
        let p = Power::from_kw(50.0).expect("50.0 should be valid");
        let neg_p = -p;
        assert_abs_diff_eq!(neg_p.as_kw(), -50.0, epsilon = EPSILON);

        let p = Power::from_kw(-50.0).expect("-50.0 should be valid");
        let neg_p = -p;
        assert_abs_diff_eq!(neg_p.as_kw(), 50.0, epsilon = EPSILON);
    }

    #[test]
    fn test_as_power() {
        let p = 1.5.kw();
        assert_abs_diff_eq!(p.0, 1.5, epsilon = EPSILON);

        let p = 4.2.mw();
        assert_abs_diff_eq!(p.0, 4200., epsilon = EPSILON);

        let p = 4200.0.watt();
        assert_abs_diff_eq!(p.0, 4.2, epsilon = EPSILON);
    }

    #[test]
    fn test_power_comparison() {
        let p1 = Power::from_kw(10.0).expect("10.0 should be valid");
        let p2 = Power::from_kw(20.0).expect("20.0 should be valid");
        let p3 = Power::from_kw(10.0).expect("10.0 should be valid");

        assert!(p1 < p2);
        assert!(p1 <= p2);
        assert!(p2 > p1);
        assert!(p2 >= p1);
        assert!(p1 == p3);
        assert!(p1 != p2);
    }

    /* --------------- DURATION TESTS ------------------- */

    #[test]
    fn test_duration_from_hour_accepts_valid_values() {
        let d = Duration::from_hour(1.5).expect("valid values should be accepted");
        assert_abs_diff_eq!(d.0, 1.5, epsilon = EPSILON);

        let d = Duration::from_hour(0.001).expect("small positive values should be accepted");
        assert_abs_diff_eq!(d.0, 0.001, epsilon = EPSILON);
    }

    #[test]
    fn test_duration_to_hour() {
        let d = Duration::from_hour(2.5).expect("finite values should be accepted");
        let val = d.as_hour();
        assert_abs_diff_eq!(val, 2.5, epsilon = EPSILON);
    }

    #[test]
    fn test_duration_from_hour_rejects_nan() {
        let err = Duration::from_hour(f64::NAN).unwrap_err();
        assert!(err.is_nan());
    }

    #[test]
    fn test_duration_from_hour_rejects_infinity() {
        let err = Duration::from_hour(f64::INFINITY).unwrap_err();
        assert!(err.is_infinite());
        assert!(err.is_sign_positive());

        let err = Duration::from_hour(f64::NEG_INFINITY).unwrap_err();
        assert!(err.is_infinite());
        assert!(err.is_sign_negative());
    }

    #[test]
    fn test_duration_rejects_below_min_value() {
        let err = Duration::from_hour(0.0).unwrap_err();
        assert_abs_diff_eq!(err, 0.0, epsilon = EPSILON);

        let err = Duration::from_hour(-1.0).unwrap_err();
        assert_abs_diff_eq!(err, -1.0, epsilon = EPSILON);

        let err = Duration::from_hour(MIN_VALUE / 2.0).unwrap_err();
        assert!(err < MIN_VALUE);
    }

    #[test]
    fn test_duration_rejects_above_max_value() {
        let err = Duration::from_hour(MAX_VALUE + 1.0).unwrap_err();
        assert!(err > MAX_VALUE);
    }

    #[test]
    fn test_as_duration() {
        let d = 1.5.hour();
        assert_abs_diff_eq!(d.0, 1.5, epsilon = EPSILON);

        let d = 90.0.minute();
        assert_abs_diff_eq!(d.0, 1.5, epsilon = EPSILON);

        let d = 3600.0.second();
        assert_abs_diff_eq!(d.0, 1.0, epsilon = EPSILON);
    }

    #[test]
    fn test_duration_comparison() {
        let d1 = Duration::from_hour(1.0).expect("1.0 should be valid");
        let d2 = Duration::from_hour(2.0).expect("2.0 should be valid");
        let d3 = Duration::from_hour(1.0).expect("1.0 should be valid");

        assert!(d1 < d2);
        assert!(d1 <= d2);
        assert!(d2 > d1);
        assert!(d2 >= d1);
        assert!(d1 == d3);
        assert!(d1 != d2);
    }

    /* --------------- EFFICIENCY TESTS ------------------- */

    #[test]
    fn test_efficiency_from_fraction_accepts_valid_values() {
        let e = Efficiency::from_fraction(0.5).expect("valid values should be accepted");
        assert_abs_diff_eq!(e.0, 0.5, epsilon = EPSILON);

        let e = Efficiency::from_fraction(1.0).expect("1.0 should be accepted");
        assert_abs_diff_eq!(e.0, 1.0, epsilon = EPSILON);

        let e = Efficiency::from_fraction(0.001).expect("small positive values should be accepted");
        assert_abs_diff_eq!(e.0, 0.001, epsilon = EPSILON);
    }

    #[test]
    fn test_efficiency_to_fraction() {
        let e = Efficiency::from_fraction(0.85).expect("valid value should be accepted");
        let val = e.as_fraction();
        assert_abs_diff_eq!(val, 0.85, epsilon = EPSILON);
    }

    #[test]
    fn test_efficiency_from_fraction_rejects_nan() {
        let err = Efficiency::from_fraction(f64::NAN).unwrap_err();
        assert!(err.is_nan());
    }

    #[test]
    fn test_efficiency_from_fraction_rejects_infinity() {
        let err = Efficiency::from_fraction(f64::INFINITY).unwrap_err();
        assert!(err.is_infinite());
        assert!(err.is_sign_positive());

        let err = Efficiency::from_fraction(f64::NEG_INFINITY).unwrap_err();
        assert!(err.is_infinite());
        assert!(err.is_sign_negative());
    }

    #[test]
    fn test_efficiency_rejects_zero_and_below() {
        let err = Efficiency::from_fraction(0.0).unwrap_err();
        assert_abs_diff_eq!(err, 0.0, epsilon = EPSILON);

        let err = Efficiency::from_fraction(-0.5).unwrap_err();
        assert_abs_diff_eq!(err, -0.5, epsilon = EPSILON);

        let err = Efficiency::from_fraction(MIN_VALUE / 2.0).unwrap_err();
        assert!(err < MIN_VALUE);
    }

    #[test]
    fn test_efficiency_rejects_above_one() {
        let err = Efficiency::from_fraction(1.1).unwrap_err();
        assert_abs_diff_eq!(err, 1.1, epsilon = EPSILON);

        let err = Efficiency::from_fraction(2.0).unwrap_err();
        assert_abs_diff_eq!(err, 2.0, epsilon = EPSILON);
    }

    #[test]
    fn test_efficiency_sqrt() {
        let e = Efficiency::from_fraction(0.81).expect("0.81 should be valid");
        assert_abs_diff_eq!(e.sqrt().as_fraction(), 0.9, epsilon = EPSILON);

        let e = Efficiency::from_fraction(1.0).expect("1.0 should be valid");
        assert_abs_diff_eq!(e.sqrt().as_fraction(), 1.0, epsilon = EPSILON);
    }

    #[test]
    fn test_as_efficiency() {
        let e = 0.9.fraction();
        assert_abs_diff_eq!(e.0, 0.9, epsilon = EPSILON);

        let e = 90.0.percent();
        assert_abs_diff_eq!(e.0, 0.9, epsilon = EPSILON);

        let e = 100.0.percent();
        assert_abs_diff_eq!(e.0, 1.0, epsilon = EPSILON);
    }

    #[test]
    fn test_efficiency_comparison() {
        let e1 = Efficiency::from_fraction(0.8).expect("0.8 should be valid");
        let e2 = Efficiency::from_fraction(0.9).expect("0.9 should be valid");
        let e3 = Efficiency::from_fraction(0.8).expect("0.8 should be valid");

        assert!(e1 < e2);
        assert!(e1 <= e2);
        assert!(e2 > e1);
        assert!(e2 >= e1);
        assert!(e1 == e3);
        assert!(e1 != e2);
    }

    /* --------------- TYPE CONVERSION TESTS ------------------- */

    #[test]
    fn test_power_times_duration_equals_energy() {
        let p = Power::from_kw(100.0).expect("100.0 should be valid");
        let d = Duration::from_hour(2.0).expect("2.0 should be valid");
        let e = p * d;
        assert_abs_diff_eq!(e.as_kwh(), 200.0, epsilon = EPSILON);
    }

    #[test]
    fn test_duration_times_power_equals_energy() {
        let p = Power::from_kw(100.0).expect("100.0 should be valid");
        let d = Duration::from_hour(2.0).expect("2.0 should be valid");
        let e = d * p;
        assert_abs_diff_eq!(e.as_kwh(), 200.0, epsilon = EPSILON);
    }

    #[test]
    fn test_energy_div_duration_equals_power() {
        let e = Energy::from_kwh(200.0).expect("200.0 should be valid");
        let d = Duration::from_hour(2.0).expect("2.0 should be valid");
        let p = e / d;
        assert_abs_diff_eq!(p.as_kw(), 100.0, epsilon = EPSILON);
    }

    #[test]
    fn test_power_div_efficiency() {
        let p = Power::from_kw(80.0).expect("80.0 should be valid");
        let eff = Efficiency::from_fraction(0.8).expect("0.8 should be valid");
        let result = p / eff;
        assert_abs_diff_eq!(result.as_kw(), 100.0, epsilon = EPSILON);
    }

    #[test]
    fn test_power_times_efficiency() {
        let p = Power::from_kw(100.0).expect("100.0 should be valid");
        let eff = Efficiency::from_fraction(0.8).expect("0.8 should be valid");
        let result = p * eff;
        assert_abs_diff_eq!(result.as_kw(), 80.0, epsilon = EPSILON);
    }

    #[test]
    fn test_efficiency_times_power() {
        let p = Power::from_kw(100.0).expect("100.0 should be valid");
        let eff = Efficiency::from_fraction(0.8).expect("0.8 should be valid");
        let result = eff * p;
        assert_abs_diff_eq!(result.as_kw(), 80.0, epsilon = EPSILON);
    }

    #[test]
    fn test_energy_div_efficiency() {
        let e = Energy::from_kwh(80.0).expect("80.0 should be valid");
        let eff = Efficiency::from_fraction(0.8).expect("0.8 should be valid");
        let result = e / eff;
        assert_abs_diff_eq!(result.as_kwh(), 100.0, epsilon = EPSILON);
    }

    #[test]
    fn test_energy_times_efficiency() {
        let e = Energy::from_kwh(100.0).expect("100.0 should be valid");
        let eff = Efficiency::from_fraction(0.8).expect("0.8 should be valid");
        let result = e * eff;
        assert_abs_diff_eq!(result.as_kwh(), 80.0, epsilon = EPSILON);
    }

    #[test]
    fn test_efficiency_times_energy() {
        let e = Energy::from_kwh(100.0).expect("100.0 should be valid");
        let eff = Efficiency::from_fraction(0.8).expect("0.8 should be valid");
        let result = eff * e;
        assert_abs_diff_eq!(result.as_kwh(), 80.0, epsilon = EPSILON);
    }

    /* --------------- DISPLAY TESTS ------------------- */

    #[test]
    fn test_energy_display() {
        let e = Energy::from_kwh(123.456).expect("123.456 should be valid");
        assert_eq!(format!("{}", e), "123.46 kWh");
        assert_eq!(format!("{:.1}", e), "123.5 kWh");
        assert_eq!(format!("{:.4}", e), "123.4560 kWh");
    }

    #[test]
    fn test_power_display() {
        let p = Power::from_kw(456.789).expect("456.789 should be valid");
        assert_eq!(format!("{}", p), "456.79 kW");
        assert_eq!(format!("{:.1}", p), "456.8 kW");
        assert_eq!(format!("{:.4}", p), "456.7890 kW");
    }

    #[test]
    fn test_duration_display() {
        let d = Duration::from_hour(2.5).expect("2.5 should be valid");
        assert_eq!(format!("{}", d), "2.50 hours");
        assert_eq!(format!("{:.1}", d), "2.5 hours");
        assert_eq!(format!("{:.4}", d), "2.5000 hours");
    }

    #[test]
    fn test_efficiency_display() {
        let e = Efficiency::from_fraction(0.85).expect("0.85 should be valid");
        assert_eq!(format!("{}", e), "0.85 %");
        assert_eq!(format!("{:.1}", e), "0.8 %");
        assert_eq!(format!("{:.4}", e), "0.8500 %");
    }
}
