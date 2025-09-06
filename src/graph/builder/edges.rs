use crate::{graph::Node, Edge, Geometry};

/// An edge and its location in the graph.
pub struct EdgeLocation<'a, T>
where
    T: 'a + Geometry,
{
    /// The actual edge.
    pub edge: T::Edge<'a>,
    /// The position in the graph of the first endpoint of the edge.
    pub position: usize,
}

/// Yields all the edges of the boundary starting at the given position.
pub struct LocateEdges<'a, T>
where
    T: Geometry,
{
    pub vertices: &'a Vec<Node<T>>,
    pub next: Option<usize>,
    pub start: usize,
}

impl<'a, T> Iterator for LocateEdges<'a, T>
where
    T: Geometry,
{
    type Item = EdgeLocation<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(current) = self.next
            && current == self.start
        {
            return None;
        }

        let position = self.next.unwrap_or(self.start);
        let node = &self.vertices[position];
        self.next = Some(node.next);

        Some(EdgeLocation {
            edge: T::Edge::new(&node.vertex, &self.vertices[node.next].vertex),
            position,
        })
    }
}
