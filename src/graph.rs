use std::{
    cmp::Ordering,
    collections::{BTreeMap, BTreeSet},
    fmt::Debug,
};

use crate::{
    node::{Node, Role},
    Edge, Geometry, IsClose, Shape, Vertex,
};

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
{
    type Item = EnumeratedEdge<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.next?;
        let node = self.graph.nodes[current].as_ref()?;
        self.next = (node.next != self.start).then_some(node.next);

        Some(EnumeratedEdge {
            edge: T::Edge::new(&node.vertex, &self.graph.nodes[node.next].as_ref()?.vertex),
            index: current,
        })
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
    pub(super) nodes: Vec<Option<Node<T>>>,
}

impl<T> Default for Graph<T>
where
    T: Geometry,
{
    fn default() -> Self {
        Self {
            nodes: Default::default(),
        }
    }
}

impl<T> Graph<T>
where
    T: Geometry,
{
    fn edges(&self, boundary: &Role) -> impl Iterator<Item = EnumeratedEdge<T>> {
        let start = match boundary {
            Role::Clip(index) | Role::Subject(index) => *index,
        };

        EnumeratedEdgesIterator {
            graph: self,
            start,
            next: Some(start),
        }
    }

    /// Returns the index of the first node in the graph satifying the given closure.
    pub(super) fn position_where(&self, f: impl Fn(&Node<T>) -> bool) -> Option<usize> {
        self.nodes
            .iter()
            .enumerate()
            .filter_map(|(index, slot)| slot.as_ref().map(|node| (index, node)))
            .find(|(_, node)| f(node))
            .map(|(start, _)| start)
    }

    /// Returns an iterator of indexes in the graph containing the successors of the given node.
    pub(super) fn successors(&self, node: &Node<T>) -> impl Iterator<Item = usize> {
        node.siblings
            .iter()
            .filter_map(|&sibling| self.nodes[sibling].as_ref())
            .map(|sibling| sibling.next)
            .chain([node.next])
    }
}

pub(super) struct GraphBuilder<'a, T>
where
    T: Geometry,
{
    graph: Graph<T>,
    boundaries: Vec<Role>,
    tolerance: &'a <T::Vertex as IsClose>::Tolerance,
}

impl<'a, T> GraphBuilder<'a, T>
where
    T: Geometry,
{
    pub(super) fn new(tolerance: &'a <T::Vertex as IsClose>::Tolerance) -> Self {
        Self {
            graph: Default::default(),
            boundaries: Default::default(),
            tolerance,
        }
    }
}

impl<T> GraphBuilder<'_, T>
where
    T: Geometry,
    T::Vertex: Copy + PartialEq + PartialOrd,
    <T::Vertex as Vertex>::Scalar: Copy + PartialOrd,
{
    pub(super) fn build(mut self) -> Graph<T> {
        let intersections = self.intersections();
        let mut visited = PartialOrdBTreeMap::new();

        for (edge, mut intersection_indexes) in intersections.by_edge {
            let &Node {
                vertex: first,
                role,
                next,
                ..
            } = self.graph.nodes[edge]
                .as_ref()
                .expect("edge vertex should exist");

            let last = self.graph.nodes[next]
                .as_ref()
                .expect("edge endpoint should exist")
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
                        // Otherwise, the intersection point is a new point somewhere between of the endpoints of the edge.
                        self.graph.nodes.len()
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
                            self.graph.nodes[sibling]
                                .as_mut()
                                .expect("sibling should exist")
                                .siblings
                                .insert(index);
                        })
                        .collect();

                    if [first, last].contains(&intersection_point) {
                        // If the intersection point is any of the endpoints of the edge, do not
                        // create any node in the graph. Instead finds that endpoint and update
                        // the siblings list.
                        self.graph.nodes[index]
                            .as_mut()
                            .expect("endpoint node should exists")
                            .siblings
                            .extend(siblings);
                    } else {
                        // Cut the edge and register the new vertex.
                        let next = self.graph.nodes[previous]
                            .as_ref()
                            .expect("previous node should exist")
                            .next;

                        self.graph.nodes[previous]
                            .as_mut()
                            .expect("previous node should exist")
                            .next = index;

                        self.graph.nodes[next]
                            .as_mut()
                            .expect("next node should exist")
                            .previous = index;

                        self.graph.nodes.push(Some(Node {
                            vertex: intersection_point,
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

impl<T> GraphBuilder<'_, T>
where
    T: Geometry,
{
    /// Returns a record of all the intersections between the edges of the subject and clip shapes.
    fn intersections(&self) -> EdgeIntersections<T> {
        let mut intersections = EdgeIntersections::default();
        for subject_boundary in self.boundaries.iter().filter(|p| p.is_subject()) {
            for clip_boundary in self.boundaries.iter().filter(|p| !p.is_subject()) {
                for subject_edge in self.graph.edges(subject_boundary) {
                    for clip_edge in self.graph.edges(clip_boundary) {
                        if let Some(intersection) = subject_edge
                            .edge
                            .intersection(&clip_edge.edge, self.tolerance)
                        {
                            intersections = intersections.with_intersection(EdgeIntersection {
                                vertex: intersection,
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

impl<T> GraphBuilder<'_, T>
where
    T: Geometry + IntoIterator<Item = T::Vertex>,
{
    pub(super) fn with_subject(self, shape: Shape<T>) -> Self {
        self.with_shape(shape, Role::Subject)
    }

    pub(super) fn with_clip(self, shape: Shape<T>) -> Self {
        self.with_shape(shape, Role::Clip)
    }

    fn with_shape(mut self, shape: Shape<T>, role: impl Fn(usize) -> Role) -> Self {
        self.graph.nodes.reserve(shape.total_vertices());
        self.boundaries.reserve(shape.boundaries.len());

        for boundary in shape.boundaries {
            let offset = self.graph.nodes.len();
            let role = role(offset);

            self.boundaries.push(role);

            let total_vertices = boundary.total_vertices();
            for (mut index, point) in boundary.into_iter().enumerate() {
                index += total_vertices;

                self.graph.nodes.push(Some(Node {
                    vertex: point,
                    role,
                    next: offset + ((index + 1) % total_vertices),
                    previous: offset + ((index - 1) % total_vertices),
                    siblings: BTreeSet::new(),
                }));
            }
        }

        self
    }
}
