use std::{
    collections::{btree_map::IntoIter, BTreeMap, HashSet},
    rc::Rc,
};

use num_traits::{Float, Signed};

use crate::{point::Point, polygon::Segment, shape::Shape};

struct Unknown;

/// Implements the clipping algorithm.                                                                                                                                   
pub struct Clipper<Operator, Subject, Clip> {
    operation: Operator,
    subject: Subject,
    clip: Clip,
}

impl<Op> Clipper<Op, Unknown, Unknown> {
    fn new(operation: Op) -> Self {
        Self {
            operation,
            subject: Unknown,
            clip: Unknown,
        }
    }
}

impl<Op, Clip> Clipper<Op, Unknown, Clip> {
    pub fn with_subject<T>(self, subject: impl Into<Shape<T>>) -> Clipper<Op, Shape<T>, Clip> {
        Clipper {
            operation: self.operation,
            subject: subject.into(),
            clip: self.clip,
        }
    }
}

impl<Op, Sub> Clipper<Op, Sub, Unknown> {
    pub fn with_clip<T>(self, clip: impl Into<Shape<T>>) -> Clipper<Op, Sub, Shape<T>> {
        Clipper {
            operation: self.operation,
            subject: self.subject,
            clip: clip.into(),
        }
    }
}

impl<T, Op> Clipper<Op, Shape<T>, Shape<T>>
where
    T: Ord + Signed + Float,
{
    pub fn execute(self) -> Shape<T> {
        let graph = ClipperGraph::default()
            .with_subject(self.subject)
            .with_clip(self.clip);

        let intersections = Intersections::from(&graph);
        for (edge, mut indexes) in intersections.by_edge {
            let vertex = graph.vertices[edge].point;
            indexes.sort_by(|&a, &b| {
                vertex
                    .distance(&intersections.all[a].point)
                    .cmp(&vertex.distance(&intersections.all[b].point))
            });
        }

        todo!()
    }
}

struct Vertex<T> {
    /// The location of the vertex.
    point: Point<T>,
    /// The index of the vertex following this one.
    next: usize,
    /// The index of the vertex previous to this one.
    previous: usize,
    /// Vertices from the oposite shape located at the same point.
    sibling: Vec<usize>,
}

/// The index of the first [`Vertex`] of a [`Polygon`] belonging to the clip or subject [`Shape`].
enum Boundary {
    Clip(usize),
    Subject(usize),
}

impl Boundary {
    fn is_subject(&self) -> bool {
        matches!(self, Boundary::Subject(_))
    }

    fn edges<'a, T>(&self, graph: &'a ClipperGraph<T>) -> impl Iterator<Item = Edge<'a, T>> {
        let start = match self {
            Boundary::Clip(index) | Boundary::Subject(index) => *index,
        };

        EdgesIterator {
            graph,
            start,
            next: start,
        }
    }
}

struct ClipperGraph<T> {
    vertices: Vec<Vertex<T>>,
    polygons: Vec<Boundary>,
}

impl<T> Default for ClipperGraph<T> {
    fn default() -> Self {
        Self {
            vertices: Default::default(),
            polygons: Default::default(),
        }
    }
}

impl<T> ClipperGraph<T> {
    fn with_subject(self, shape: Shape<T>) -> Self {
        self.with_shape(shape, Boundary::Subject)
    }

    fn with_clip(self, shape: Shape<T>) -> Self {
        self.with_shape(shape, Boundary::Clip)
    }

    fn with_shape(mut self, shape: Shape<T>, boundary: impl Fn(usize) -> Boundary) -> Self {
        self.vertices.reserve(shape.total_vertices());
        self.polygons.reserve(shape.polygons.len());

        for polygon in shape.polygons {
            let offset = self.vertices.len();
            self.polygons.push(boundary(offset));

            let total_vertices = polygon.vertices.len();
            for (mut index, vertex) in polygon.vertices.into_iter().enumerate() {
                index += total_vertices;

                self.vertices.push(Vertex {
                    point: vertex,
                    next: offset + ((index + 1) % total_vertices),
                    previous: offset + ((index - 1) % total_vertices),
                    sibling: Vec::new(),
                });
            }
        }

        self
    }
}

struct Edge<'a, T> {
    segment: Segment<'a, T>,
    index: usize,
}

struct EdgesIterator<'a, T> {
    graph: &'a ClipperGraph<T>,
    start: usize,
    next: usize,
}

impl<'a, T> Iterator for EdgesIterator<'a, T> {
    type Item = Edge<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.next;
        let vertex = &self.graph.vertices[current];
        self.next = vertex.next;

        if self.next == self.start {
            return None;
        }

        Some(Edge {
            segment: Segment {
                from: &vertex.point,
                to: &self.graph.vertices[vertex.next].point,
            },
            index: current,
        })
    }
}

/// The intersection between two edges.
struct Intersection<T> {
    /// The [`Point`] of intersection between the edges started by subject and clip.
    point: Point<T>,
    /// The index of the starting vertex in the subject edge involved in this intersection.
    subject: usize,
    /// The index of the starting vertex in the clip edge involved in this intersection.
    clip: usize,
}

/// All the intersections between the edges of a subject and clip [`Shape`]s.
struct Intersections<T> {
    all: Vec<Intersection<T>>,
    by_edge: BTreeMap<usize, Vec<usize>>,
}

impl<T> Default for Intersections<T> {
    fn default() -> Self {
        Self {
            all: Default::default(),
            by_edge: Default::default(),
        }
    }
}

impl<T> From<&ClipperGraph<T>> for Intersections<T>
where
    T: Signed + Float,
{
    fn from(graph: &ClipperGraph<T>) -> Self {
        let mut intersections = Self::default();
        for subject_polygon in graph.polygons.iter().filter(|p| p.is_subject()) {
            for clip_polygon in graph.polygons.iter().filter(|p| !p.is_subject()) {
                for subject_edge in subject_polygon.edges(graph) {
                    for clip_edge in clip_polygon.edges(graph) {
                        if let Some(intersection) =
                            subject_edge.segment.intersection(&clip_edge.segment)
                        {
                            intersections = intersections.with_intersection(Intersection {
                                point: intersection,
                                subject: subject_edge.index,
                                clip: clip_edge.index,
                            });
                        }
                    }
                }
            }
        }

        intersections
    }
}

impl<T> Intersections<T> {
    fn with_intersection(mut self, intersection: Intersection<T>) -> Self {
        match self.by_edge.get_mut(&intersection.subject) {
            Some(group) => group.push(self.all.len()),
            None => {
                self.by_edge
                    .insert(intersection.subject, vec![self.all.len()]);
            }
        };

        match self.by_edge.get_mut(&intersection.clip) {
            Some(group) => group.push(self.all.len()),
            None => {
                self.by_edge.insert(intersection.clip, vec![self.all.len()]);
            }
        };

        self.all.push(intersection);
        self
    }
}
