use super::spec_type::SpecificationType;

/// An entity that has a specific type and can show it to other.
pub trait Specialized {
    /// Returns SpecificationType to differ concrete component from other
    fn get_spec_type(&self) -> SpecificationType;
}
