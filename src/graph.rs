use std::{cmp::Ordering, collections::BTreeMap, fmt::Debug};

use crate::{
    vertex::{Role, Vertex},
    Edge, Geometry, IsClose, Metric, Secant, Shape, Tolerance,
};

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
struct EnumeratedEdge<'a, T>
where
    T: 'a + Geometry,
{
    edge: T::Edge<'a>,
    index: usize,
}

struct EnumeratedEdgesIterator<'a, T>
where
    T: Geometry,
{
    graph: &'a Graph<T>,
    next: Option<usize>,
    start: usize,
}

impl<'a, T> Iterator for EnumeratedEdgesIterator<'a, T>
where
    T: Geometry,
    for<'b> T::Edge<'b>: Edge<'b, Endpoint = T::Point>,
{
    type Item = EnumeratedEdge<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.next?;
        let vertex = self.graph.vertices[current].as_ref()?;
        self.next = (vertex.next != self.start).then_some(vertex.next);

        Some(EnumeratedEdge {
            edge: <T::Edge<'_> as Edge>::new(
                &vertex.point,
                &self.graph.vertices[vertex.next].as_ref()?.point,
            ),
            index: current,
        })
    }
}

/// The intersection between two edges.
#[derive(Debug)]
struct Intersection<T>
where
    T: Geometry,
{
    /// The [`Point`] of intersection between the edges started by subject and clip.
    point: T::Point,
    /// The index of the starting vertex in the subject edge involved in this intersection.
    subject: usize,
    /// The index of the starting vertex in the clip edge involved in this intersection.
    clip: usize,
}

/// All the intersections between the edges of a subject and clip [`Shape`]s.
struct Intersections<T>
where
    T: Geometry,
{
    all: Vec<Intersection<T>>,
    by_edge: BTreeMap<usize, Vec<usize>>,
}

impl<T> Default for Intersections<T>
where
    T: Geometry,
{
    fn default() -> Self {
        Self {
            all: Default::default(),
            by_edge: Default::default(),
        }
    }
}

impl<T> Intersections<T>
where
    T: Geometry,
{
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

pub(super) struct Graph<T>
where
    T: Geometry,
{
    pub(super) vertices: Vec<Option<Vertex<T>>>,
}

impl<T> Default for Graph<T>
where
    T: Geometry,
{
    fn default() -> Self {
        Self {
            vertices: Default::default(),
        }
    }
}

impl<T> Graph<T>
where
    T: Geometry,
    for<'a> T::Edge<'a>: Edge<'a, Endpoint = T::Point>,
{
    fn edges(&self, boundary: &Boundary) -> impl Iterator<Item = EnumeratedEdge<T>> {
        let start = match boundary {
            Boundary::Clip(index) | Boundary::Subject(index) => *index,
        };

        EnumeratedEdgesIterator {
            graph: self,
            start,
            next: Some(start),
        }
    }

    pub(super) fn position_where(&self, f: impl Fn(&Vertex<T>) -> bool) -> Option<usize> {
        self.vertices
            .iter()
            .enumerate()
            .filter_map(|(index, slot)| slot.as_ref().map(|vertex| (index, vertex)))
            .find(|(_, vertex)| f(vertex))
            .map(|(start, _)| start)
    }

    pub(super) fn purge(&mut self, index: usize) {
        self.vertices[index]
            .take()
            .iter()
            .map(|vertex| vertex.siblings.iter())
            .flatten()
            .for_each(|&sibling| self.purge(sibling));
    }
}

pub(super) struct GraphBuilder<T>
where
    T: Geometry,
{
    graph: Graph<T>,
    polygons: Vec<Boundary>,
    tolerance: Tolerance<<T::Point as Metric>::Scalar>,
}

impl<T> GraphBuilder<T>
where
    T: Geometry,
{
    pub(super) fn new(tolerance: Tolerance<<T::Point as Metric>::Scalar>) -> Self {
        Self {
            graph: Default::default(),
            polygons: Default::default(),
            tolerance,
        }
    }
}

impl<T> GraphBuilder<T>
where
    T: Geometry,
    for<'a> T::Edge<'a>: Secant<Point = T::Point> + Edge<'a, Endpoint = T::Point>,
    T::Point: Metric
        + IsClose<Tolerance = Tolerance<<T::Point as Metric>::Scalar>>
        + Copy
        + PartialEq
        + PartialOrd,
    <T::Point as Metric>::Scalar: Copy + PartialOrd,
{
    pub(super) fn build(mut self) -> Graph<T> {
        let intersections = self.intersections();
        let mut visited = PartialOrdBTreeMap::new();

        for (edge, mut intersection_indexes) in intersections.by_edge {
            let &Vertex {
                point: first,
                role,
                next,
                ..
            } = self.graph.vertices[edge]
                .as_ref()
                .expect("edge vertex should exist");

            let last = self.graph.vertices[next]
                .as_ref()
                .expect("edge endpoint should exist")
                .point;

            // Sorting the intersections by its distance to the edge starting point ensures each
            // intersection will "cut" the edge in order of appearance, preserving its original
            // direction.
            intersection_indexes.sort_by(|&a, &b| {
                first
                    .distance(&intersections.all[a].point)
                    .partial_cmp(&first.distance(&intersections.all[b].point))
                    .unwrap_or(Ordering::Equal)
            });

            intersection_indexes
                .chunk_by(|&a, &b| intersections.all[a].point == intersections.all[b].point)
                .fold(edge, |previous, chunk| {
                    let intersection_point = intersections.all[chunk[0]].point;

                    let index = if intersection_point == first {
                        // If the intersection point equals the edge starting point there is
                        // nothing to add into the graph. The index of this intersection in the
                        // graph is the index of the starting point.
                        edge
                    } else if intersection_point == last {
                        // Likewise, if the intersection point equals the edge final point there is
                        // nothing to add into the graph. The index of this intersection in the
                        // graph is the index of the final point.
                        next
                    } else {
                        // Otherwise, the intersection point is a new point somewhere between of the endpoints of the edge.
                        self.graph.vertices.len()
                    };

                    // Mark this intersection point as been visited by this edge. This will allow
                    // siblings from the oposite shape to know about its index in the graph.
                    visited.insert((edge, intersection_point), index);

                    let siblings = chunk
                        .iter()
                        .map(|&index| {
                            // Select the oposite shape of this intersection.
                            // e.g. If this edge belong to the clip shape, return the subject edge
                            // involved in the intersection.
                            if edge == intersections.all[index].clip {
                                intersections.all[index].subject
                            } else {
                                intersections.all[index].clip
                            }
                        })
                        .filter_map(|edge| {
                            // Get the index of the intersection point on that edge, if is already
                            // set.
                            visited.get((edge, intersection_point))
                        })
                        .copied()
                        .inspect(|&sibling| {
                            // While searching for siblings, update their siblings list by adding
                            // the index of this intersection.
                            self.graph.vertices[sibling]
                                .as_mut()
                                .expect("sibling should exist")
                                .siblings
                                .push(index)
                        })
                        .collect();

                    if [first, last].contains(&intersection_point) {
                        // If the intersection point is any of the endpoints of the edge, do not
                        // create any vertex in the graph. Instead finds that endpoint and update
                        // the siblings list.
                        self.graph.vertices[index]
                            .as_mut()
                            .expect("endpoint vertex should exists")
                            .siblings
                            .extend(siblings);
                    } else {
                        // Cut the edge and register the new vertex.
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

impl<T> GraphBuilder<T>
where
    T: Geometry,
    for<'a> T::Edge<'a>: Secant<Point = T::Point> + Edge<'a, Endpoint = T::Point>,
    <T::Point as Metric>::Scalar: Copy,
{
    /// Returns a record of all the intersections between the edges of the subject and clip shapes.
    fn intersections(&self) -> Intersections<T> {
        let mut intersections = Intersections::default();
        for subject_polygon in self.polygons.iter().filter(|p| p.is_subject()) {
            for clip_polygon in self.polygons.iter().filter(|p| !p.is_subject()) {
                for subject_edge in self.graph.edges(subject_polygon) {
                    for clip_edge in self.graph.edges(clip_polygon) {
                        if let Some(intersection) = subject_edge
                            .edge
                            .intersection(&clip_edge.edge, &self.tolerance)
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

impl<T> GraphBuilder<T>
where
    T: Geometry + IntoIterator<Item = T::Point>,
{
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

            let total_vertices = polygon.total_vertices();
            for (mut index, point) in polygon.into_iter().enumerate() {
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
