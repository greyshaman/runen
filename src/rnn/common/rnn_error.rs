use std::error::Error;

/// Runen library errors
#[derive(Debug)]
pub enum RnnError {
    /// Already has key when attempt insert element into Map or Set
    OccupiedKey,

    /// Not found key when try to use it to access element in Map or Set
    IdNotFound,

    /// Error on attempt to create unique entity with used Id
    IdBusy(String),

    /// Not found matched data by provided pattern
    PatternNotFound,

    /// Expected data not present
    NotPresent(String),

    /// Not supported argument value
    NotSupportedArgValue,

    /// When create entity in container which should be single
    OnlySingleAllowed,

    /// When connection to self not allowed
    ClosedLoop,

    /// Happened when sending signal suppressed. Needed to logging in trace log.
    SignalSuppressed,

    /// Indicate then signal sending into channel whit no one receivers
    SignalSendError,

    /// Not connected axon
    DeadEndAxon,

    /// Port is not connected to any neurons.
    AlreadyFree,

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
