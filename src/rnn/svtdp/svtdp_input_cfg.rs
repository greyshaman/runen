use std::error::Error;

use chrono::TimeDelta;
use serde::{Deserialize, Serialize};

use crate::rnn::common::{arithmetic::Arithmetic, rnn_error::RnnError};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SvtdpInputCfg<S>
where
    S: Arithmetic,
{
    pub capacity_max: S,
    pub regeneration: S,
    pub weight: S,
    pub processing_delay: TimeDelta,
}

impl<S> SvtdpInputCfg<S>
where
    S: Arithmetic,
{
    pub fn new(
        capacity_max: S,
        regeneration: S,
        weight: S,
        processing_delay: TimeDelta,
    ) -> Result<Self, Box<dyn Error>> {
        if regeneration > capacity_max {
            return Err(Box::new(RnnError::NotSupportedArgValue));
        }

        Ok(SvtdpInputCfg {
            capacity_max,
            regeneration,
            weight,
            processing_delay,
        })
    }
}

#[macro_export]
macro_rules! svtdp_synapse_input_cfg {
    ($capacity_max:expr, $regeneration:expr, $weight:expr, $processing_delay:expr) => {
        SvtdpInputCfg::new($capacity_max, $regeneration, $weight, $processing_delay)
    };
    ($capacity_max:expr, $regeneration:expr, $weight:expr) => {
        svtdp_synapse_input_cfg!(
            $capacity_max,
            $regeneration,
            $weight,
            TimeDelta::new(0, 0).unwrap()
        )
    };
    ($capacity_max:expr, $regeneration:expr) => {
        svtdp_synapse_input_cfg!(
            $capacity_max,
            $regeneration,
            1,
            TimeDelta::new(0, 0).unwrap()
        )
    };
    ($capacity_max:expr) => {
        svtdp_synapse_input_cfg!(
            $capacity_max,
            $capacity_max,
            1,
            TimeDelta::new(0, 0).unwrap()
        )
    };
    () => {
        svtdp_synapse_input_cfg!(1, 1, 1, TimeDelta::new(0, 0).unwrap())
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_should_serialize_json_string() {
        let cfg = svtdp_synapse_input_cfg!().expect("configuration with default parameters");
        let cfg_json = serde_json::to_string(&cfg).expect("Should serialize to json string");
        assert_eq!(
            cfg_json,
            "{\"capacity_max\":1,\"regeneration\":1,\"weight\":1,\"processing_delay\":[0,0]}"
        );
    }
}
