use std::any::Any;

/// Derive trait for reflection from abstract to concrete type
pub trait AsAny: Any {
    /// A method for carrying out reflection from a characteristic
    /// to the types that implement it
    fn as_any(&self) -> &dyn Any;

    /// The same as "as_any", but for mutable entities.
    fn as_any_mut(&mut self) -> &mut dyn Any;
}