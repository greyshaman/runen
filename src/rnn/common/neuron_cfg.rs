use serde::{Deserialize, Serialize};

use crate::rnn::stdp::{stdp_input_cfg::StdpInputCfg, stdp_processor_cfg::StdpProcessorCfg};
use crate::rnn::svtdp::svtdp_input_cfg::SvtdpInputCfg;
use crate::rnn::svtdp::svtdp_processor_cfg::SvtdpProcessorCfg;

use super::arithmetic::Arithmetic;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NeuronCfg<S>
where
    S: Arithmetic,
{
    pub id: String,
    pub processor_cfg: ProcessorCfg<S>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ProcessorCfg<S>
where
    S: Arithmetic,
{
    Stdp(StdpProcessorCfg<S>, Vec<StdpInputCfg>),
    Svtdp(SvtdpProcessorCfg<S>, Vec<SvtdpInputCfg<S>>),
    Rcsa,
}
