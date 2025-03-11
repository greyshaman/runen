use std::error::Error;

use serde::{Deserialize, Serialize};

use crate::rnn::common::{arithmetic::Arithmetic, rnn_error::RnnError};

/// Input configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RcsaInputCfg<S>
where
    S: Arithmetic,
{
    /// High limit of synapse (input) capacity
    pub capacity_max: S,

    /// The amount of capacity recovery after its reduction
    pub regeneration: S,

    /// Th dendrite's weight
    pub weight: S,
}

impl<S> RcsaInputCfg<S>
where
    S: Arithmetic,
{
    pub fn new(capacity_max: S, regeneration: S, weight: S) -> Result<Self, Box<dyn Error>> {
        if regeneration > capacity_max {
            Err(Box::new(RnnError::NotSupportedArgValue))
        } else {
            Ok(RcsaInputCfg {
                capacity_max,
                regeneration,
                weight,
            })
        }
    }
}

#[macro_export]
macro_rules! rcsa_input_cfg {
    ($capacity_max:expr, $regeneration:expr, $weight:expr) => {
        RcsaInputCfg::new($capacity_max, $regeneration, $weight)
    };
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn correct_args_should_return_instance_in_result() {
        assert!(rcsa_input_cfg!(1, 1, 1).is_ok());
        assert!(rcsa_input_cfg!(2, 1, -1).is_ok());
    }

    #[test]
    fn should_returns_error_when_regeneration_more_then_capacity() {
        assert!(rcsa_input_cfg!(1, 2, 1).is_err());
    }

    #[test]
    #[ignore = "need change config structure"]
    fn config_should_serialize_into_json_string() {
        let cfg = rcsa_input_cfg!(1, 1, 1).unwrap();
        let cfg_json = serde_json::to_string(&cfg).unwrap();

        assert_eq!(
            cfg_json,
            "{\"capacity_max\":1,\"regeneration\":1,\"weight\":1,\"processing_delay\":0}"
        );
    }

    #[test]
    #[ignore = "need change config structure"]
    fn config_should_serialize_into_yaml_string() {
        let cfg = RcsaInputCfg::new(3, 2, 1).unwrap();
        let cfg_yaml = serde_yaml::to_string(&cfg).unwrap();

        assert_eq!(
            cfg_yaml,
            "capacity_max: 3\nregeneration: 2\nweight: 1\nprocessing_delay: 0\n"
        );
    }

    #[test]
    fn config_should_deserialize_from_json_string() {
        let cfg_json =
            json!({"capacity_max": 1, "regeneration": 1,"weight": 1, "processing_delay": 0})
                .to_string();
        let cfg: RcsaInputCfg<i16> = serde_json::from_str(&cfg_json).unwrap();

        assert_eq!(cfg.capacity_max, 1);
        assert_eq!(cfg.regeneration, 1);
        assert_eq!(cfg.weight, 1);
    }

    #[test]
    fn config_shoud_deserialize_from_yaml_string() {
        let cfg_yaml = "capacity_max: 3\nregeneration: 2\nweight: 1\nprocessing_delay: 0\n";
        let cfg: RcsaInputCfg<i16> = serde_yaml::from_str(&cfg_yaml).unwrap();

        assert_eq!(cfg.capacity_max, 3);
        assert_eq!(cfg.regeneration, 2);
        assert_eq!(cfg.weight, 1);
    }
}
