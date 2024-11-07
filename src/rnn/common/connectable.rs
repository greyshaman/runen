/// An entity can connect to or disconnect from another connected entity.
pub trait Connectable {
    /// Connect to other component by its id
    /// This component should be in owner container except when axon tried connect to synapse from another neuron
    fn connect(&mut self, _party_id: &str) {}

    /// Disconnect from connected component by its id
    fn disconnect(&mut self, _party_id: &str) {}
}
