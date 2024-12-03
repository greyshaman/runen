use std::error::Error;

use serde::{Deserialize, Serialize};

use super::rnn_error::RnnError;

/// Input configuration
#[derive(Debug, Serialize, Deserialize)]
pub struct InputCfg {
    /// High limit of synapse (input) capacity
    pub capacity_max: u8,

    /// The amount of capacity recovery after its reduction
    pub regeneration: u8,

    /// Th dendrite's weight
    pub weight: i16,
}

impl InputCfg {
    pub fn new(capacity_max: u8, regeneration: u8, weight: i16) -> Result<Self, Box<dyn Error>> {
        if regeneration > capacity_max {
            Err(Box::new(RnnError::NotSupportedArgValue))
        } else {
            Ok(InputCfg {
                capacity_max,
                regeneration,
                weight,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn correct_args_should_return_instance_in_result() {
        assert!(InputCfg::new(1, 1, 1).is_ok());
        assert!(InputCfg::new(2, 1, -1).is_ok());
    }

    #[test]
    fn should_returns_error_when_regeneration_more_then_capacity() {
        assert!(InputCfg::new(1, 2, 1).is_err());
    }

    #[test]
    fn config_should_serialize_into_json_string() {
        let cfg = InputCfg::new(1, 1, 1).unwrap();
        let cfg_json = serde_json::to_string(&cfg).unwrap();

        assert_eq!(
            cfg_json,
            "{\"capacity_max\":1,\"regeneration\":1,\"weight\":1}"
        );
    }

    #[test]
    fn config_should_serialize_into_yaml_string() {
        let cfg = InputCfg::new(3, 2, 1).unwrap();
        let cfg_yaml = serde_yaml::to_string(&cfg).unwrap();

        assert_eq!(cfg_yaml, "capacity_max: 3\nregeneration: 2\nweight: 1\n");
    }

    #[test]
    fn config_should_deserialize_from_json_string() {
        let cfg_json = json!({"capacity_max": 1, "regeneration": 1,"weight": 1}).to_string();
        let cfg: InputCfg = serde_json::from_str(&cfg_json).unwrap();

        assert_eq!(cfg.capacity_max, 1);
        assert_eq!(cfg.regeneration, 1);
        assert_eq!(cfg.weight, 1);
    }

    #[test]
    fn config_shoud_deserialize_from_yaml_string() {
        let cfg_yaml = "capacity_max: 3\nregeneration: 2\nweight: 1\n";
        let cfg: InputCfg = serde_yaml::from_str(&cfg_yaml).unwrap();

        assert_eq!(cfg.capacity_max, 3);
        assert_eq!(cfg.regeneration, 2);
        assert_eq!(cfg.weight, 1);
    }
}
