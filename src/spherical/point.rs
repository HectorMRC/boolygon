use geocart::{Cartesian, Geographic, Latitude, Longitude};
use num_traits::{Euclid, Float, FloatConst, Signed};

use crate::{IsClose, Tolerance, Vertex};

/// The angle between a radial line and the polar axis.
///
/// ## Definition
/// Since the inclination (or polar angle) of a point on a unit sphere is the smallest angle
/// between the radial line to that point and the positive polar axis, the angle must be in the
/// range __\[0, π\]__.
/// Any other value must be computed in order to set its equivalent inside the range.
///
/// ### Overflow
/// Overflowing any of both boundaries of the inclination range behaves like moving away from
/// that boundary and getting closer to the opposite one.
///
/// ## Example
/// ```
/// use std::f64::consts::FRAC_PI_2;
///
/// use boolygon::{spherical::Inclination, Tolerance, IsClose};
///
/// let overflowing_polar = Inclination::from(3. * FRAC_PI_2);
/// let equivalent_polar = Inclination::from(FRAC_PI_2);
///
/// // due precision error both values may not be exactly the same  
/// let tolerance = Tolerance {
///     relative: 1e-9.into(),
///     ..Default::default()
/// };
///
/// assert!(
///     overflowing_polar.is_close(&equivalent_polar, &tolerance),
///     "the overflowing inclination should be close to the equivalent one"
/// );
/// ```
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Inclination<T>(T);

impl<T> From<T> for Inclination<T>
where
    T: Float + FloatConst,
{
    fn from(value: T) -> Self {
        Self(if (T::zero()..=T::PI()).contains(&value) {
            value
        } else {
            value.cos().acos()
        })
    }
}

impl<T> From<Latitude<T>> for Inclination<T>
where
    T: Signed + FloatConst,
{
    fn from(latitude: Latitude<T>) -> Self {
        Self(T::FRAC_PI_2() - latitude.into_inner())
    }
}

impl<T> From<Inclination<T>> for Latitude<T>
where
    T: Signed + Float + FloatConst,
{
    fn from(angle: Inclination<T>) -> Self {
        (-angle.into_inner() + T::FRAC_PI_2()).into()
    }
}

impl<T> IsClose for Inclination<T>
where
    T: IsClose<Tolerance = Tolerance<T>>,
{
    type Tolerance = Tolerance<T>;

    fn is_close(&self, other: &Self, tolerance: &Self::Tolerance) -> bool {
        self.0.is_close(&other.0, tolerance)
    }
}

impl<T> Inclination<T> {
    /// Returns the inner value.
    pub fn into_inner(self) -> T {
        self.0
    }
}

/// The angle of rotation of a radial line around the polar axis.
///
/// ## Definition
/// Since the azimuth (or azimuthal angle) of a point on a unit sphere is the positive right-handed
/// angle of the radial line to that point around the polar axis, the angle must be in the range
/// __\[0, 2π\)__.
/// Any other value will be computed in order to set its equivalent inside that range.
///
/// ### Overflow
/// Both boundaries of the azimuth range are consecutive, which means that overflowing one is the
/// same as continuing from the other one in the same direction.
///
/// ## Example
/// ```
/// use std::f64::consts::PI;
///
/// use boolygon::{spherical::Azimuth, Tolerance, IsClose};
///
/// let overflowing_azimuth = Azimuth::from(3. * PI);
/// let equivalent_azimuth = Azimuth::from(PI);
///
/// // due precision error both values may not be exactly the same  
/// let tolerance = Tolerance {
///     relative: 1e-9.into(),
///     ..Default::default()
/// };
///
/// assert!(
///     overflowing_azimuth.is_close(&equivalent_azimuth, &tolerance),
///     "the overflowing azimuth should be close to the equivalent one"
/// );
/// ```
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Azimuth<T>(T);

impl<T> From<T> for Azimuth<T>
where
    T: Float + FloatConst,
{
    fn from(value: T) -> Self {
        if (T::zero()..T::TAU()).contains(&value) {
            return Self(value);
        }

        let mut modulus = value % T::TAU();
        if value.is_sign_negative() {
            modulus = (modulus + T::TAU()) % T::TAU();
        }

        Self(modulus)
    }
}

impl<T> From<Longitude<T>> for Azimuth<T>
where
    T: Float + FloatConst,
{
    fn from(longitude: Longitude<T>) -> Self {
        longitude.into_inner().into()
    }
}

impl<T> From<Azimuth<T>> for Longitude<T>
where
    T: Signed + Float + FloatConst + Euclid,
{
    fn from(angle: Azimuth<T>) -> Self {
        angle.into_inner().into()
    }
}

impl<T> IsClose for Azimuth<T>
where
    T: IsClose<Tolerance = Tolerance<T>>,
{
    type Tolerance = Tolerance<T>;

    fn is_close(&self, other: &Self, tolerance: &Self::Tolerance) -> bool {
        self.0.is_close(&other.0, tolerance)
    }
}

impl<T> Azimuth<T> {
    /// Returns the inner value.
    pub fn into_inner(self) -> T {
        self.0
    }
}

/// A point on the surface of a unit sphere.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Point<T> {
    /// The angle between the radial line to this point and the polar axis.
    pub inclination: Inclination<T>,
    /// The angle of rotation of the radial line to this point around the polar axis.
    pub azimuth: Azimuth<T>,
}

impl<T> From<Point<T>> for Geographic<T>
where
    T: Signed + Float + FloatConst + Euclid,
{
    fn from(point: Point<T>) -> Self {
        Self {
            latitude: point.inclination.into(),
            longitude: point.azimuth.into(),
            altitude: T::one().into(),
        }
    }
}

impl<T> From<Point<T>> for Cartesian<T>
where
    T: PartialOrd + Signed + Float + FloatConst + Euclid,
{
    fn from(point: Point<T>) -> Self {
        Geographic::from(point).into()
    }
}

impl<T, U> From<[U; 2]> for Point<T>
where
    T: PartialOrd + Signed + Float + FloatConst + Euclid,
    U: Into<T>,
{
    fn from([inclination, azimuth]: [U; 2]) -> Self {
        Self {
            inclination: inclination.into().into(),
            azimuth: azimuth.into().into(),
        }
    }
}

impl<T> From<Geographic<T>> for Point<T>
where
    T: Signed + Float + FloatConst + Euclid,
{
    fn from(point: Geographic<T>) -> Self {
        Self {
            inclination: point.latitude.into(),
            azimuth: point.longitude.into(),
        }
    }
}

impl<T> From<Cartesian<T>> for Point<T>
where
    T: PartialOrd + Signed + Float + FloatConst + Euclid,
{
    fn from(point: Cartesian<T>) -> Self {
        Geographic::from(point).into()
    }
}

impl<T> Vertex for Point<T>
where
    T: Signed + Float + FloatConst + Euclid,
{
    type Scalar = T;

    /// Returns the [great-circle distance](https://en.wikipedia.org/wiki/Great-circle_distance)
    /// from this point to the other.
    fn distance(&self, other: &Self) -> T {
        Geographic::from(*self).distance(&(*other).into())
    }
}

impl<T> IsClose for Point<T>
where
    T: IsClose<Tolerance = Tolerance<T>>,
{
    type Tolerance = Tolerance<T>;

    fn is_close(&self, other: &Self, tolerance: &Self::Tolerance) -> bool {
        self.inclination.is_close(&other.inclination, tolerance)
            && self.azimuth.is_close(&other.azimuth, tolerance)
    }
}

#[cfg(test)]
mod tests {
    use std::f64::consts::{FRAC_PI_2, PI, TAU};

    use crate::{
        spherical::{Azimuth, Inclination},
        IsClose, Tolerance,
    };

    #[test]
    fn inclination_must_not_exceed_boundaries() {
        struct Test {
            name: &'static str,
            input: f64,
            output: f64,
        }

        vec![
            Test {
                name: "angle within range must not change",
                input: PI,
                output: PI,
            },
            Test {
                name: "2π radians must equals zero",
                input: TAU,
                output: 0.,
            },
            Test {
                name: "negative angle must change",
                input: -FRAC_PI_2,
                output: FRAC_PI_2,
            },
            Test {
                name: "overflowing angle must change",
                input: 3. * FRAC_PI_2,
                output: FRAC_PI_2,
            },
        ]
        .into_iter()
        .for_each(|test| {
            let inclination = Inclination::from(test.input).into_inner();

            let tolerance = Tolerance {
                relative: 1e-9.into(),
                ..Default::default()
            };

            assert!(
                inclination.is_close(&test.output, &tolerance),
                "{}: got inclination = {}, want {}",
                test.name,
                inclination,
                test.output
            );
        });
    }

    #[test]
    fn azimuth_must_not_exceed_boundaries() {
        struct Test {
            name: &'static str,
            input: f64,
            output: f64,
        }

        vec![
            Test {
                name: "angle within range must not change",
                input: PI,
                output: PI,
            },
            Test {
                name: "2π radians must equals zero",
                input: TAU,
                output: 0.,
            },
            Test {
                name: "negative angle must change",
                input: -FRAC_PI_2,
                output: TAU - FRAC_PI_2,
            },
            Test {
                name: "overflowing angle must change",
                input: TAU + FRAC_PI_2,
                output: FRAC_PI_2,
            },
        ]
        .into_iter()
        .for_each(|test| {
            let azimuth = Azimuth::from(test.input).into_inner();

            assert_eq!(azimuth, test.output, "{}", test.name);
        });
    }
}
