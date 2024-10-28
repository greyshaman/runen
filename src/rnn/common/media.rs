use std::{any::Any, cell::RefCell, error::Error, rc::Rc};

use as_any::AsAny;

use super::{
    container::Container, identity::Identity, spec_type::SpecificationType,
    specialized::Specialized,
};

/// Media is a system that consists of various functional elements in containers.
/// It manages the creation, updating, and deletion of these containers.
pub trait Media: Identity + Specialized + AsAny {
    /// Gets container by id
    fn get_container(&self, id: &str) -> Option<&Rc<RefCell<dyn Container>>>;

    /// Create and insert container
    fn create_container(
        &mut self,
        spec_type: &SpecificationType,
        media: &Rc<RefCell<dyn Media>>,
    ) -> Result<Rc<RefCell<dyn Container>>, Box<dyn Error>>;

    /// Remove container with dependencies
    fn remove_container(&mut self, id: &str) -> Result<(), Box<dyn Error>>;

    /// Verify if has container by id
    fn has_container(&self, id: &str) -> bool;

    /// Get how many container in this media
    fn len(&self) -> usize;
}
