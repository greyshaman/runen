use super::{spec_type::SpecificationType, specialized::Specialized};

pub trait Aggregator: Specialized {
  fn get_spec_type(&self) -> SpecificationType {
    SpecificationType::Aggregator
  }
}