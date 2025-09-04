use std::collections::BTreeMap;

use crate::{graph::{GraphBuilder}, Edge, Geometry, MaybePair, Shape};

/// The intersection between two edges.
#[derive(Debug)]
pub(super) struct EdgeIntersection<T> {
    /// The vertex of intersection between the edges.
    pub(super) vertex: T,
    /// The position in the graph of the intersecting subject edge.
    pub(super) subject: usize,
    /// The position in the graph of the intersecting clip edge.
    pub(super) clip: usize,
}

/// All the intersections between the edges of a subject and clip [`Shape`]s.
pub(super) struct EdgeIntersections<T> {
    /// The intersections between subject and clip shape.
    pub(super) all: Vec<EdgeIntersection<T>>,
    /// The position of the intersections grouped by edge.
    pub(super) by_edge: BTreeMap<usize, Vec<usize>>,
}

impl<T> Default for EdgeIntersections<T> {
    fn default() -> Self {
        Self {
            all: Default::default(),
            by_edge: Default::default(),
        }
    }
}

impl<'a, T> From<&'a GraphBuilder<'_, T, &Shape<T>, &Shape<T>>> for EdgeIntersections<T::Vertex>
where T: Geometry {
    fn from(builder: &'a GraphBuilder<T, &Shape<T>, &Shape<T>>) -> Self {
        let mut intersections = EdgeIntersections::default();
        for subject_boundary in builder
            .boundaries
            .iter()
            .filter(|boundary| boundary.role.is_subject())
        {
            for clip_boundary in builder
                .boundaries
                .iter()
                .filter(|boundary| !boundary.role.is_subject())
            {
                for subject in builder.edges(subject_boundary) {
                    for clip in builder.edges(clip_boundary) {
                        if let Some(intersection) =
                            subject.edge.intersection(&clip.edge, &builder.tolerance)
                        {
                            intersections = match intersection {
                                MaybePair::Single(vertex) => {
                                    intersections.with_intersection(EdgeIntersection {
                                        vertex,
                                        subject: subject.position,
                                        clip: clip.position,
                                    })
                                }
                                MaybePair::Pair([first, second]) => {
                                    let intersection = EdgeIntersection {
                                        vertex: first,
                                        subject: subject.position,
                                        clip: clip.position,
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

impl<T> EdgeIntersections<T> {
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
