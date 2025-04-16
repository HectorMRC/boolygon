use std::collections::{btree_map::IntoIter, BTreeMap};

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
            .with_clip(self.clip)
            .with_intersections();

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
    vertices: Vec<Option<Vertex<T>>>,
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

                self.vertices.push(Some(Vertex {
                    point: vertex,
                    next: offset + ((index + 1) % total_vertices),
                    previous: offset + ((index - 1) % total_vertices),
                    sibling: Vec::new(),
                }));
            }
        }

        self
    }

    fn with_intersections(mut self) -> Self
    where
        T: Ord + Signed + Float,
    {
        for (index, mut intersections) in Intersections::from(&self).into_iter() {
            intersections.sort_by(|a, b| a.distance.cmp(&b.distance));
            intersections
                .into_iter()
                .enumerate()
                .inspect(|(index, intersection)| {
                    // TODO: register position for siblings
                })
                .map(|(index, intersection)| Vertex {
                    point: intersection.point,
                    next: self.vertices.len() + index + 1,
                    previous: self.vertices.len() + index - 1,
                    sibling: Vec::new(),
                });
            // TODO: register intersections and update endpoints
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
        let vertex = self.graph.vertices[current].as_ref()?;
        self.next = vertex.next;

        if self.next == self.start {
            return None;
        }

        Some(Edge {
            segment: Segment {
                from: &vertex.point,
                to: self.graph.vertices[vertex.next]
                    .as_ref()
                    .map(|vertex| &vertex.point)?,
            },
            index: current,
        })
    }
}

struct IntersectingLine<T> {
    /// The index of the vertex previous at this intersection.
    index: usize,
    /// The distance of the intersection point to the index.
    distance: T,
}

struct Intersection<T> {
    /// The intersection point.
    point: Point<T>,
    /// The index of the vertex previous at this intersection.
    previous: usize,
    /// The index of the vertex previous at this intersection at the oposite shape.
    sibling: usize,
    /// The distance from this intersection point to the previous vertex.
    distance: T,
}

struct Intersections<T>(BTreeMap<usize, Vec<Intersection<T>>>);

impl<T> Default for Intersections<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<T> From<&ClipperGraph<T>> for Intersections<T>
where
    T: Ord + Signed + Float,
{
    fn from(graph: &ClipperGraph<T>) -> Self {
        let mut intersections = Self::default();

        let Some(partition) = graph
            .polygons
            .iter()
            .enumerate()
            .skip(1)
            .find(|(index, polygon)| polygon.is_subject() != graph.polygons[index - 1].is_subject())
            .map(|(index, _)| index)
        else {
            // No partition means all polygons belongs to the same shape; hence, there are no
            // intersections.
            return intersections;
        };

        for subject_polygon in &graph.polygons[..partition] {
            for clip_polygon in &graph.polygons[partition..] {
                for subject_edge in subject_polygon.edges(graph) {
                    for clip_edge in clip_polygon.edges(graph) {
                        if let Some(intersection) =
                            subject_edge.segment.intersection(&clip_edge.segment)
                        {
                            intersections = intersections
                                .with_intersection(
                                    subject_edge.index,
                                    Intersection {
                                        point: intersection,
                                        previous: subject_edge.index,
                                        sibling: clip_edge.index,
                                        distance: subject_edge.segment.from.distance(&intersection),
                                    },
                                )
                                .with_intersection(
                                    clip_edge.index,
                                    Intersection {
                                        point: intersection,
                                        previous: clip_edge.index,
                                        sibling: subject_edge.index,
                                        distance: clip_edge.segment.from.distance(&intersection),
                                    },
                                )
                        }
                    }
                }
            }
        }

        intersections
    }
}

impl<T> IntoIterator for Intersections<T> {
    type Item = (usize, Vec<Intersection<T>>);
    type IntoIter = IntoIter<usize, Vec<Intersection<T>>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<T> Intersections<T> {
    fn with_intersection(mut self, index: usize, intersection: Intersection<T>) -> Self {
        match self.0.get_mut(&index) {
            Some(group) => group.push(intersection),
            None => {
                self.0.insert(index, vec![intersection]);
            }
        }

        self
    }
}
