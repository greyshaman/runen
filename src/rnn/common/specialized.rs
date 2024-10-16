use super::spec_type::SpecificationType;

pub trait Specialized {
  fn get_spec_type(&self) -> SpecificationType;
}