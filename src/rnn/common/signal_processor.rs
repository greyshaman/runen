use std::{error::Error, pin::Pin};

use crate::rnn::common::arithmetic::Arithmetic;

use super::{neuron_cfg::ProcessorCfg, signal::Signal};

pub trait SignalProcessor<S>: Send + Sync
where
    S: Arithmetic,
{
    fn process(
        &self,
        signal_value: S,
        port: usize,
    ) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn Error>>> + Send + '_>>;

    fn clear(&self) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn Error>>> + Send + '_>>;

    fn config(
        &self,
        conf: ProcessorCfg<S>,
    ) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn Error>>> + Send + '_>>;

    fn get_config(
        &self,
    ) -> Pin<Box<dyn Future<Output = Result<ProcessorCfg<S>, Box<dyn Error>>> + Send + '_>>;

    fn connect(
        &self,
        dst_neuron_id: &str,
        synapse_id: usize,
    ) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn Error>>> + Send + '_>>;
    fn output_connect(
        &self,
        dst_output_port_id: &str,
    ) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn Error>>> + Send + '_>>;

    fn receive(
        &self,
        synapse_id: usize,
        signal: Signal<S>,
    ) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn Error>>> + Send + '_>>;
    fn send(
        &self,
        signal: Signal<S>,
    ) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn Error>>> + Send + '_>>;
}
