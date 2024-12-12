use crate::rnn::layouts::network::{self, MonitoringMode};

/// Commands set to control network state
pub enum NetCommand {
    /// Loading network configuration from file with specified filename.
    LoadCfg(String),

    /// Saving configuration into specified config file
    SaveCfg(String),

    /// Switch network tracing mode. It will propagate to all neurons
    SwitchTracingMode(network::MonitoringMode),
    Start,
    Stop,
    Pause,
    Resume,
    Training,
    Snapshot,
    Optimize,
}

/// Commands set to control neuron state
#[derive(Debug, Clone)]
pub enum NeuronCommand {
    SwitchMonitoringMode(MonitoringMode),
}
