use std::error::Error;

/// Runen library errors
#[derive(Debug)]
pub enum RnnError {
    NeuronNotFound(String),

    DendriteNotFound(usize),

    PortNotFound(usize),

    IncorrectPortType,

    PortBusy(String),

    NeuronAlreadyExists(String),

    /// Not found matched data by provided pattern
    PatternNotFound,

    /// Expected data not present
    ExpectedDataNotPresent(String),

    /// Not supported argument value
    NotSupportedArgValue,

    /// When connection to self not allowed
    ClosedLoop,

    /// Happened when sending signal suppressed. Needed to logging in trace log.
    SignalSuppressed,

    /// Indicate then signal sending into channel whit no one receivers
    SignalSendError,

    /// Not connected axon
    DeadEndAxon,

    /// Port is not connected to any neurons.
    PortAlreadyFree,

    /// When attempt to send neuron status but channel has been closed.
    MonitoringChannelClosed(String),

    /// When attempt to send neuron status but channel is full.
    MonitoringChannelFull(String),
}

impl std::fmt::Display for RnnError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}", self)
    }
}

impl Error for RnnError {}
