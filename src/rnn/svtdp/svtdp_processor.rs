use std::{
    collections::{BTreeMap, HashMap},
    error::Error,
    sync::Arc,
};

use tokio::{sync::RwLock, task::JoinHandle};
use tokio_util::task::TaskTracker;

use crate::rnn::common::{
    arithmetic::Arithmetic, neuron_cfg::ProcessorCfg, rnn_error::RnnError,
    signal_processor::SignalProcessor,
};

use super::svtdp_processor_cfg::SvtdpProcessorCfg;
use super::{svtdp_axon::SvtdpAxon, svtdp_synapse::SvtdpSynapse};

pub struct SvtdpProcessor<S>
where
    S: Arithmetic,
{
    bias: S,
    threshold: S,

    accumulator: Arc<RwLock<S>>,
    reset_counter: Arc<RwLock<usize>>,
    hit_counter: Arc<RwLock<usize>>,
    synapses: Arc<RwLock<BTreeMap<usize, SvtdpSynapse<S>>>>,
    axon: Arc<RwLock<Option<SvtdpAxon<S>>>>,
    synapse_connection_handlers: HashMap<usize, JoinHandle<()>>,
    receivers_task_tracker: TaskTracker,
}

impl<S> SvtdpProcessor<S>
where
    S: Arithmetic,
{
    pub fn new(bias: S, threshold: S) -> Self {
        SvtdpProcessor {
            bias,
            threshold,
            accumulator: Arc::new(RwLock::new(bias)),
            reset_counter: Arc::new(RwLock::new(0)),
            hit_counter: Arc::new(RwLock::new(0)),
            synapses: Arc::new(RwLock::new(BTreeMap::new())),
            axon: Arc::new(RwLock::new(None)),
            synapse_connection_handlers: HashMap::new(),
            receivers_task_tracker: TaskTracker::new(),
        }
    }

    pub fn from_cfg(cfg: ProcessorCfg<S>) -> Result<Self, Box<dyn Error>> {
        if let ProcessorCfg::Svtdp(processor_cfg, input_cfg_vec) = cfg {
            let SvtdpProcessorCfg { bias, threshold } = processor_cfg;
            let synapses = input_cfg_vec.iter().enumerate().fold(
                BTreeMap::new(),
                |mut acc, (port_id, input_cfg)| {
                    acc.insert(
                        port_id,
                        SvtdpSynapse::new(
                            port_id,
                            input_cfg.capacity_max,
                            input_cfg.regeneration,
                            input_cfg.weight,
                            input_cfg.processing_delay,
                        ),
                    );
                    acc
                },
            );

            let processor = SvtdpProcessor {
                bias,
                threshold,
                accumulator: Arc::new(RwLock::new(bias)),
                reset_counter: Arc::new(RwLock::new(0)),
                hit_counter: Arc::new(RwLock::new(0)),
                synapses: Arc::new(RwLock::new(synapses)),
                axon: Arc::new(RwLock::new(None)),
                synapse_connection_handlers: HashMap::new(),
                receivers_task_tracker: TaskTracker::new(),
            };

            Ok(processor)
        } else {
            Err(Box::new(RnnError::BadConfig))
        }
    }
}

impl<S> SignalProcessor<S> for SvtdpProcessor<S>
where
    S: Arithmetic,
{
    fn process(
        &self,
        signal_value: S,
        port: usize,
    ) -> std::pin::Pin<Box<dyn Future<Output = Result<(), Box<dyn Error>>> + Send + '_>> {
        todo!()
    }

    fn clear(
        &self,
    ) -> std::pin::Pin<Box<dyn Future<Output = Result<(), Box<dyn Error>>> + Send + '_>> {
        todo!()
    }

    fn config(
        &self,
        conf: ProcessorCfg<S>,
    ) -> std::pin::Pin<Box<dyn Future<Output = Result<(), Box<dyn Error>>> + Send + '_>> {
        todo!()
    }

    fn get_config(
        &self,
    ) -> std::pin::Pin<Box<dyn Future<Output = Result<ProcessorCfg<S>, Box<dyn Error>>> + Send + '_>>
    {
        todo!()
    }

    fn connect(
        &self,
        dst_neuron_id: &str,
        synapse_id: usize,
    ) -> std::pin::Pin<Box<dyn Future<Output = Result<(), Box<dyn Error>>> + Send + '_>> {
        todo!()
    }

    fn output_connect(
        &self,
        dst_output_port_id: &str,
    ) -> std::pin::Pin<Box<dyn Future<Output = Result<(), Box<dyn Error>>> + Send + '_>> {
        todo!()
    }

    fn receive(
        &self,
        synapse_id: usize,
        signal: crate::rnn::common::signal::Signal<S>,
    ) -> std::pin::Pin<Box<dyn Future<Output = Result<(), Box<dyn Error>>> + Send + '_>> {
        todo!()
    }

    fn send(
        &self,
        signal: crate::rnn::common::signal::Signal<S>,
    ) -> std::pin::Pin<Box<dyn Future<Output = Result<(), Box<dyn Error>>> + Send + '_>> {
        todo!()
    }
}
