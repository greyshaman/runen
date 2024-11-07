use crate::rnn::common::component_config::ComponentConfig;
use crate::rnn::common::spec_type::SpecificationType;

#[derive(Debug)]
pub struct AxonConfig(pub usize, pub SpecificationType);

impl ComponentConfig for AxonConfig {
    fn get_id(&self) -> usize {
        self.0
    }

    fn get_spec_type(&self) -> SpecificationType {
        self.1.clone()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
