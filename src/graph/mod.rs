mod builder;

pub(crate) use self::builder::{GraphBuilder, Unknown};

use crate::{Corner, Event, Geometry, IsClose, Neighbors, Role};

/// A boundary in the [`Graph`].
pub(crate) struct Boundary {
    /// An arbitrary vertex in the [`Graph`] belonging to this boundary.
    pub entrypoint: usize,
    /// If true, at least one vertex in this boundary has been taken by the clipping operation.
    pub(crate) visited: bool,
    /// The role of this boundary.
    pub(crate) role: Role,
}

/// The intersection data of a [`Vertex`].
#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct Intersection {
    /// Whether the boundary of this intersection is entering or exiting the other.
    pub(crate) event: Option<Event>,
    /// The position in the graph of the complementary vertex of this intersection.
    pub(crate) sibling: usize,
}

impl Intersection {
    pub(crate) fn new(sibling: usize) -> Self {
        Intersection {
            sibling,
            ..Default::default()
        }
    }
}

/// A node in the [`Graph`].
#[derive(Debug)]
pub(crate) struct Node<T>
where
    T: Geometry,
{
    /// The vertex being represented by this node.
    pub vertex: T::Vertex,
    /// The position in the graph of the corresponding boundary.   
    pub boundary: usize,
    /// The position in the graph of the previous vertex.
    pub previous: usize,
    /// The position in the graph of the next vertex.
    pub next: usize,
    /// The intersection info at this vertex, if any.
    pub(crate) intersection: Option<Intersection>,
    /// If true, this vertex has already been taken by the clipping operation.
    pub(crate) visited: bool,
}

impl<T> Copy for Node<T>
where
    T: Geometry,
    T::Vertex: Copy,
{
}

impl<T> Clone for Node<T>
where
    T: Geometry,
    T::Vertex: Clone,
{
    fn clone(&self) -> Self {
        Self {
            vertex: self.vertex.clone(),
            boundary: self.boundary.clone(),
            previous: self.previous.clone(),
            next: self.next.clone(),
            intersection: self.intersection.clone(),
            visited: self.visited.clone(),
        }
    }
}

pub(crate) struct Graph<T>
where
    T: Geometry,
{
    /// The vertices in the graph.
    pub vertices: Vec<Node<T>>,
    /// The boundaries in the graph.
    pub boundaries: Vec<Boundary>,
}

impl<T> Graph<T>
where
    T: Geometry,
    T::Vertex: Copy,
{
    /// Returns a reference of the vertex at the given position in the graph if, and only if,
    /// it has not been taken; otherwise returns [`Option::None`].
    pub(crate) fn get(&self, position: usize) -> Option<&Node<T>> {
        if self.vertices[position].visited {
            return None;
        }

        Some(&self.vertices[position])
    }

    /// Returns the vertex at the given position in the graph if, and only if, it has not been
    /// taken before; otherwise returns [`Option::None`].
    pub(crate) fn take(&mut self, position: usize) -> Option<Node<T>> {
        if self.vertices[position].visited {
            return None;
        }

        self.vertices[position].visited = true;
        self.boundaries[self.vertices[position].boundary].visited = true;

        Some(self.vertices[position])
    }
}

impl<T> Graph<T>
where
    T: Geometry,
{
    /// Returns the builder for a new graph.
    pub(crate) fn builder<'a>(tolerance: &'a <T::Vertex as IsClose>::Tolerance) -> GraphBuilder<'a, T, Unknown, Unknown> {
        GraphBuilder {
            vertices: Vec::new(),
            boundaries: Vec::new(),
            subject: Unknown,
            clip: Unknown,
            tolerance,
        }
    }

    pub(crate) fn corner(&self, position: usize) -> Corner<'_, T::Vertex> {
        let node = &self.vertices[position];

        Corner { 
            neighbors: Neighbors { tail: &self.vertices[node.previous].vertex, head: &self.vertices[node.next].vertex }, 
            role: self.boundaries[node.boundary].role,
            intersection: node.intersection.as_ref().map(|intersection| {
                let sibling = &self.vertices[intersection.sibling];
                
                crate::Intersection { 
                event: intersection.event,
                neighbors: Neighbors { tail: &self.vertices[sibling.previous].vertex, head: &self.vertices[sibling.next].vertex }
            }}),
            vertex: &node.vertex,
        }
    }
}
