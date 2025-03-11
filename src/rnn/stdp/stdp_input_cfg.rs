use std::error::Error;

use chrono::TimeDelta;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StdpInputCfg {
    pub processing_delay: TimeDelta,
}

impl StdpInputCfg {
    pub fn new(processing_delay: TimeDelta) -> Result<Self, Box<dyn Error>> {
        Ok(StdpInputCfg { processing_delay })
    }
}

#[macro_export]
macro_rules! stdp_synapse_cfg {
    ($processing_delay:expr) => {
        crate::rnn::stdp::stdp_input_cfg::StdpInputCfg::new($processing_delay)
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_should_serialize_with_zero_delay() {
        let cfg = StdpInputCfg::new(TimeDelta::new(0, 0).expect("empty time delta"))
            .expect("should create correct config");
        let cfg_json =
            serde_json::to_string(&cfg).expect("Config with zero delay should serialize");
        assert_eq!(cfg_json, "{\"processing_delay\":[0,0]}");
    }
}
