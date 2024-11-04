use crate::rnn::common::{component_config::ComponentConfig, spec_type::SpecificationType};

pub struct DendriteConfig(pub usize, pub SpecificationType, pub usize);

impl DendriteConfig {
    pub fn get_weight(&self) -> usize {
        self.2
    }
}

impl ComponentConfig for DendriteConfig {
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
