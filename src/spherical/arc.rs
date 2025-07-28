use geocart::Cartesian;
use num_traits::{Euclid, Float, FloatConst, Signed};

use crate::{spherical::Point, Edge, Element, IsClose, Tolerance};

/// Reresents the arc between two consecutive vertices of a [`Polygon`].
#[derive(Debug)]
pub struct Arc<'a, T> {
    /// The first point in the segment.
    pub(super) from: &'a Point<T>,
    /// The last point in the segment.
    pub(super) to: &'a Point<T>,
}

impl<'a, T> Edge<'a, Point<T>> for Arc<'a, T>
where
    T: Signed + Float + FloatConst + Euclid,
{
    fn new(from: &'a Point<T>, to: &'a Point<T>) -> Self {
        Self { from, to }
    }

    fn midpoint(&self) -> Point<T> {
        if self.is_antipodal() {
            return Point {
                polar_angle: (T::FRAC_PI_2() + self.from.polar_angle.into_inner()).into(),
                azimuthal_angle: (T::FRAC_PI_2() + self.from.azimuthal_angle.into_inner()).into(),
            };
        }

        (Cartesian::from(*self.from) + Cartesian::from(*self.to))
            .normal()
            .into()
    }

    fn contains(&self, point: &Point<T>, tolerance: &Tolerance<T>) -> bool {
        (self.from.distance(point) + self.to.distance(point)).is_close(&self.length(), tolerance)
    }

    fn intersection(&self, rhs: &Self, tolerance: &Tolerance<T>) -> Option<Point<T>> {
        if self.length().is_zero() || rhs.length().is_zero() {
            return None;
        }

        if self.is_antipodal() {
            let point = self.midpoint();
            let first_half = Arc::new(self.from, &point);
            let second_half = Arc::new(&point, self.to);

            return rhs
                .intersection(&first_half, tolerance)
                .or_else(|| rhs.intersection(&second_half, tolerance));
        }

        let direction = self.normal().cross(&rhs.normal());
        if direction.magnitude().is_zero() {
            // When two arcs lie on the same great circle, their normal vectors coincide.
            return self.co_great_circular_common_point(rhs, tolerance);
        }

        if self.contains(rhs.from, tolerance) {
            return Some(*rhs.from);
        }

        if self.contains(rhs.to, tolerance) {
            return Some(*rhs.to);
        }

        if rhs.contains(self.from, tolerance) {
            return Some(*self.from);
        }

        if rhs.contains(self.to, tolerance) {
            return Some(*self.to);
        }

        let lambda = T::one() / direction.magnitude();

        let intersection = (direction * lambda).into();
        if self.contains(&intersection, tolerance) && rhs.contains(&intersection, tolerance) {
            return Some(intersection);
        }

        let intersection = (direction * -lambda).into();
        if self.contains(&intersection, tolerance) && rhs.contains(&intersection, tolerance) {
            return Some(intersection);
        }

        None
    }
}

impl<T> Arc<'_, T>
where
    T: PartialOrd + Signed + Float + FloatConst + Euclid,
{
    /// Returns the normal vector of the great circle containing the endpoints of self.
    pub(super) fn normal(&self) -> Cartesian<T> {
        let from = Cartesian::from(*self.from);
        let to = Cartesian::from(*self.to);
        from.normal().cross(&to.normal()).normal()
    }

    /// Being self and rhs two arcs lying on the same great circle, returns the single common
    /// [`Point`] between them, if any.
    fn co_great_circular_common_point(
        &self,
        rhs: &Self,
        tolerance: &Tolerance<T>,
    ) -> Option<Point<T>> {
        if (self.from.is_close(rhs.from, tolerance)
            && !self.contains(rhs.to, tolerance)
            && !rhs.contains(self.to, tolerance))
            || (self.from.is_close(rhs.to, tolerance)
                && !self.contains(rhs.from, tolerance)
                && !rhs.contains(self.to, tolerance))
        {
            return Some(*self.from);
        }

        if (self.to.is_close(rhs.from, tolerance)
            && !self.contains(rhs.to, tolerance)
            && !rhs.contains(self.from, tolerance))
            || (self.to.is_close(rhs.to, tolerance)
                && !self.contains(rhs.from, tolerance)
                && !rhs.contains(self.from, tolerance))
        {
            return Some(*self.to);
        }

        None
    }
}

impl<T> Arc<'_, T>
where
    T: Signed + Float + FloatConst + Euclid,
{
    /// Returns the distance between the two endpoints of this arc.
    fn length(&self) -> T {
        self.from.distance(self.to)
    }

    /// Returns true if, and only if, the endpoints in the arc are antipodals.
    fn is_antipodal(&self) -> bool {
        let from = Cartesian::from(*self.from);
        let to = Cartesian::from(*self.to);
        from.dot(&to) == -T::one()
    }
}
