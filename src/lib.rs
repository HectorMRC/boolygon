mod clipper;
mod graph;
mod tolerance;
mod vertex;

#[cfg(feature = "cartesian")]
pub mod cartesian;
#[cfg(feature = "spherical")]
pub mod spherical;

use std::{fmt::Debug, marker::PhantomData};

pub use self::clipper::Operands;
pub use self::tolerance::{IsClose, Positive, Tolerance};

use self::{
    clipper::{Clipper, Operator},
    vertex::{Role, Vertex},
};

/// A geometry whose orientation is defined by the right-hand rule.
pub trait RightHanded {
    /// Returns true if, and only if, self is oriented clockwise.
    fn is_clockwise(&self) -> bool;
}

/// A point whose distance to other points is defined.
pub trait Metric {
    /// The scalar type representing the distance.
    type Scalar;

    /// Returns the distance between this point and rhs.
    fn distance(&self, rhs: &Self) -> Self::Scalar;
}

/// A segment that can intersect with other instances of its type.
pub trait Secant {
    /// The point of intersection.
    type Point: Metric;

    /// Returns the point of intersection between this segment and rhs, if any.
    fn intersection(
        &self,
        rhs: &Self,
        tolerance: &Tolerance<<Self::Point as Metric>::Scalar>,
    ) -> Option<Self::Point>;
}

/// A segment whose midpoint is defined.
pub trait Midpoint {
    /// The midpoint type.
    type Point;

    /// Returns the middle point between the endpoints of this segment.
    fn midpoint(&self) -> Self::Point;
}

/// A geometry that may wind around points.
pub trait Wind {
    /// The point whose winding number can be inferred.
    type Point: Metric;

    /// Returns this geometry with the reversed winding.
    fn reversed(self) -> Self;

    /// Returns the amount of time this geometry winds around the given point.
    fn winding(
        &self,
        point: &Self::Point,
        tolerance: &Tolerance<<Self::Point as Metric>::Scalar>,
    ) -> isize;
}

pub trait Edge<'a> {
    /// The enpoint type of the segment.
    type Endpoint: Metric;

    /// Returns a segment from the given endpoints.
    fn new(from: &'a Self::Endpoint, to: &'a Self::Endpoint) -> Self;

    /// Returns true if, and only if, the given point exists in this segment.
    fn contains(
        &self,
        point: &Self::Endpoint,
        tolerance: &Tolerance<<Self::Endpoint as Metric>::Scalar>,
    ) -> bool;
}

/// Construction from a geometric operation given the operands and resulting vertices.
pub trait FromRaw
where
    Self: Sized + Geometry,
{
    /// Tries to construct a geometry from the given iterator of vertices.
    fn from_raw(
        operands: Operands<Self>,
        vertices: Vec<Self::Point>,
        tolerance: &Tolerance<<Self::Point as Metric>::Scalar>,
    ) -> Option<Self>;
}

/// A geometry in an arbitrary space.
pub trait Geometry: Wind {
    /// The type of edge in this geometry.
    type Edge<'a>: Edge<'a>
    where
        Self: 'a;

    /// Returns the total amount of vertices in the geometry.
    fn total_vertices(&self) -> usize;

    /// Returns an ordered iterator over all the segmentss of this geometry.
    fn edges(&self) -> impl Iterator<Item = Self::Edge<'_>>;
}

/// A combination of disjoint polygons.
#[derive(Debug, Clone)]
pub struct Shape<T> {
    /// The list of non-crossing polygons.
    pub(crate) polygons: Vec<T>,
}

impl<T> From<T> for Shape<T>
where
    T: RightHanded + Wind,
{
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

impl<T> PartialEq for Shape<T>
where
    T: PartialEq + Clone,
{
    fn eq(&self, other: &Self) -> bool {
        if self.polygons.len() != other.polygons.len() {
            return false;
        }

        self.polygons
            .iter()
            .all(|a| other.polygons.iter().any(|b| a.eq(b)))
    }
}

impl<T> Shape<T>
where
    T: RightHanded + Geometry + FromRaw + Clone + IntoIterator<Item = T::Point>,
    for<'a> T::Edge<'a>:
        Edge<'a, Endpoint = T::Point> + Midpoint<Point = T::Point> + Secant<Point = T::Point>,
    T::Point: Metric
        + Copy
        + IsClose<Tolerance = Tolerance<<T::Point as Metric>::Scalar>>
        + PartialEq
        + PartialOrd,
    <T::Point as Metric>::Scalar: Copy + PartialOrd,
{
    /// Returns the union of self and rhs.
    pub fn or(self, rhs: Self, tolerance: Tolerance<<T::Point as Metric>::Scalar>) -> Self {
        struct OrOperator<T>(PhantomData<T>);

        impl<T> Operator<T> for OrOperator<T>
        where
            T: Geometry,
            for<'a> T::Edge<'a>: Edge<'a, Endpoint = T::Point>,
            <T::Point as Metric>::Scalar: Copy,
        {
            fn is_output<'a>(
                ops: Operands<'a, T>,
                vertex: &'a Vertex<T>,
                tolerance: &Tolerance<<T::Point as Metric>::Scalar>,
            ) -> bool {
                match vertex.role {
                    Role::Subject => {
                        !ops.clip.contains(&vertex.point, tolerance)
                            || ops.clip.is_boundary(&vertex.point, tolerance)
                    }
                    Role::Clip => {
                        !ops.subject.contains(&vertex.point, tolerance)
                            || ops.subject.is_boundary(&vertex.point, tolerance)
                    }
                }
            }
        }

        Clipper::new(tolerance)
            .with_operator::<OrOperator<T>>()
            .with_subject(self)
            .with_clip(rhs)
            .execute()
            .expect("union should always return a shape")
    }

    /// Returns the difference of rhs on self.
    pub fn not(
        self,
        rhs: Self,
        tolerance: Tolerance<<T::Point as Metric>::Scalar>,
    ) -> Option<Self> {
        struct NotOperator<T>(PhantomData<T>);

        impl<T> Operator<T> for NotOperator<T>
        where
            T: Geometry,
            for<'a> T::Edge<'a>: Edge<'a, Endpoint = T::Point>,
            <T::Point as Metric>::Scalar: Copy,
        {
            fn is_output<'a>(
                ops: Operands<'a, T>,
                vertex: &'a Vertex<T>,
                tolerance: &Tolerance<<T::Point as Metric>::Scalar>,
            ) -> bool {
                match vertex.role {
                    Role::Subject => {
                        !ops.clip.contains(&vertex.point, tolerance)
                            && !ops.clip.is_boundary(&vertex.point, tolerance)
                    }
                    Role::Clip => {
                        ops.subject.contains(&vertex.point, tolerance)
                            && !ops.subject.is_boundary(&vertex.point, tolerance)
                    }
                }
            }
        }

        Clipper::new(tolerance)
            .with_operator::<NotOperator<T>>()
            .with_clip(rhs.inverted_winding())
            .with_subject(self)
            .execute()
    }

    /// Returns the intersection of self and rhs.
    pub fn and(
        self,
        rhs: Self,
        tolerance: Tolerance<<T::Point as Metric>::Scalar>,
    ) -> Option<Self> {
        struct AndOperator<T>(PhantomData<T>);

        impl<T> Operator<T> for AndOperator<T>
        where
            T: Geometry,
            for<'a> T::Edge<'a>: Edge<'a, Endpoint = T::Point>,
            <T::Point as Metric>::Scalar: Copy,
        {
            fn is_output<'a>(
                ops: Operands<'a, T>,
                vertex: &'a Vertex<T>,
                tolerance: &Tolerance<<T::Point as Metric>::Scalar>,
            ) -> bool {
                match vertex.role {
                    Role::Subject => {
                        ops.clip.contains(&vertex.point, tolerance)
                            || ops.clip.is_boundary(&vertex.point, tolerance)
                    }
                    Role::Clip => {
                        ops.subject.contains(&vertex.point, tolerance)
                            || ops.subject.is_boundary(&vertex.point, tolerance)
                    }
                }
            }
        }

        Clipper::new(tolerance)
            .with_operator::<AndOperator<T>>()
            .with_subject(self)
            .with_clip(rhs)
            .execute()
    }
}

impl<T> Shape<T>
where
    T: Geometry,
    for<'a> T::Edge<'a>: Edge<'a, Endpoint = T::Point>,
{
    /// Returns true if, and only if, the given point is in any of the outlines of this shape.
    pub(crate) fn is_boundary(
        &self,
        point: &T::Point,
        tolerance: &Tolerance<<T::Point as Metric>::Scalar>,
    ) -> bool {
        self.polygons
            .iter()
            .map(|polygon| polygon.edges())
            .flatten()
            .any(|segment| segment.contains(point, tolerance))
    }
}

impl<T> Shape<T>
where
    T: Geometry,
    T::Point: Metric,
    <T::Point as Metric>::Scalar: Copy,
{
    /// Returns the amount of times self winds around the given [`Point`].
    fn winding(
        &self,
        point: &T::Point,
        tolerance: &Tolerance<<T::Point as Metric>::Scalar>,
    ) -> isize {
        self.polygons
            .iter()
            .map(|polygon| polygon.winding(point, &tolerance))
            .sum()
    }

    /// Returns true if, and only if, self contains the given [`Point`].
    pub(crate) fn contains(
        &self,
        point: &T::Point,
        tolerance: &Tolerance<<T::Point as Metric>::Scalar>,
    ) -> bool {
        self.winding(point, tolerance) != 0
    }
}

impl<T> Shape<T>
where
    T: RightHanded + Wind,
{
    /// Creates a new shape from the given polygon.
    pub fn new(value: impl Into<T>) -> Self {
        let polygon = value.into();

        Self {
            polygons: vec![if polygon.is_clockwise() {
                polygon.reversed()
            } else {
                polygon
            }],
        }
    }
}

impl<T> Shape<T>
where
    T: Wind,
{
    /// Returns  a new shape with the inverted winding.
    fn inverted_winding(self) -> Self {
        Self {
            polygons: self.polygons.into_iter().map(T::reversed).collect(),
        }
    }
}

impl<T> Shape<T>
where
    T: Geometry,
{
    /// Returns the amount of vertices in the shape.
    pub(crate) fn total_vertices(&self) -> usize {
        self.polygons
            .iter()
            .map(|polygon| polygon.total_vertices())
            .sum()
    }

    pub(crate) fn edges(&self) -> impl Iterator<Item = T::Edge<'_>> {
        self.polygons
            .iter()
            .map(|polygon| polygon.edges())
            .flatten()
    }
}
