use std::error::Error;

use serde::{Deserialize, Serialize};

use crate::rnn::common::arithmetic::Arithmetic;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SvtdpProcessorCfg<S>
where
    S: Arithmetic,
{
    pub bias: S,
    pub threshold: S,
}

impl<S> SvtdpProcessorCfg<S>
where
    S: Arithmetic,
{
    pub fn new(bias: S, threshold: S) -> Result<Self, Box<dyn Error>> {
        Ok(SvtdpProcessorCfg { bias, threshold })
    }
}
