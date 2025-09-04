mod btree;
mod edges;
mod intersection;

use std::cmp::Ordering;

use crate::{Corner, Neighbors, graph::{Boundary, Graph, Intersection, Node, Role}, Edge, Geometry, IsClose, Shape, Vertex};

use self::intersection::{EdgeIntersections};
use self::btree::{PartialOrdBTreeMap};
use self::edges::{LocateEdges};

/// Marker for yet undefined generic parameters.
pub(crate) struct Unknown;

pub(crate) struct GraphBuilder<'a, T, S, C>
where
    T: Geometry,
{
    /// The vertices in the graph.
    /// This include all the vertices from the subject and clip shapes, plus intersection.
    pub(super) vertices: Vec<Node<T>>,
    /// The boundaries in the graph.
    pub(super) boundaries: Vec<Boundary>,
    /// The shape being clipped.
    pub(super) subject: S,
    /// The shape clipping the subject.
    pub(super) clip: C,
    /// The tolerance to use during building.
    pub(super) tolerance: &'a <T::Vertex as IsClose>::Tolerance,
}

impl<'a, T, C> GraphBuilder<'a, T, Unknown, C> 
where
    T: Geometry,
    for<'b> &'b T: IntoIterator<Item = &'b T::Vertex>,
    T::Vertex: Copy,
{
    /// Sets the subject [`Shape`] into the graph.
    pub(crate) fn with_subject(self, subject: &'a Shape<T>) -> GraphBuilder<'a, T, &'a Shape<T>, C> {
        let builder = self.with_shape(Role::Subject, &subject);

        GraphBuilder {
            vertices: builder.vertices,
            boundaries: builder.boundaries,
            clip: builder.clip,
            subject,
            tolerance: builder.tolerance,
        }
    }
}

impl<'a, T, S> GraphBuilder<'a, T, S, Unknown> 
where
    T: Geometry,
    for<'b> &'b T: IntoIterator<Item = &'b T::Vertex>,
    T::Vertex: Copy,
{
    /// Sets the clipper [`Shape`] into the graph.
    pub(crate) fn with_clip(self, clip: &'a Shape<T>) -> GraphBuilder<'a, T, S, &'a Shape<T>> {
        let builder = self.with_shape(Role::Clip, &clip);

        GraphBuilder {
            vertices: builder.vertices,
            boundaries: builder.boundaries,
            subject: builder.subject,
            clip,
            tolerance: builder.tolerance,
        }
    }
}

impl<T, S, C> GraphBuilder<'_, T, S, C>
where
    T: Geometry,
    for<'a> &'a T: IntoIterator<Item = &'a T::Vertex>,
    T::Vertex: Copy,
{
    fn with_shape(mut self, role: Role, shape: &Shape<T>) -> Self {
        self.boundaries.reserve(shape.boundaries.len());
        self.vertices.reserve(shape.total_vertices());

        for boundary in shape.boundaries.iter() {
            let vertices_base = self.vertices.len();
            let boundary_position = self.boundaries.len();

            self.boundaries.push(Boundary {
                entrypoint: vertices_base,
                visited: false,
                role,
            });

            let total_vertices = boundary.total_vertices();
            for (position, &vertex) in boundary.into_iter().enumerate() {
                let position = position + total_vertices;

                self.vertices.push(Node {
                    vertex,
                    boundary: boundary_position,
                    previous: vertices_base + ((position - 1) % total_vertices),
                    next: vertices_base + ((position + 1) % total_vertices),
                    intersection: None,
                    visited: false,
                });
            }
        }

        self
    }
}

impl<T> GraphBuilder<'_, T, &Shape<T>, &Shape<T>>
where
    T: Geometry,
{
    /// Returns an interator over the edges of the given boundary.
    pub(super) fn edges(&self, boundary: &Boundary) -> LocateEdges<'_, T> {
        LocateEdges {
            vertices: &self.vertices,
            start: boundary.entrypoint,
            next: None,
        }
    }
}

impl<T> GraphBuilder<'_, T, &Shape<T>, &Shape<T>>
where
    T: Geometry,
    for<'a> &'a T: IntoIterator<Item = &'a T::Vertex>,
    for<'a> T::Edge<'a>: Edge<'a>,
    T::Vertex: Copy + PartialEq + PartialOrd,
    <T::Vertex as Vertex>::Scalar: PartialOrd,
{
    pub(crate) fn build(mut self) -> Graph<T> {
        let intersections = EdgeIntersections::from(&self);
        let mut visited = PartialOrdBTreeMap::<_, usize>::new();
        
        for (current, mut intersection_indexes) in intersections.by_edge {
            let &Node {
                vertex: first,
                boundary,
                next,
                ..
            } = &self.vertices[current];

            let last = self.vertices[next].vertex;

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
                        self.vertices.len()
                    };

                    let intersection = visited
                        .get(intersection_point)
                        .copied()
                        .inspect(|sibling| {
                            self.vertices[*sibling].intersection = Some(Intersection::new(index));
                        })
                        .map(Intersection::new);

                    if index == next {
                        self.vertices[index].intersection = intersection;
                    } else {
                        let next = self.vertices[previous].next;
                        self.vertices[previous].next = index;
                        self.vertices[next].previous = index;

                        self.vertices.push(Node {
                            vertex: intersection_point,
                            intersection,
                            boundary,
                            previous,
                            next,
                            visited: false,
                        });
                    };

                    visited.insert(intersection_point, index);
                    index
                });
        }

        for position in 0..self.vertices.len() {
            if let Some(intersection) = self.vertices[position]
                    .intersection
                    .take()
            {
                let node = &self.vertices[position];
                let sibling = &self.vertices[intersection.sibling];

                let intersection = Intersection {
                    event: T::Edge::event(
                        Corner{
                            vertex: &node.vertex,
                            neighbors: Neighbors { 
                                tail: &self.vertices[node.previous].vertex,
                                head: &self.vertices[node.next].vertex,
                            },
                            role: self.boundaries[node.boundary].role,
                            intersection: Some(crate::Intersection {
                                event: None,
                                neighbors: Neighbors {
                                    tail: &self.vertices[sibling.previous].vertex,
                                    head: &self.vertices[sibling.next].vertex,
                                },
                            })
                        },         
                        &self.tolerance,
                    ),
                    ..intersection
                };

                self.vertices[position].intersection = Some(intersection);
            }
        }

        Graph { 
            vertices: self.vertices, 
            boundaries: self.boundaries, 
        }
    }
}
