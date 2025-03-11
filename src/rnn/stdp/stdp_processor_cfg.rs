use std::error::Error;

use serde::{Deserialize, Serialize};

use crate::rnn::common::arithmetic::Arithmetic;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StdpProcessorCfg<S>
where
    S: Arithmetic,
{
    pub bias: S,
    pub threshold: S,
}

impl<S> StdpProcessorCfg<S>
where
    S: Arithmetic,
{
    pub fn new(bias: S, threshold: S) -> Result<Self, Box<dyn Error>> {
        Ok(StdpProcessorCfg { bias, threshold })
    }
}
