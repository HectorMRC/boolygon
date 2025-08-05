use std::{cmp::Ordering, collections::{BTreeMap, BTreeSet}};

use crate::{Edge, Geometry, IsClose, Shape, Vertex};

/// The role of a boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BoundaryRole {
    /// The boundary belongs to the subject shape.
    Subject,
    /// The boundary belongs to the clip shape.
    Clip,
}

impl BoundaryRole {
    /// Returns true if, and only if, the boundary belongs to the subject shape.
    pub(crate) fn is_subject(&self) -> bool {
        matches!(self, BoundaryRole::Subject)
    }
}

/// The kind of intersection being represented by a [`Node`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Intersection {
    /// The shape is entering into the oposite one.
    Entry,
    /// The shape is exiting from the oposite one.
    Exit,
}

impl Intersection {
    /// Returns the oposite intersection type.
    pub(crate) fn oposite(self) -> Self {
        match self {
            Self::Entry => Self::Exit,
            Self::Exit => Self::Entry,
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
    /// Being this node an intersection, determines whether the boundary is entering or exiting the opposite one.
    pub(crate) intersection: Option<Intersection>,
    /// Vertices from the oposite shape located at the same point.
    pub(crate) siblings: BTreeSet<usize>,
}

impl<T> Node<T>
where
    T: Geometry,
{
    /// Returns true if, and only if, this node has siblings.
    pub(crate) fn is_intersection(&self) -> bool {
        !self.siblings.is_empty()
    }
}

/// A boundary in the [`Graph`].
pub(crate) struct Boundary {
    /// The total amount of intersection in this boundary.
    pub(crate) intersection_count: usize,
    /// The index in the [`Graph`] at which this boundary begins.
    pub(crate) start: usize,
    /// The role of this boundary
    pub(crate) role: BoundaryRole
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
        intersections.boundary_count.into_iter().for_each(|(position, count)| self.boundaries[position].intersection_count = count);
        
        let mut visited = PartialOrdBTreeMap::new();
        for (edge, mut intersection_indexes) in intersections.by_edge {
            let &Node {
                vertex: first,
                boundary,
                next,
                ..
            } = &self.nodes[edge];

            let last = self.nodes[next]
                .vertex;

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
                            self.nodes[sibling]
                                .siblings
                                .insert(index);
                        })
                        .collect();

                    if [first, last].contains(&intersection_point) {
                        // If the intersection point is any of the endpoints of the edge, do not
                        // create any node in the graph. Instead finds that endpoint and update
                        // the siblings list.
                        self.nodes[index]
                            .siblings
                            .extend(siblings);
                    } else {
                        // Cut the edge and register the new vertex.
                        let next = self.nodes[previous]
                            .next;

                        self.nodes[previous]
                            .next = index;

                        self.nodes[next]
                            .previous = index;

                        self.nodes.push(Node {
                            vertex: intersection_point,
                            boundary,
                            previous,
                            next,
                            intersection: None,
                            siblings,
                        });
                    }

                    index
                });
        }

        self
    }

    /// Returns the graph.
    pub(crate) fn build(self) -> Graph<T> {
        let builder = self
            .with_intersections()
            .with_statuses();

        Graph { 
            nodes: builder.nodes.into_iter().map(Some).collect(),
            boundaries: builder.boundaries
        }
    }
}

impl<T> GraphBuilder<'_, T, &Shape<T>, &Shape<T>>
where
    T: Geometry,
{
    /// Returns a record of all the intersections between the edges of the subject and clip shapes.
    fn intersections(&self) -> EdgeIntersections<T> {
        let edges_of = |boundary: &Boundary| {
            Edges {
                nodes: &self.nodes,
                start: boundary.start,
                next: None,
            }
        };

        let mut intersections = EdgeIntersections::default();
        for (subject_position, subject_boundary) in self.boundaries.iter().enumerate().filter(|(_, boundary)| boundary.role.is_subject()) {
            for (clip_position, clip_boundary) in self.boundaries.iter().enumerate().filter(|(_, boundary)| !boundary.role.is_subject()) {
                for (subject_index, subject_edge) in edges_of(subject_boundary) {
                    for (clip_index, clip_edge) in edges_of(clip_boundary) {
                        if let Some(intersection) = subject_edge
                            .intersection(&clip_edge, self.tolerance)
                        {
                            intersections = intersections.with_intersection(EdgeIntersection {
                                vertex: intersection,
                                subject: subject_index,
                                clip: clip_index,
                            });

                            intersections.boundary_count.entry(subject_position).and_modify(|count| *count += 1).or_insert(1);
                            
                            intersections.boundary_count.entry(clip_position).and_modify(|count| *count += 1).or_insert(1);
                        }
                    }
                }
            }
        }

        intersections
    }

    /// Computes the [`Status`] of each intersection [`Node`] in the graph.
    fn with_statuses(mut self) -> Self {
        for boundary in &self.boundaries {
            let base_status = self.base_status(boundary);

            NodesMut {
                nodes: &mut self.nodes,
                start: boundary.start,
                next: None,
            }
            .filter(|node| node.is_intersection())
            .fold(base_status, |status, node| {
                node.intersection = Some(status);
                status.oposite()
            });
        }

        self
    }

    /// Returns the base [`Status`] of the given boundary. 
    fn base_status(&self, boundary: &Boundary) -> Intersection {
        let start = &self.nodes[boundary.start];

        let vertex = if start.is_intersection() {
            let previous = &self.nodes[start.previous];

            if previous.is_intersection() {
                &T::Edge::new(&previous.vertex, &start.vertex).midpoint()
            } else {
                &previous.vertex
            }
        } else {
            &start.vertex
        };

        if match boundary.role {
            BoundaryRole::Subject => self.clip.contains(vertex, self.tolerance),
            BoundaryRole::Clip => self.subject.contains(vertex, self.tolerance),
        } {
            Intersection::Exit
        } else {
            Intersection::Entry
        }
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
    fn with_shape(mut self, shape: Shape<T>, role: BoundaryRole) -> Self {
        self.nodes.reserve(shape.total_vertices());
        self.boundaries.reserve(shape.boundaries.len());

        for boundary in shape.boundaries {
            let offset = self.nodes.len();
            self.boundaries.push(Boundary { intersection_count: 0, start: offset, role });

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
                    siblings: BTreeSet::new(),
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
    boundary_count: BTreeMap<usize, usize>
}

impl<T> Default for EdgeIntersections<T>
where
    T: Geometry,
{
    fn default() -> Self {
        Self {
            all: Default::default(),
            by_edge: Default::default(),
            boundary_count: Default::default(),
        }
    }
}

impl<T> EdgeIntersections<T>
where
    T: Geometry,
{
    fn with_intersection(mut self, intersection: EdgeIntersection<T>) -> Self {
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

struct NodesMut<'a, T>
where
    T: Geometry,
{
    nodes: &'a mut Vec<Node<T>>,
    next: Option<usize>,
    start: usize,
}

impl<'a, 'b, T> Iterator for &'b mut NodesMut<'a, T>
where
    T: Geometry,
{
    type Item = &'b mut Node<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(current) = self.next && current == self.start {
            return None;
        }

        let current = self.next.unwrap_or(self.start);
        let node = &mut self.nodes[current];
        self.next = Some(node.next);

        unsafe { Some(std::mem::transmute(node)) }
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
        if let Some(current) = self.next && current == self.start {
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