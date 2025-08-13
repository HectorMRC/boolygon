use std::{
    cmp::Ordering,
    collections::{BTreeMap, BTreeSet},
};

use crate::{either::Either, Edge, Geometry, IsClose, Shape, Vertex};

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

    fn position(&self) -> usize {
        match self {
            BoundaryRole::Subject(position) | BoundaryRole::Clip(position) => *position,
        }
    }
}

/// A boundary in the [`Graph`].
pub(crate) struct Boundary {
    /// The amount of intersections in this boundary.
    pub(crate) intersection_count: usize,
    /// The index in the [`Graph`] at which this boundary begins.
    pub(crate) start: usize,
    /// The role of this boundary
    pub(crate) role: BoundaryRole,
}

/// The kind of intersection being represented by a [`Node`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum IntersectionKind {
    /// The shape is entering into the oposite one.
    Entry,
    /// The shape is exiting from the oposite one.
    Exit,
}

impl IntersectionKind {
    /// Returns the oposite intersection type.
    pub(crate) fn oposite(self) -> Self {
        match self {
            Self::Entry => Self::Exit,
            Self::Exit => Self::Entry,
        }
    }
}

/// The intersection data of a [`Node`].
#[derive(Debug, Default)]
pub(crate) struct Intersection {
    /// If true, this intersection is a vertex from this or the oposite shape.
    pub(crate) is_pseudo: bool,
    /// Whether the boundary is entering or exiting the opposite one.
    pub(crate) kind: Option<IntersectionKind>,
    /// Vertices from the oposite shape located at the same point.
    pub(crate) siblings: BTreeSet<usize>,
}

impl FromIterator<usize> for Intersection {
    fn from_iter<T: IntoIterator<Item = usize>>(iter: T) -> Self {
        Self {
            siblings: FromIterator::from_iter(iter),
            ..Default::default()
        }
    }
}

impl Intersection {
    /// Returns true if, and only if, this intersection has siblings.
    pub(crate) fn has_siblings(&self) -> bool {
        !self.siblings.is_empty()
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
    pub(crate) intersection: Intersection
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
        let mut visited = PartialOrdBTreeMap::new();
        for (edge, mut intersection_indexes) in intersections.by_edge {
            let &Node {
                vertex: first,
                boundary,
                next,
                ..
            } = &self.nodes[edge];

            let last = self.nodes[next].vertex;

            // Sorting the intersections by its distance to the edge starting point ensures each
            // intersection will "cut" the edge in order of appearance, preserving its original
            // direction.
            intersection_indexes.sort_by(|&a, &b| {
                first
                    .distance(&intersections.all[a].vertex)
                    .partial_cmp(&first.distance(&intersections.all[b].vertex))
                    .unwrap_or(Ordering::Equal)
            });

            intersection_indexes
                .chunk_by(|&a, &b| intersections.all[a].vertex == intersections.all[b].vertex)
                .fold(edge, |previous, chunk| {
                    let intersection_point = intersections.all[chunk[0]].vertex;

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
                        // Otherwise, the intersection point is a new point somewhere between of
                        // the endpoints of the edge.
                        self.nodes.len()
                    };

                    // Mark this intersection point as been visited by this edge. This will allow
                    // siblings from the oposite shape to know about its index in the graph.
                    visited.insert((edge, intersection_point), index);

                    // Count this intersection into the corresponding boundary.
                    self.boundaries[boundary.position()].intersection_count += 1;

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
                            self.nodes[sibling].intersection.siblings.insert(index);
                        })
                        .collect::<Vec<_>>();

                    if [first, last].contains(&intersection_point) {
                        // If the intersection point is any of the endpoints of the edge, do not
                        // create any node in the graph. Instead finds that endpoint and update
                        // the siblings list.
                        self.nodes[index].intersection.siblings.extend(siblings);
                        self.nodes[index].intersection.is_pseudo = true;
                    } else {
                        // Cut the edge and register the new vertex.
                        let next = self.nodes[previous].next;

                        self.nodes[previous].next = index;

                        self.nodes[next].previous = index;

                        self.nodes.push(Node {
                            vertex: intersection_point,
                            intersection: FromIterator::from_iter(siblings),
                            boundary,
                            previous,
                            next,
                        });
                    };

                    index
                });
        }

        self
    }

    /// Returns the graph.
    pub(crate) fn build(self) -> Graph<T> {
        let builder = self.with_intersections().with_statuses();

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
                                Either::Left(vertex) => {
                                    intersections.with_intersection(EdgeIntersection {
                                        vertex,
                                        subject: subject_index,
                                        clip: clip_index,
                                    })
                                }
                                Either::Right([first, second]) => {
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

    /// Returns the [`IntersectionKind`] corresponding to the [`Node`] at the given position.
    fn intersection_kind(&self, position: usize) -> IntersectionKind {
        let node = &self.nodes[position];
        let boundary = match &node.boundary {
            BoundaryRole::Subject(_) => self.clip,
            BoundaryRole::Clip(_) => self.subject,
        };

        let previous = if node.intersection.has_siblings() {
            let previous = &self.nodes[node.previous];
            &T::Edge::new(&previous.vertex, &node.vertex).midpoint()
        } else {
            &node.vertex
        };

        if boundary.contains(previous, self.tolerance) {
            IntersectionKind::Exit
        } else {
            IntersectionKind::Entry
        }
    }

    /// Returns true if, and only if, the given the [`Node`] at the given position is indeed an intersection.
    fn is_intersection(&self, position: usize) -> bool {
        let node = &self.nodes[position];
        let previous = &self.nodes[node.previous];
        let next = &self.nodes[node.next];

        if previous.intersection.is_pseudo && next.intersection.is_pseudo {
            return false;
        }

        let previous = T::Edge::new(&node.vertex, &previous.vertex).midpoint();
        let next = T::Edge::new(&node.vertex, &next.vertex).midpoint();
        let oposite = match node.boundary {
            BoundaryRole::Subject(_) => self.clip,
            BoundaryRole::Clip(_) => self.subject,
        };

        oposite.contains(&previous, self.tolerance) != oposite.contains(&next, self.tolerance)
    }

    /// Downgrades the [`Node`] at the given position from intersection to non-intersection.
    fn downgrade_intersection(&mut self, position: usize) {
        let node = &mut self.nodes[position];
        if !node.intersection.has_siblings() {
            return;
        }

        let boundary = &mut self.boundaries[node.boundary.position()];
        boundary.intersection_count = boundary
            .intersection_count
            .saturating_sub(if node.intersection.is_pseudo { 2 } else { 1 });

        node.intersection.kind.take();
        std::mem::replace(&mut self.nodes[position].intersection.siblings, Default::default())
            .into_iter()
            .for_each(|sibling| self.downgrade_intersection(sibling));
    }

    /// Computes the [`Status`] of each intersection [`Node`] in the graph.
    fn with_statuses(mut self) -> Self {
        for boundary in 0..self.boundaries.len() {
            let start = self.boundaries[boundary].start;

            let mut intersection_traversal = IntersectionSearch::new(start);
            let mut intersection_kind = self.intersection_kind(start);

            while let Some(node) = intersection_traversal.next(&self.nodes) {
                if self.nodes[node].intersection.is_pseudo && !self.is_intersection(node) {
                    self.downgrade_intersection(node);
                } else {
                    self.nodes[node].intersection.kind = Some(intersection_kind);
                    intersection_kind = intersection_kind.oposite();
                }
            }
        }

        self
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
                intersection_count: 0,
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
                    intersection: Default::default(),
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

struct IntersectionSearch {
    next: Option<usize>,
    start: usize,
}

impl IntersectionSearch {
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

        if !node.intersection.has_siblings() {
            return self.next(nodes);
        }

        Some(current)
    }
}

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
