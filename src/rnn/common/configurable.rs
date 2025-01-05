use std::{error::Error, future::Future};

pub trait Configurable {
    /// Configuring the input interface of the network in such a way that an unambiguous
    fn setup_input(
        &self,
        network_port: usize,
        neuron_id: &str,
        neuron_port: usize,
    ) -> impl Future<Output = Result<(), Box<dyn Error>>> + Send;

    /// Configuring the output interface of network
    fn setup_output(
        &self,
        network_port: usize,
        neuron_id: &str,
    ) -> impl Future<Output = Result<(), Box<dyn Error>>>;
}