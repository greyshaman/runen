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
}
