use std::{cmp::Ordering, collections::BTreeMap};

use crate::{Edge, Geometry, IsClose, MaybePair, Neighbors, Shape, Vertex};

/// The role of the boundary at the inner position in the [`Graph`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BoundaryRole {
    /// The boundary belongs to the subject shape.
    Subject(usize),
    /// The boundary belongs to the clip shape.
    Clip(usize),
}

impl BoundaryRole {
    /// Returns true if, and only if, the boundary belongs to the subject shape.
    pub(crate) fn is_subject(&self) -> bool {
        matches!(self, BoundaryRole::Subject(_))
    }

    pub(crate) fn position(&self) -> usize {
        match self {
            BoundaryRole::Subject(position) | BoundaryRole::Clip(position) => *position,
        }
    }
}

/// A boundary in the [`Graph`].
pub(crate) struct Boundary {
    /// If true, the boundary exists uninterrupted in the graph. Otherwise one or more nodes
    /// may be unavailable.
    pub(crate) healthy: bool,
    /// The index in the [`Graph`] at which this boundary begins.
    pub(crate) start: usize,
    /// The role of this boundary
    pub(crate) role: BoundaryRole,
}

/// The kind of intersection being represented by a [`Node`].
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum IntersectionKind {
    /// The shape is entering into the oposite one.
    Entry,
    /// The shape is exiting from the oposite one.
    Exit,
    /// The shape is just touching the oposite one.
    #[default]
    Vertex,
}

impl IntersectionKind {
    /// Returns true if, and only if, this is [`IntersectionKind::Vertex`].
    pub(crate) fn is_vertex(&self) -> bool {
        matches!(self, Self::Vertex)
    }
}

/// The intersection data of a [`Node`].
#[derive(Debug, Default)]
pub(crate) struct Intersection {
    /// Whether the boundary is entering or exiting the opposite one.
    pub(crate) kind: IntersectionKind,
    /// This intersection vertex on the oposite shape.
    pub(crate) sibling: usize,
}

impl Intersection {
    fn new(sibling: usize) -> Self {
        Intersection {
            sibling,
            ..Default::default()
        }
    }
}

/// A vertex and its metadata inside a graph.
#[derive(Debug)]
pub(crate) struct Node<T>
where
    T: Geometry,
{
    /// The vertex being represented by this node.
    pub(crate) vertex: T::Vertex,
    /// The boundary at which this node belongs.
    pub(crate) boundary: BoundaryRole,
    /// The index of the node previous to this one.
    pub(crate) previous: usize,
    /// The index of the node following this one.
    pub(crate) next: usize,
    /// The intersection info of this node.
    pub(crate) intersection: Option<Intersection>,
}

/// A graph of vertices and its relations.
pub(crate) struct Graph<T>
where
    T: Geometry,
{
    pub(crate) nodes: Vec<Option<Node<T>>>,
    pub(crate) boundaries: Vec<Boundary>,
}

impl<T> Default for Graph<T>
where
    T: Geometry,
{
    fn default() -> Self {
        Self {
            nodes: Default::default(),
            boundaries: Default::default(),
        }
    }
}

impl<T> Graph<T>
where
    T: Geometry,
{
    /// Takes the node at the given position from the graph, if exists.
    pub(crate) fn take(&mut self, position: usize) -> Option<Node<T>> {
        self.nodes[position]
            .take()
            .inspect(|node| self.boundaries[node.boundary.position()].healthy = false)
    }
}

/// Marker for yet undefined generic parameters.
pub(crate) struct Unknown;

/// A [`Graph`] builder.
pub(crate) struct GraphBuilder<'a, T, S, C>
where
    T: Geometry,
{
    nodes: Vec<Node<T>>,
    boundaries: Vec<Boundary>,
    tolerance: &'a <T::Vertex as IsClose>::Tolerance,
    subject: S,
    clip: C,
}

impl<'a, T> GraphBuilder<'a, T, Unknown, Unknown>
where
    T: Geometry,
{
    pub(crate) fn new(tolerance: &'a <T::Vertex as IsClose>::Tolerance) -> Self {
        Self {
            nodes: Default::default(),
            boundaries: Default::default(),
            tolerance,
            subject: Unknown,
            clip: Unknown,
        }
    }
}

impl<T> GraphBuilder<'_, T, &Shape<T>, &Shape<T>>
where
    T: Geometry,
    T::Vertex: Copy + PartialOrd,
    <T::Vertex as Vertex>::Scalar: PartialOrd,
{
    /// Populates the graph with all the intersections.
    fn with_intersections(mut self) -> Self {
        let intersections = self.intersections();
        let mut visited = PartialOrdBTreeMap::<_, usize>::new();
        for (current, mut intersection_indexes) in intersections.by_edge {
            let &Node {
                vertex: first,
                boundary,
                next,
                ..
            } = &self.nodes[current];

            let last = self.nodes[next].vertex;

            intersection_indexes.sort_by(|&a, &b| {
                first
                    .distance(&intersections.all[a].vertex)
                    .partial_cmp(&first.distance(&intersections.all[b].vertex))
                    .unwrap_or(Ordering::Equal)
            });

            intersection_indexes
                .chunk_by(|&a, &b| intersections.all[a].vertex == intersections.all[b].vertex)
                .fold(current, |previous, chunk| {
                    let intersection_point = intersections.all[chunk[0]].vertex;

                    if intersection_point == first {
                        // If intersection_point == first then there is another edge in which
                        // intersection_point == last. Discard one endpoint to avoid processing
                        // the same intersection twice.
                        return current;
                    }

                    let index = if intersection_point == last {
                        next
                    } else {
                        self.nodes.len()
                    };

                    let intersection = visited
                        .get(intersection_point)
                        .copied()
                        .inspect(|sibling| {
                            self.nodes[*sibling].intersection = Some(Intersection::new(index));
                        })
                        .map(Intersection::new);

                    if index == next {
                        self.nodes[index].intersection = intersection;
                    } else {
                        let next = self.nodes[previous].next;
                        self.nodes[previous].next = index;
                        self.nodes[next].previous = index;

                        self.nodes.push(Node {
                            vertex: intersection_point,
                            intersection,
                            boundary,
                            previous,
                            next,
                        });
                    };

                    visited.insert(intersection_point, index);
                    index
                });
        }

        for boundary in 0..self.boundaries.len() {
            let start = self.boundaries[boundary].start;

            let mut traversal = Traverse::new(start);
            while let Some(position) = traversal.next(&self.nodes) {
                self.nodes[position].intersection =
                    self.nodes[position]
                        .intersection
                        .take()
                        .map(|intersection| {
                            let node = &self.nodes[position];
                            let sibling = &self.nodes[intersection.sibling];

                            Intersection {
                                kind: T::Edge::intersection_kind(
                                    &node.vertex,
                                    Neighbors {
                                        tail: &self.nodes[node.previous].vertex,
                                        head: &self.nodes[node.next].vertex,
                                    },
                                    Neighbors {
                                        tail: &self.nodes[sibling.previous].vertex,
                                        head: &self.nodes[sibling.next].vertex,
                                    },
                                    self.tolerance,
                                ),
                                ..intersection
                            }
                        });
            }
        }

        self
    }

    /// Returns the graph.
    pub(crate) fn build(self) -> Graph<T> {
        let builder = self.with_intersections();

        Graph {
            nodes: builder.nodes.into_iter().map(Some).collect(),
            boundaries: builder.boundaries,
        }
    }
}

impl<T> GraphBuilder<'_, T, &Shape<T>, &Shape<T>>
where
    T: Geometry,
{
    /// Returns a record of all the intersections between the edges of the subject and clip shapes.
    fn intersections(&self) -> EdgeIntersections<T> {
        let edges_of = |boundary: &Boundary| Edges {
            nodes: &self.nodes,
            start: boundary.start,
            next: None,
        };

        let mut intersections = EdgeIntersections::default();
        for subject_boundary in self
            .boundaries
            .iter()
            .filter(|boundary| boundary.role.is_subject())
        {
            for clip_boundary in self
                .boundaries
                .iter()
                .filter(|boundary| !boundary.role.is_subject())
            {
                for (subject_index, subject_edge) in edges_of(subject_boundary) {
                    for (clip_index, clip_edge) in edges_of(clip_boundary) {
                        if let Some(intersection) =
                            subject_edge.intersection(&clip_edge, self.tolerance)
                        {
                            intersections = match intersection {
                                MaybePair::Single(vertex) => {
                                    intersections.with_intersection(EdgeIntersection {
                                        vertex,
                                        subject: subject_index,
                                        clip: clip_index,
                                    })
                                }
                                MaybePair::Pair([first, second]) => {
                                    let intersection = EdgeIntersection {
                                        vertex: first,
                                        subject: subject_index,
                                        clip: clip_index,
                                    };

                                    intersections
                                        .with_intersection(EdgeIntersection { ..intersection })
                                        .with_intersection(EdgeIntersection {
                                            vertex: second,
                                            ..intersection
                                        })
                                }
                            };
                        };
                    }
                }
            }
        }

        intersections
    }
}

impl<'a, T, S, C> GraphBuilder<'a, T, S, C>
where
    T: Geometry + Clone + IntoIterator<Item = T::Vertex>,
{
    pub(crate) fn with_subject(
        self,
        subject: &'a Shape<T>,
    ) -> GraphBuilder<'a, T, &'a Shape<T>, C> {
        GraphBuilder {
            nodes: self.nodes,
            boundaries: self.boundaries,
            tolerance: self.tolerance,
            clip: self.clip,
            subject,
        }
        .with_shape(subject.clone(), BoundaryRole::Subject)
    }
}

impl<'a, T, S, C> GraphBuilder<'a, T, S, C>
where
    T: Geometry + Clone + IntoIterator<Item = T::Vertex>,
{
    pub(crate) fn with_clip(self, clip: &'a Shape<T>) -> GraphBuilder<'a, T, S, &'a Shape<T>> {
        GraphBuilder {
            nodes: self.nodes,
            boundaries: self.boundaries,
            tolerance: self.tolerance,
            subject: self.subject,
            clip,
        }
        .with_shape(clip.clone(), BoundaryRole::Clip)
    }
}

impl<T, S, C> GraphBuilder<'_, T, S, C>
where
    T: Geometry + IntoIterator<Item = T::Vertex>,
{
    fn with_shape(mut self, shape: Shape<T>, role: impl Fn(usize) -> BoundaryRole) -> Self {
        self.nodes.reserve(shape.total_vertices());
        self.boundaries.reserve(shape.boundaries.len());

        for boundary in shape.boundaries {
            let offset = self.nodes.len();
            let role = role(self.boundaries.len());
            self.boundaries.push(Boundary {
                healthy: true,
                start: offset,
                role,
            });

            let total_vertices = boundary.total_vertices();
            for (mut index, point) in boundary.into_iter().enumerate() {
                // Avoid usize overflow when index == 0.
                index += total_vertices;

                self.nodes.push(Node {
                    vertex: point,
                    boundary: role,
                    previous: offset + ((index - 1) % total_vertices),
                    next: offset + ((index + 1) % total_vertices),
                    intersection: None,
                });
            }
        }

        self
    }
}

/// The intersection between two edges.
#[derive(Debug)]
struct EdgeIntersection<T>
where
    T: Geometry,
{
    /// The vertex of intersection between the edges started by subject and clip.
    vertex: T::Vertex,
    /// The index of the starting vertex in the subject edge involved in this intersection.
    subject: usize,
    /// The index of the starting vertex in the clip edge involved in this intersection.
    clip: usize,
}

/// All the intersections between the edges of a subject and clip [`Shape`]s.
struct EdgeIntersections<T>
where
    T: Geometry,
{
    all: Vec<EdgeIntersection<T>>,
    by_edge: BTreeMap<usize, Vec<usize>>,
}

impl<T> Default for EdgeIntersections<T>
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

impl<T> EdgeIntersections<T>
where
    T: Geometry,
{
    fn with_intersection(mut self, intersection: EdgeIntersection<T>) -> Self {
        let index = self.all.len();

        self.by_edge
            .entry(intersection.subject)
            .and_modify(|group| group.push(index))
            .or_insert(vec![index]);

        self.by_edge
            .entry(intersection.clip)
            .and_modify(|group| group.push(index))
            .or_insert(vec![index]);

        self.all.push(intersection);
        self
    }
}

/// Traverses the boundary starting at the given position.
struct Traverse {
    next: Option<usize>,
    start: usize,
}

impl Traverse {
    fn new(start: usize) -> Self {
        Self { next: None, start }
    }

    fn next<T>(&mut self, nodes: &[Node<T>]) -> Option<usize>
    where
        T: Geometry,
    {
        if let Some(current) = self.next
            && current == self.start
        {
            return None;
        }

        let current = self.next.unwrap_or(self.start);
        let node = &nodes[current];
        self.next = Some(node.next);

        Some(current)
    }
}

/// Iteratos over all the edges in the boundary starting at the given position.
struct Edges<'a, T>
where
    T: Geometry,
{
    nodes: &'a Vec<Node<T>>,
    next: Option<usize>,
    start: usize,
}

impl<'a, T> Iterator for Edges<'a, T>
where
    T: Geometry,
{
    type Item = (usize, T::Edge<'a>);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(current) = self.next
            && current == self.start
        {
            return None;
        }

        let current = self.next.unwrap_or(self.start);
        let node = &self.nodes[current];
        self.next = Some(node.next);

        Some((
            current,
            T::Edge::new(&node.vertex, &self.nodes[node.next].vertex),
        ))
    }
}

/// The key for the [`PartialOrdBTreeMap`].
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

/// A [`BTreeMap`] that accepts keys that only implements [`PartialOrd`].
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
