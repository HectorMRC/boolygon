//! Radian unit.

use std::ops::Mul;

use num_traits::{Float, FloatConst, Signed};

/// The [radian](https://en.wikipedia.org/wiki/Radian) unit, which is always a positive number
/// within the range of [0, 2π).
#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd)]
pub struct Radian<T>(T);

impl<T> From<T> for Radian<T>
where
    T: Signed + Float + FloatConst,
{
    fn from(value: T) -> Self {
        if (T::zero()..T::TAU()).contains(&value) {
            return Self(value.into());
        }

        let mut modulus = value % T::TAU();
        if value.is_sign_negative() {
            modulus = (modulus + T::TAU()) % T::TAU();
        }

        Self(modulus)
    }
}

impl<T> Mul<T> for Radian<T>
where
    T: Signed + Float + FloatConst,
{
    type Output = Self;

    fn mul(self, rhs: T) -> Self::Output {
        Self::from(self.into_inner() * rhs)
    }
}

impl<T> Radian<T> {
    /// Returns the inner value.
    pub fn into_inner(self) -> T {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use std::f64::consts::{FRAC_PI_2, PI, TAU};

    use crate::radian::Radian;

    #[test]
    fn radiant_must_not_exceed_boundaries() {
        struct Test {
            name: &'static str,
            input: f64,
            output: f64,
        }

        vec![
            Test {
                name: "radiant within range must not change",
                input: PI,
                output: PI,
            },
            Test {
                name: "2π radiants must equals zero",
                input: TAU,
                output: 0.,
            },
            Test {
                name: "negative radiant must change",
                input: -FRAC_PI_2,
                output: TAU - FRAC_PI_2,
            },
            Test {
                name: "overflowing radiant must change",
                input: TAU + FRAC_PI_2,
                output: FRAC_PI_2,
            },
        ]
        .into_iter()
        .for_each(|test| {
            let radiant = Radian::from(test.input).into_inner();

            assert_eq!(
                radiant, test.output,
                "{}: got radiant = {}, want {}",
                test.name, radiant, test.output
            );
        });
    }
}
