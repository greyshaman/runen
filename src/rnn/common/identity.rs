/// An entity capable of identification
pub trait Identity {
    /// Returns the ID.
    fn get_id(&self) -> String;
}
