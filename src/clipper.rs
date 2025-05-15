use std::{cmp::Ordering, collections::BTreeMap, fmt::Debug};

use num_traits::{Float, Signed};

use crate::{point::Point, polygon::Segment, shape::Shape};

pub struct Unknown;

/// Implements the clipping algorithm.                                                                                                                                   
pub struct Clipper<Operator, Subject, Clip> {
    operation: Operator,
    subject: Subject,
    clip: Clip,
}

impl<Op> Clipper<Op, Unknown, Unknown> {
    pub fn new(operation: Op) -> Self {
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
    T: Clone + PartialOrd + Signed + Float + Debug,
{
    pub fn execute(self) -> Option<Shape<T>> {
        let mut graph = ClipperGraph::default()
            .with_subject(self.subject.clone())
            .with_clip(self.clip.clone())
            .with_intersections();

        let mut shape = None;
        while let Some(iter) = graph
            .position_where(Vertex::is_intersection)
            .map(|position| VerticesIterator {
                clipper: &self,
                graph: &mut graph,
                next: position,
            })
        {
            let polygon = iter.map(|vertex| vertex.point).collect::<Vec<_>>().into();
            match shape.as_mut() {
                None => shape = Some(Shape::from(polygon)),
                Some(shape) => shape.polygons.push(polygon),
            };
        }

        while let Some(iter) = graph
            .position_where(|vertex| {
                self.subject.contains(&vertex.point) ^ self.clip.contains(&vertex.point)
            })
            .map(|position| VerticesIterator {
                clipper: &self,
                graph: &mut graph,
                next: position,
            })
        {
            let start = iter.next;
            let vertices = iter.collect::<Vec<_>>();

            if vertices[vertices.len() - 1].next != start {
                // The succession of vertices is an open shape.
                continue;
            }

            let polygon = vertices
                .into_iter()
                .map(|vertex| vertex.point)
                .collect::<Vec<_>>()
                .into();

            match shape.as_mut() {
                None => shape = Some(Shape::from(polygon)),
                Some(shape) => shape.polygons.push(polygon),
            };
        }

        shape
    }
}

#[derive(Debug)]
struct Vertex<T> {
    /// The location of the vertex.
    point: Point<T>,
    /// The index of the vertex following this one.
    next: usize,
    /// The index of the vertex previous to this one.
    previous: usize,
    /// Vertices from the oposite shape located at the same point.
    siblings: Vec<usize>,
}

impl<T> Vertex<T> {
    fn is_intersection(&self) -> bool {
        !self.siblings.is_empty()
    }
}

struct VerticesIterator<'a, Op, T> {
    clipper: &'a Clipper<Op, Shape<T>, Shape<T>>,
    graph: &'a mut ClipperGraph<T>,
    next: usize,
}

impl<'a, Op, T> Iterator for VerticesIterator<'a, Op, T>
where
    T: Signed + Float,
{
    type Item = Vertex<T>;

    fn next(&mut self) -> Option<Self::Item> {
        let vertex = self.graph.vertices[self.next].take()?;

        self.next = vertex
            .siblings
            .iter()
            .find_map(|&sibling| {
                if self.graph.vertices[sibling]
                    .as_ref()
                    .and_then(|vertex| self.graph.vertices[vertex.next].as_ref())
                    .is_some_and(|vertex| {
                        self.clipper.subject.contains(&vertex.point)
                            ^ self.clipper.clip.contains(&vertex.point)
                    })
                {
                    return self.graph.vertices[sibling]
                        .take()
                        .map(|vertex| vertex.next);
                }

                None
            })
            .unwrap_or(vertex.next);

        Some(vertex)
    }
}

/// The index of the first [`Vertex`] of a [`Polygon`] belonging to the clip or subject [`Shape`].
#[derive(Debug)]
enum Boundary {
    Clip(usize),
    Subject(usize),
}

impl Boundary {
    fn is_subject(&self) -> bool {
        matches!(self, Boundary::Subject(_))
    }
}

#[derive(Debug)]
struct Edge<'a, T> {
    segment: Segment<'a, T>,
    index: usize,
}

struct EdgesIterator<'a, T> {
    graph: &'a ClipperGraph<T>,
    next: Option<usize>,
    start: usize,
}

impl<'a, T> Iterator for EdgesIterator<'a, T> {
    type Item = Edge<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.next?;
        let vertex = self.graph.vertices[current].as_ref()?;
        self.next = if vertex.next != self.start {
            Some(vertex.next)
        } else {
            None
        };

        Some(Edge {
            segment: Segment {
                from: &vertex.point,
                to: &self.graph.vertices[vertex.next].as_ref()?.point,
            },
            index: current,
        })
    }
}

/// The intersection between two edges.
#[derive(Debug)]
struct Intersection<T> {
    /// The [`Point`] of intersection between the edges started by subject and clip.
    point: Point<T>,
    /// The index of the starting vertex in the subject edge involved in this intersection.
    subject: usize,
    /// The index of the starting vertex in the clip edge involved in this intersection.
    clip: usize,
}

/// All the intersections between the edges of a subject and clip [`Shape`]s.
#[derive(Debug)]
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
                for subject_edge in graph.edges(subject_polygon) {
                    for clip_edge in graph.edges(clip_polygon) {
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
        let index = self.all.len();

        match self.by_edge.get_mut(&intersection.subject) {
            Some(group) => group.push(index),
            None => {
                self.by_edge.insert(intersection.subject, vec![index]);
            }
        };

        match self.by_edge.get_mut(&intersection.clip) {
            Some(group) => group.push(index),
            None => {
                self.by_edge.insert(intersection.clip, vec![index]);
            }
        };

        self.all.push(intersection);
        self
    }
}

#[derive(Debug, PartialEq, PartialOrd)]
struct PartialOrdKey<T>(T);

impl<T> Eq for PartialOrdKey<T> where T: PartialEq {}
impl<T> Ord for PartialOrdKey<T>
where
    T: PartialOrd,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap_or(Ordering::Less)
    }
}

impl<T> From<T> for PartialOrdKey<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}

#[derive(Debug)]
struct PartialOrdBTreeMap<K, V>(BTreeMap<PartialOrdKey<K>, V>);

impl<K, V> PartialOrdBTreeMap<K, V>
where
    K: PartialOrd,
{
    fn insert(&mut self, key: K, value: V) {
        self.0.insert(key.into(), value);
    }

    fn get(&self, key: K) -> Option<&V> {
        self.0.get(&key.into())
    }
}

impl<K, V> PartialOrdBTreeMap<K, V> {
    fn new() -> Self {
        Self(BTreeMap::new())
    }
}

#[derive(Debug)]
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

impl<T> ClipperGraph<T>
where
    T: PartialOrd + Signed + Float + Debug,
{
    fn with_intersections(mut self) -> Self {
        let intersections = Intersections::from(&self);
        let mut visited = PartialOrdBTreeMap::new();

        for (edge, mut indexes) in intersections.by_edge {
            let vertex = self.vertices[edge]
                .as_ref()
                .expect("edge vertex should exist")
                .point;

            indexes.sort_by(|&a, &b| {
                vertex
                    .distance(&intersections.all[a].point)
                    .partial_cmp(&vertex.distance(&intersections.all[b].point))
                    .unwrap_or(Ordering::Equal)
            });

            indexes
                .chunk_by(|&a, &b| intersections.all[a].point == intersections.all[b].point)
                .fold(edge, |previous, chunk| {
                    let point = intersections.all[chunk[0]].point;
                    let index = self.vertices.len();

                    visited.insert((edge, point), index);

                    let next = self.vertices[previous]
                        .as_ref()
                        .expect("previous vertex should exist")
                        .next;

                    self.vertices[previous]
                        .as_mut()
                        .expect("previous vertex should exist")
                        .next = index;

                    self.vertices[next]
                        .as_mut()
                        .expect("next vertex should exist")
                        .previous = index;

                    let siblings = chunk
                        .into_iter()
                        .map(|&index| {
                            if edge == intersections.all[index].clip {
                                intersections.all[index].subject
                            } else {
                                intersections.all[index].clip
                            }
                        })
                        .filter_map(|edge| visited.get((edge, point)))
                        .copied()
                        .inspect(|&sibling| {
                            self.vertices[sibling]
                                .as_mut()
                                .expect("sibling should exist")
                                .siblings
                                .push(index)
                        })
                        .collect();

                    self.vertices.push(Some(Vertex {
                        point,
                        next,
                        previous,
                        siblings,
                    }));

                    index
                });
        }

        self
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
                    siblings: Vec::new(),
                }));
            }
        }

        self
    }

    fn edges(&self, boundary: &Boundary) -> impl Iterator<Item = Edge<'_, T>> {
        let start = match boundary {
            Boundary::Clip(index) | Boundary::Subject(index) => *index,
        };

        EdgesIterator {
            graph: self,
            start,
            next: Some(start),
        }
    }

    fn position_where(&self, f: impl Fn(&Vertex<T>) -> bool) -> Option<usize> {
        self.vertices
            .iter()
            .enumerate()
            .filter_map(|(index, cell)| cell.as_ref().map(|vertex| (index, vertex)))
            .find(|(_, vertex)| f(vertex))
            .map(|(start, _)| start)
    }
}
