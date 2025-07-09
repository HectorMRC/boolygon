use crate::radian::Radian;

/// The angle between a radial and the polar axis.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct PolarAngle<T>(Radian<T>);

/// The angle of rotation of a radial line around the polar axis.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct AzimuthalAngle<T>(Radian<T>);

/// The angle of rotation of a radial line around the polar axis.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Point<T> {
    /// The angle between the radial to this point and the polar axis.
    pub polar_angle: PolarAngle<T>,
    /// The angle of rotation of the radial to this point around the polar axis.
    pub azimuthal_angle: AzimuthalAngle<T>,
}
