use std::cmp::Ordering;
use std::ops::{Add, Sub};

#[derive(Debug, Clone, Copy)]
pub struct Energy(f64);

impl Energy {
    pub fn from_kw(energy_kw: f64) -> Result<Self, f64> {
        if energy_kw.is_infinite() || energy_kw.is_nan() {
            Err(energy_kw)
        } else {
            Ok(Self(energy_kw))
        }
    }

    pub fn to_kwh(&self) -> f64 {
        self.0
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

impl PartialEq<Self> for Energy {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl PartialOrd for Energy {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }

    fn lt(&self, other: &Self) -> bool {
        self.0 < other.0
    }

    fn le(&self, other: &Self) -> bool {
        self.0 <= other.0
    }

    fn gt(&self, other: &Self) -> bool {
        self.0 > other.0
    }

    fn ge(&self, other: &Self) -> bool {
        self.0 >= other.0
    }
}

pub trait AsEnergy {
    fn mwh(self) -> Energy;

    fn kwh(self) -> Energy;

    fn wh(self) -> Energy;
}

impl AsEnergy for f64 {
    fn mwh(self) -> Energy {
        Energy::from_kw(self * 1_000.).expect("Invalid energy value")
    }
    fn kwh(self) -> Energy {
        Energy::from_kw(self).expect("Invalid energy value")
    }
    fn wh(self) -> Energy {
        Energy::from_kw(self / 1_000.).expect("Invalid energy value")
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use approx::{assert_abs_diff_eq};
    const EPSILON: f64 = 1e-12;

    #[test]
    fn test_energy_from_kw_accepts_finite_values() {
        let e:Energy = Energy::from_kw(123.45).expect("finite values should be accepted");
        assert_abs_diff_eq!(e.0, 123.45, epsilon=EPSILON);

        let e:Energy = Energy::from_kw(-10.0).expect("finite negative values are allowed for Energy");
        assert_abs_diff_eq!(e.0, -10.0, epsilon=EPSILON);

        let e:Energy = Energy::from_kw(0.0).expect("zero should be accepted");
        assert_abs_diff_eq!(e.0, 0.0, epsilon=EPSILON);
    }

    #[test]
    fn test_energy_to_kw() {
        let e:Energy = Energy::from_kw(123.45).expect("finite values should be accepted");
        let val:f64 = e.to_kwh();
        assert_abs_diff_eq!(val, 123.45, epsilon=EPSILON);
    }

    #[test]
    fn test_energy_from_kw_rejects_nan() {
        let err = Energy::from_kw(f64::NAN).unwrap_err();
        assert!(err.is_nan());
    }

    #[test]
    fn test_energy_from_kw_rejects_infinity() {
        let err = Energy::from_kw(f64::INFINITY).unwrap_err();
        assert!(err.is_infinite());
        assert!(err.is_sign_positive());

        let err = Energy::from_kw(f64::NEG_INFINITY).unwrap_err();
        assert!(err.is_infinite());
        assert!(err.is_sign_negative());
    }

    #[test]
    fn test_energy_add_energy() {
        let e:Energy = Energy::from_kw(4.1).expect("4.1 should be accepted");
        let e2:Energy = Energy::from_kw(-5.1).expect("-5.1 should be accepted");
        let e3:Energy = e + e2;
        assert_abs_diff_eq!(e3.0, 4.1 - 5.1, epsilon=EPSILON);
    }

    #[test]
    fn test_energy_sub_energy() {
        let e:Energy = Energy::from_kw(4.1).expect("4.1 should be accepted");
        let e2:Energy = Energy::from_kw(-5.1).expect("-5.1 should be accepted");
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
        let e1 = Energy::from_kw(10.0).unwrap();
        let e2 = Energy::from_kw(20.0).unwrap();
        let e3 = Energy::from_kw(10.0).unwrap();

        assert!(e1 < e2);
        assert!(e1 <= e2);
        assert!(e2 > e1);
        assert!(e2 >= e1);
        assert!(e1 == e3);
        assert!(e1 != e2);
    }
}
