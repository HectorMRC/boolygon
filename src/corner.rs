/// The local information of a vertex.
pub struct Neighbors<'a, T> {
    /// The vertex before.
    pub tail: &'a T,
    /// The vertex after.
    pub head: &'a T,
}

/// The kind of event happening on an [`Intersection`], if any.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    /// The boundary is entering into the other.
    Entry,
    /// The boundary is exiting from the other.
    Exit,
}

/// Intersection details between two edges.
pub struct Intersection<'a, T> {
    /// The event happening on this intersection, if any.
    pub event: Option<Event>,
    /// The local information of the complementary vertex of this intersection.
    pub neighbors: Neighbors<'a, T>,
}

/// The role of a shape during a clipping operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    /// The shape being clipped.
    Subject,
    /// The shape clipping the subject.
    Clip,
}

impl Role {
    /// Returns true if, and only if, this is [`Role::Subject`].
    pub(crate) fn is_subject(&self) -> bool {
        matches!(self, Self::Subject)
    }
}

/// The local information of a vertex in a shape.
pub struct Corner<'a, T> {
    /// The central vertex of the corner.
    pub vertex: &'a T,
    /// The local information around this corner's vertex.
    pub neighbors: Neighbors<'a, T>,
    /// The role of the shape in which this corner belongs.
    pub role: Role,
    /// The intersection happening at this corner, if any.
    pub intersection: Option<Intersection<'a, T>>,
}