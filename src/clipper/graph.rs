use std::{cmp::Ordering, collections::BTreeMap, fmt::Debug};

use num_traits::{Float, Signed};

use crate::{point::Point, polygon::Segment, shape::Shape};

use super::{Role, vertex::Vertex};

/// The index of the first [`Vertex`] of a [`Polygon`] belonging to the clip or subject [`Shape`].
#[derive(Debug)]
enum Boundary {
    Clip(usize),
    Subject(usize),
}

impl From<&Boundary> for Role {
    fn from(boundary: &Boundary) -> Self {
        match boundary {
            Boundary::Subject(_) => Role::Subject,
            Boundary::Clip(_) => Role::Clip,
        }
    }
}

impl Boundary {
    fn is_subject(&self) -> bool {
        matches!(self, Boundary::Subject(_))
    }

    fn role(&self) -> Role {
        self.into()
    }
}

#[derive(Debug)]
struct Edge<'a, T> {
    segment: Segment<'a, T>,
    index: usize,
}

struct EdgesIterator<'a, T> {
    graph: &'a Graph<T>,
    next: Option<usize>,
    start: usize,
}

impl<'a, T> Iterator for EdgesIterator<'a, T> {
    type Item = Edge<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.next?;
        let vertex = self.graph.vertices[current].as_ref()?;
        self.next = (vertex.next != self.start).then_some(vertex.next);

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

impl<T> From<&GraphBuilder<T>> for Intersections<T>
where
    T: Signed + Float,
{
    fn from(builder: &GraphBuilder<T>) -> Self {
        let mut intersections = Self::default();
        for subject_polygon in builder.polygons.iter().filter(|p| p.is_subject()) {
            for clip_polygon in builder.polygons.iter().filter(|p| !p.is_subject()) {
                for subject_edge in builder.graph.edges(subject_polygon) {
                    for clip_edge in builder.graph.edges(clip_polygon) {
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

#[derive(Debug, PartialEq)]
struct PartialOrdKey<T>(T);

impl<T> PartialOrd for PartialOrdKey<T>
where
    T: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Eq for PartialOrdKey<T> where T: PartialEq {}
impl<T> Ord for PartialOrdKey<T>
where
    T: PartialOrd,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.partial_cmp(&other.0).unwrap_or(Ordering::Less)
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
pub(super) struct Graph<T> {
    pub(super) vertices: Vec<Option<Vertex<T>>>,
}

impl<T> Default for Graph<T> {
    fn default() -> Self {
        Self {
            vertices: Default::default(),
        }
    }
}

impl<T> Graph<T> {
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

    pub(super) fn position_where(&self, f: impl Fn(&Vertex<T>) -> bool) -> Option<usize> {
        self.vertices
            .iter()
            .enumerate()
            .filter_map(|(index, cell)| cell.as_ref().map(|vertex| (index, vertex)))
            .find(|(_, vertex)| f(vertex))
            .map(|(start, _)| start)
    }
}

#[derive(Debug)]
pub(super) struct GraphBuilder<T> {
    graph: Graph<T>,
    polygons: Vec<Boundary>,
}

impl<T> Default for GraphBuilder<T> {
    fn default() -> Self {
        Self {
            graph: Default::default(),
            polygons: Default::default(),
        }
    }
}

impl<T> GraphBuilder<T>
where
    T: PartialOrd + Signed + Float + Debug,
{
    pub(super) fn build(mut self) -> Graph<T> {
        let intersections = Intersections::from(&self);
        let mut visited = PartialOrdBTreeMap::new();

        for (edge, mut indexes) in intersections.by_edge {
            let &Vertex {
                point, role, next, ..
            } = self.graph.vertices[edge]
                .as_ref()
                .expect("edge vertex should exist");

            let endpoint = self.graph.vertices[next]
                .as_ref()
                .expect("edge endpoint should exist")
                .point;

            indexes.sort_by(|&a, &b| {
                point
                    .distance(&intersections.all[a].point)
                    .partial_cmp(&point.distance(&intersections.all[b].point))
                    .unwrap_or(Ordering::Equal)
            });

            indexes
                .chunk_by(|&a, &b| intersections.all[a].point == intersections.all[b].point)
                .fold(edge, |previous, chunk| {
                    let intersection_point = intersections.all[chunk[0]].point;
                    let index = if intersection_point == point {
                        edge
                    } else if intersection_point == endpoint {
                        next
                    } else {
                        self.graph.vertices.len()
                    };

                    visited.insert((edge, intersection_point), index);

                    let siblings = chunk
                        .iter()
                        .map(|&index| {
                            if edge == intersections.all[index].clip {
                                intersections.all[index].subject
                            } else {
                                intersections.all[index].clip
                            }
                        })
                        .filter_map(|edge| visited.get((edge, intersection_point)))
                        .copied()
                        .inspect(|&sibling| {
                            self.graph.vertices[sibling]
                                .as_mut()
                                .expect("sibling should exist")
                                .siblings
                                .push(index)
                        })
                        .collect();

                    if [point, endpoint].contains(&intersection_point) {
                        self.graph.vertices[index]
                            .as_mut()
                            .expect("endpoint vertex should exists")
                            .siblings
                            .extend(siblings);
                    } else {
                        let next = self.graph.vertices[previous]
                            .as_ref()
                            .expect("previous vertex should exist")
                            .next;

                        self.graph.vertices[previous]
                            .as_mut()
                            .expect("previous vertex should exist")
                            .next = index;

                        self.graph.vertices[next]
                            .as_mut()
                            .expect("next vertex should exist")
                            .previous = index;

                        self.graph.vertices.push(Some(Vertex {
                            point: intersection_point,
                            role,
                            next,
                            previous,
                            siblings,
                        }));
                    }

                    index
                });
        }

        self.graph
    }
}

impl<T> GraphBuilder<T> {
    pub(super) fn with_subject(self, shape: Shape<T>) -> Self {
        self.with_shape(shape, Boundary::Subject)
    }

    pub(super) fn with_clip(self, shape: Shape<T>) -> Self {
        self.with_shape(shape, Boundary::Clip)
    }

    fn with_shape(mut self, shape: Shape<T>, boundary: impl Fn(usize) -> Boundary) -> Self {
        self.graph.vertices.reserve(shape.total_vertices());
        self.polygons.reserve(shape.polygons.len());

        for polygon in shape.polygons {
            let offset = self.graph.vertices.len();
            let boundary = boundary(offset);
            let role = boundary.role();

            self.polygons.push(boundary);

            let total_vertices = polygon.vertices.len();
            for (mut index, point) in polygon.vertices.into_iter().enumerate() {
                index += total_vertices;

                self.graph.vertices.push(Some(Vertex {
                    point,
                    role,
                    next: offset + ((index + 1) % total_vertices),
                    previous: offset + ((index - 1) % total_vertices),
                    siblings: Vec::new(),
                }));
            }
        }

        self
    }
}
