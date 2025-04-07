use crate::{point::Point, polygon::Polygon, shape::Shape};

struct Noop;

struct Vertex<T> {
    point: Point<T>,
    next: usize,
    previous: usize,
    sibling: Option<usize>,
}

struct PolygonMetadata {
    first_vertex: usize,
    is_subject: bool,
}

/// Implements the clipping algorithm.                                                                                                                                   
///
/// ## Preconditions                                                                                                                                                     
///                                                                                                                                                                      
/// This clipper assumes all subject polygons are disjoint, they may intersect themselves but never                                                                      
/// other subject polygons. The same rule applies for clip polygons.
struct Clipper<T, Operation> {
    operation: Operation,
    vertices: Vec<Option<Vertex<T>>>,
    polygons: Vec<PolygonMetadata>,
}

impl<T, Operation> Clipper<T, Operation> {
    fn new(operation: Operation) -> Self {
        Self {
            operation: Operation,
            vertices: Default::default(),
            polygons: Default::default(),
        }
    }

    fn with_subject(self, polygon: Polygon<T>) -> Self {
        todo!()
    }

    fn with_clip(self, polygon: Polygon<T>) -> Self {
        todo!()
    }

    fn clip(self) -> Shape<T> {
        todo!()
    }
}
