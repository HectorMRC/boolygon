use geocart::{Cartesian, Geographic, Latitude, Longitude};
use num_traits::{Euclid, Float, FloatConst, Signed};

/// The angle between a radial line and the polar axis.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct PolarAngle<T>(T);

impl<T> From<T> for PolarAngle<T>
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

impl<T> From<Latitude<T>> for PolarAngle<T>
where
    T: Signed + FloatConst,
{
    fn from(latitude: Latitude<T>) -> Self {
        Self(T::FRAC_PI_2() - latitude.into_inner())
    }
}

impl<T> From<PolarAngle<T>> for Latitude<T>
where
    T: Signed + Float + FloatConst,
{
    fn from(angle: PolarAngle<T>) -> Self {
        (-angle.into_inner() + T::FRAC_PI_2()).into()
    }
}

impl<T> IsClose for PolarAngle<T>
where
    T: IsClose<Tolerance = Tolerance<T>> + Copy,
{
    type Tolerance = Tolerance<T>;

    fn is_close(&self, rhs: &Self, tolerance: &Self::Tolerance) -> bool {
        self.0.is_close(&rhs.0, tolerance)
    }
}

impl<T> PolarAngle<T> {
    /// Returns the inner value.
    pub fn into_inner(self) -> T {
        self.0
    }
}

/// The angle of rotation of a radial line around the polar axis.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct AzimuthalAngle<T>(T);

impl<T> From<T> for AzimuthalAngle<T>
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

impl<T> From<Longitude<T>> for AzimuthalAngle<T>
where
    T: Float + FloatConst,
{
    fn from(longitude: Longitude<T>) -> Self {
        longitude.into_inner().into()
    }
}

impl<T> From<AzimuthalAngle<T>> for Longitude<T>
where
    T: Signed + Float + FloatConst + Euclid,
{
    fn from(angle: AzimuthalAngle<T>) -> Self {
        angle.into_inner().into()
    }
}

impl<T> IsClose for AzimuthalAngle<T>
where
    T: IsClose<Tolerance = Tolerance<T>> + Copy,
{
    type Tolerance = Tolerance<T>;

    fn is_close(&self, rhs: &Self, tolerance: &Self::Tolerance) -> bool {
        self.0.is_close(&rhs.0, tolerance)
    }
}

impl<T> AzimuthalAngle<T> {
    /// Returns the inner value.
    pub fn into_inner(self) -> T {
        self.0
    }
}

/// A point on the surface of an unit sphere.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Point<T> {
    /// The angle between the radial line to this point and the polar axis.
    pub polar_angle: PolarAngle<T>,
    /// The angle of rotation of the radial line to this point around the polar axis.
    pub azimuthal_angle: AzimuthalAngle<T>,
}

impl<T> From<Point<T>> for Geographic<T>
where
    T: Signed + Float + FloatConst + Euclid,
{
    fn from(point: Point<T>) -> Self {
        Self {
            latitude: point.polar_angle.into(),
            longitude: point.azimuthal_angle.into(),
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
    fn from([polar, azimuthal]: [U; 2]) -> Self {
        Self {
            polar_angle: polar.into().into(),
            azimuthal_angle: azimuthal.into().into(),
        }
    }
}

impl<T> From<Geographic<T>> for Point<T>
where
    T: Signed + Float + FloatConst + Euclid,
{
    fn from(point: Geographic<T>) -> Self {
        Self {
            polar_angle: point.latitude.into(),
            azimuthal_angle: point.longitude.into(),
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

impl<T> Distance for Point<T>
where
    T: Signed + Float + FloatConst + Euclid,
{
    type Distance = T;

    fn distance(&self, rhs: &Self) -> T {
        let from = Cartesian::from(*self);
        let to = Cartesian::from(*rhs);

        // The perimeter of a unit circle is 2π; hence, the longitude of an arc is equivalent to
        // the angle between its endpoints.
        //
        // The formula for the angle (in radians) between two vectors is the arccosine of the
        // division between the dot product and the product of the magnitudes.
        //
        // Being both vectors normalized (magnitude = 1), the formula gets simplified as the
        // arccosine of the dot product.
        from.dot(&to).acos()
    }
}

impl<T> IsClose for Point<T>
where
    T: IsClose<Tolerance = Tolerance<T>> + Copy,
{
    type Tolerance = Tolerance<T>;

    fn is_close(&self, rhs: &Self, tolerance: &Self::Tolerance) -> bool {
        self.polar_angle.is_close(&rhs.polar_angle, tolerance)
            && self
                .azimuthal_angle
                .is_close(&rhs.azimuthal_angle, tolerance)
    }
}

use crate::{Distance, IsClose, Tolerance};
