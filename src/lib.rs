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

/// A [`Geometry`] whose orientation is defined by the right-hand rule.
pub trait RightHanded {
    /// Returns true if, and only if, this geometry is oriented clockwise.
    fn is_clockwise(&self) -> bool;
}

/// A type whose distance to other instances of itself is defined.
pub trait Distance {
    /// The distance type.
    type Distance;

    /// Returns the distance between self and rhs.
    fn distance(&self, rhs: &Self) -> Self::Distance;
}

/// An [`Edge`] that can intersect with other instances of itself.
pub trait Intersection {
    /// The intersection type.
    type Intersection: Distance;

    /// Returns the intersection between self and rhs, if any.
    fn intersection(
        &self,
        rhs: &Self,
        tolerance: &Tolerance<<Self::Intersection as Distance>::Distance>,
    ) -> Option<Self::Intersection>;
}

/// An [`Edge`] whose midpoint is defined.
pub trait Midpoint {
    /// The midpoint type.
    type Point;

    /// Returns the middle point of this edge.
    fn midpoint(&self) -> Self::Point;
}

/// A [`Geometry`] that winds around points in its space.
pub trait Winding {
    /// The type of point this geometry winds around.
    type Point: Distance;

    /// Returns this geometry with the reversed winding.
    fn reversed(self) -> Self;

    /// Returns the amount of times this geometry winds around the given point.
    fn winding(
        &self,
        point: &Self::Point,
        tolerance: &Tolerance<<Self::Point as Distance>::Distance>,
    ) -> isize;
}

/// An edge delimited by two endpoints in a [`Geometry`].
pub trait Edge<'a> {
    /// The enpoint type.
    type Endpoint: Distance;

    /// Returns a edge from the given endpoints.
    fn new(from: &'a Self::Endpoint, to: &'a Self::Endpoint) -> Self;

    /// Returns true if, and only if, the given point exists in this edge.
    fn contains(
        &self,
        point: &Self::Endpoint,
        tolerance: &Tolerance<<Self::Endpoint as Distance>::Distance>,
    ) -> bool;
}

/// A [`Geometry`] that can be constructed from clipper's raw data.
pub trait FromRaw: Sized + Geometry {
    /// Tries to construct the geometry from the given raw data.
    fn from_raw(
        operands: Operands<Self>,
        vertices: Vec<Self::Point>,
        tolerance: &Tolerance<<Self::Point as Distance>::Distance>,
    ) -> Option<Self>;
}

/// A geometry in an arbitrary space.
pub trait Geometry: Winding {
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
    T: RightHanded + Winding,
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
    for<'a> T::Edge<'a>: Edge<'a, Endpoint = T::Point>
        + Midpoint<Point = T::Point>
        + Intersection<Intersection = T::Point>,
    T::Point: Distance
        + Copy
        + IsClose<Tolerance = Tolerance<<T::Point as Distance>::Distance>>
        + PartialEq
        + PartialOrd,
    <T::Point as Distance>::Distance: Copy + PartialOrd,
{
    /// Returns the union of self and rhs.
    pub fn or(self, rhs: Self, tolerance: Tolerance<<T::Point as Distance>::Distance>) -> Self {
        struct OrOperator<T>(PhantomData<T>);

        impl<T> Operator<T> for OrOperator<T>
        where
            T: Geometry,
            for<'a> T::Edge<'a>: Edge<'a, Endpoint = T::Point>,
            <T::Point as Distance>::Distance: Copy,
        {
            fn is_output<'a>(
                ops: Operands<'a, T>,
                vertex: &'a Vertex<T>,
                tolerance: &Tolerance<<T::Point as Distance>::Distance>,
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
        tolerance: Tolerance<<T::Point as Distance>::Distance>,
    ) -> Option<Self> {
        struct NotOperator<T>(PhantomData<T>);

        impl<T> Operator<T> for NotOperator<T>
        where
            T: Geometry,
            for<'a> T::Edge<'a>: Edge<'a, Endpoint = T::Point>,
            <T::Point as Distance>::Distance: Copy,
        {
            fn is_output<'a>(
                ops: Operands<'a, T>,
                vertex: &'a Vertex<T>,
                tolerance: &Tolerance<<T::Point as Distance>::Distance>,
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
        tolerance: Tolerance<<T::Point as Distance>::Distance>,
    ) -> Option<Self> {
        struct AndOperator<T>(PhantomData<T>);

        impl<T> Operator<T> for AndOperator<T>
        where
            T: Geometry,
            for<'a> T::Edge<'a>: Edge<'a, Endpoint = T::Point>,
            <T::Point as Distance>::Distance: Copy,
        {
            fn is_output<'a>(
                ops: Operands<'a, T>,
                vertex: &'a Vertex<T>,
                tolerance: &Tolerance<<T::Point as Distance>::Distance>,
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
        tolerance: &Tolerance<<T::Point as Distance>::Distance>,
    ) -> bool {
        self.polygons
            .iter()
            .flat_map(|polygon| polygon.edges())
            .any(|segment| segment.contains(point, tolerance))
    }
}

impl<T> Shape<T>
where
    T: Geometry,
    T::Point: Distance,
    <T::Point as Distance>::Distance: Copy,
{
    /// Returns the amount of times self winds around the given [`Point`].
    fn winding(
        &self,
        point: &T::Point,
        tolerance: &Tolerance<<T::Point as Distance>::Distance>,
    ) -> isize {
        self.polygons
            .iter()
            .map(|polygon| polygon.winding(point, tolerance))
            .sum()
    }

    /// Returns true if, and only if, self contains the given [`Point`].
    pub(crate) fn contains(
        &self,
        point: &T::Point,
        tolerance: &Tolerance<<T::Point as Distance>::Distance>,
    ) -> bool {
        self.winding(point, tolerance) != 0
    }
}

impl<T> Shape<T>
where
    T: RightHanded + Winding,
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
    T: Winding,
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
        self.polygons.iter().flat_map(|polygon| polygon.edges())
    }
}
