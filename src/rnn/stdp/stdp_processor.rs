use std::{
    collections::{BTreeMap, HashMap, btree_map::Entry},
    error::Error,
    pin::Pin,
    sync::Arc,
};

use tokio::{sync::RwLock, task::JoinHandle};
use tokio_util::task::TaskTracker;

use crate::rnn::common::{
    arithmetic::Arithmetic, neuron_cfg::ProcessorCfg, rnn_error::RnnError,
    signal_processor::SignalProcessor,
};

use super::{
    stdp_axon::StdpAxon, stdp_input_cfg::StdpInputCfg, stdp_processor_cfg::StdpProcessorCfg,
    stdp_synapse::StdpSynapse,
};

pub struct StdpProcessor<S>
where
    S: Arithmetic,
{
    /// The processor Bias
    bias: S,

    /// Firing threshold
    threshold: S,

    accumulator: Arc<RwLock<S>>,
    reset_counter: Arc<RwLock<usize>>,
    hit_counter: Arc<RwLock<usize>>,
    synapses: Arc<RwLock<BTreeMap<usize, StdpSynapse<S>>>>,
    axon: Arc<RwLock<Option<StdpAxon<S>>>>,
    synapse_connection_handlers: HashMap<usize, JoinHandle<()>>,
}

impl<S> StdpProcessor<S>
where
    S: Arithmetic,
{
    pub fn new(bias: S, threshold: S) -> Self {
        StdpProcessor {
            bias,
            threshold,
            accumulator: Arc::new(RwLock::new(bias)),
            reset_counter: Arc::new(RwLock::new(0)),
            hit_counter: Arc::new(RwLock::new(0)),
            synapses: Arc::new(RwLock::new(BTreeMap::new())),
            axon: Arc::new(RwLock::new(None)),
            synapse_connection_handlers: HashMap::new(),
        }
    }

    pub fn from_cfg(cfg: ProcessorCfg<S>) -> Result<Self, Box<dyn Error>> {
        if let ProcessorCfg::Stdp(processor_cfg, input_cfg_vec) = cfg {
            let StdpProcessorCfg { bias, threshold } = processor_cfg;
            let synapses = input_cfg_vec.iter().enumerate().fold(
                BTreeMap::new(),
                |mut acc, (port_id, input_cfg)| {
                    acc.insert(
                        port_id,
                        StdpSynapse::new(port_id, input_cfg.processing_delay),
                    );
                    acc
                },
            );

            let processor = StdpProcessor {
                bias,
                threshold,
                accumulator: Arc::new(RwLock::new(bias)),
                reset_counter: Arc::new(RwLock::new(0)),
                hit_counter: Arc::new(RwLock::new(0)),
                synapses: Arc::new(RwLock::new(synapses)),
                axon: Arc::new(RwLock::new(None)),
                synapse_connection_handlers: HashMap::new(),
            };

            Ok(processor)
        } else {
            Err(Box::new(RnnError::NotSupportedArgValue))
        }
    }
}

impl<S> SignalProcessor<S> for StdpProcessor<S>
where
    S: Arithmetic,
{
    fn process(
        &self,
        signal_value: S,
        port: usize,
    ) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn Error>>> + Send + '_>>
    where
        S: Arithmetic,
    {
        todo!()
    }

    fn clear(&self) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn Error>>> + Send + '_>> {
        Box::pin(async move {
            self.synapses.write().await.clear();
            {
                let mut w_accumulator = self.accumulator.write().await;
                *w_accumulator = self.bias;
            }
            Ok(())
        })
    }

    fn config(
        &self,
        conf: ProcessorCfg<S>,
    ) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn Error>>> + Send + '_>> {
        Box::pin(async move {
            if let ProcessorCfg::Stdp(_, input_cfg_vec) = conf {
                self.clear().await;

                for (port_id, input_cfg) in input_cfg_vec.iter().enumerate() {
                    match self.synapses.write().await.entry(port_id) {
                        Entry::Occupied(entry) => {
                            let synapse = entry.get();
                            synapse
                                .set_processing_delay(input_cfg.processing_delay)
                                .await;
                        }
                        Entry::Vacant(entry) => {
                            let synapse = StdpSynapse::new(port_id, input_cfg.processing_delay);
                            entry.insert(synapse);
                        }
                    }
                }
                Ok(())
            } else {
                Err(Box::new(RnnError::NotSupportedArgValue) as Box<dyn Error>)
            }
        })
    }

    fn get_config(
        &self,
    ) -> Pin<Box<dyn Future<Output = Result<ProcessorCfg<S>, Box<dyn Error>>> + Send + '_>> {
        Box::pin(async move {
            let mut input_cfgs = vec![];
            let synapses = self.synapses.read().await;
            for synapse in synapses.values() {
                input_cfgs.push(
                    StdpInputCfg::new(synapse.processing_delay().await)
                        .expect("instanse should returns a correct config"),
                );
            }

            Ok(ProcessorCfg::Stdp(
                StdpProcessorCfg {
                    bias: self.bias,
                    threshold: self.threshold,
                },
                input_cfgs,
            ))
        })
    }

    fn connect(
        &self,
        dst_neuron_id: &str,
        synapse_id: usize,
    ) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn Error>>> + Send + '_>> {
        todo!()
    }

    fn output_connect(
        &self,
        dst_output_port_id: &str,
    ) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn Error>>> + Send + '_>> {
        todo!()
    }

    fn receive(
        &self,
        synapse_id: usize,
        signal: crate::rnn::common::signal::Signal<S>,
    ) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn Error>>> + Send + '_>> {
        todo!()
    }

    fn send(
        &self,
        signal: crate::rnn::common::signal::Signal<S>,
    ) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn Error>>> + Send + '_>> {
        todo!()
    }
}
