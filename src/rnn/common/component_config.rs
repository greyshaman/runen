use std::any::Any;

use super::spec_type::SpecificationType;

pub trait ComponentConfig: Any {
    fn get_id(&self) -> usize;
    fn get_spec_type(&self) -> SpecificationType;
    fn as_any(&self) -> &dyn Any;
}
