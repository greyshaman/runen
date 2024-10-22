use super::spec_type::SpecificationType;

/// An entity that has a specific type.
pub trait Specialized {
  fn get_spec_type(&self) -> SpecificationType;
}