use serde::{Deserialize, Serialize};

use super::{arithmetic::Arithmetic, neuron_cfg::NeuronCfg};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LinkCfg {
    Input {
        input_port: usize,
        dst_id: String,
        dst_synapse_idx: usize,
    },
    Inner {
        src_id: String,
        dst_id: String,
        dst_synapse_idx: usize,
    },
    Output {
        src_id: String,
        output_port: usize,
    },
}

// #[derive(Debug, Serialize, Deserialize)]
// pub struct OldNeuronCfg<S>
// where
//     S: Arithmetic,
// {
//     pub id: String,
//     pub bias: S,
//     pub input_configs: Vec<InputCfg<S>>,
// }

/// The network config structure used to describe neuron set and connections between them.
#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkCfg<S>
where
    S: Arithmetic,
{
    inputs: usize,
    outputs: usize,
    neurons: Vec<NeuronCfg<S>>,
    links: Vec<LinkCfg>,
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use chrono::TimeDelta;
    use serde_json::json;

    use crate::rnn::stdp::stdp_processor_cfg::StdpProcessorCfg;
    use crate::{
        rnn::{
            common::neuron_cfg::ProcessorCfg,
            stdp::{self, stdp_processor::StdpProcessor},
        },
        stdp_synapse_cfg,
    };

    use super::*;

    #[test]
    #[ignore = "need change config structure"]
    fn should_serialize_config_into_json_string() -> Result<(), Box<dyn Error>> {
        let neuron_cfgs = vec![
            NeuronCfg {
                id: String::from("M0Z0"),
                processor_cfg: ProcessorCfg::Stdp(
                    StdpProcessorCfg {
                        bias: 1,
                        threshold: 2,
                    },
                    vec![
                        stdp_synapse_cfg!(TimeDelta::new(0, 1000).unwrap())?,
                        stdp_synapse_cfg!(TimeDelta::new(0, 1000).unwrap())?,
                    ],
                ),
            },
            NeuronCfg {
                id: String::from("M0Z1"),
                processor_cfg: ProcessorCfg::Stdp(
                    StdpProcessorCfg {
                        bias: 1,
                        threshold: 2,
                    },
                    vec![stdp_synapse_cfg!(TimeDelta::new(0, 1000).unwrap())?],
                ),
            },
            NeuronCfg {
                id: String::from("M0Z2"),
                processor_cfg: ProcessorCfg::Stdp(
                    StdpProcessorCfg {
                        bias: 1,
                        threshold: 2,
                    },
                    vec![stdp_synapse_cfg!(TimeDelta::new(0, 1000).unwrap())?],
                ),
            },
            NeuronCfg {
                id: String::from("M0Z3"),
                processor_cfg: ProcessorCfg::Stdp(
                    StdpProcessorCfg {
                        bias: 1,
                        threshold: 2,
                    },
                    vec![
                        stdp_synapse_cfg!(TimeDelta::new(0, 1000).unwrap())?,
                        stdp_synapse_cfg!(TimeDelta::new(0, 1000).unwrap())?,
                    ],
                ),
            },
        ];
        let cfg = NetworkCfg {
            inputs: 3,
            outputs: 2,
            neurons: neuron_cfgs,
            links: vec![
                LinkCfg::Input {
                    input_port: 0,
                    dst_id: "M0Z0".to_string(),
                    dst_synapse_idx: 0,
                },
                LinkCfg::Input {
                    input_port: 1,
                    dst_id: "M0Z0".to_string(),
                    dst_synapse_idx: 1,
                },
                LinkCfg::Input {
                    input_port: 2,
                    dst_id: "M0Z1".to_string(),
                    dst_synapse_idx: 0,
                },
                LinkCfg::Inner {
                    src_id: String::from("M0Z0"),
                    dst_id: String::from("M0Z2"),
                    dst_synapse_idx: 0,
                },
                LinkCfg::Inner {
                    src_id: String::from("M0Z1"),
                    dst_id: String::from("M0Z3"),
                    dst_synapse_idx: 0,
                },
                LinkCfg::Inner {
                    src_id: String::from("M0Z1"),
                    dst_id: String::from("M0Z3"),
                    dst_synapse_idx: 1,
                },
                LinkCfg::Output {
                    src_id: String::from("M0Z2"),
                    output_port: 0,
                },
                LinkCfg::Output {
                    src_id: String::from("M0Z3"),
                    output_port: 1,
                },
            ],
        };

        let cfg_json = serde_json::to_string(&cfg).unwrap();

        let expected_string = "{\"inputs\":3,\"outputs\":2,\"neurons\":[{\"id\":\"M0Z0\",\"bias\":1,\"input_configs\":[{\"capacity_max\":2,\"regeneration\":2,\"weight\":1,\"processing_delay\":0},{\"capacity_max\":1,\"regeneration\":1,\"weight\":2,\"processing_delay\":0}]},{\"id\":\"M0Z1\",\"bias\":1,\"input_configs\":[{\"capacity_max\":1,\"regeneration\":1,\"weight\":1,\"processing_delay\":0}]},{\"id\":\"M0Z2\",\"bias\":1,\"input_configs\":[{\"capacity_max\":3,\"regeneration\":2,\"weight\":1,\"processing_delay\":0}]},{\"id\":\"M0Z3\",\"bias\":1,\"input_configs\":[{\"capacity_max\":1,\"regeneration\":1,\"weight\":1,\"processing_delay\":0},{\"capacity_max\":3,\"regeneration\":1,\"weight\":2,\"processing_delay\":0}]}],\"links\":[{\"Input\":{\"input_port\":0,\"dst_id\":\"M0Z0\",\"dst_synapse_idx\":0}},{\"Input\":{\"input_port\":1,\"dst_id\":\"M0Z0\",\"dst_synapse_idx\":1}},{\"Input\":{\"input_port\":2,\"dst_id\":\"M0Z1\",\"dst_synapse_idx\":0}},{\"Inner\":{\"src_id\":\"M0Z0\",\"dst_id\":\"M0Z2\",\"dst_synapse_idx\":0}},{\"Inner\":{\"src_id\":\"M0Z1\",\"dst_id\":\"M0Z3\",\"dst_synapse_idx\":0}},{\"Inner\":{\"src_id\":\"M0Z1\",\"dst_id\":\"M0Z3\",\"dst_synapse_idx\":1}},{\"Output\":{\"src_id\":\"M0Z2\",\"output_port\":0}},{\"Output\":{\"src_id\":\"M0Z3\",\"output_port\":1}}]}";
        assert_eq!(cfg_json, expected_string);

        Ok(())
    }

    #[test]
    #[ignore = "need change config structure"]
    fn should_serialize_config_into_yaml_string() -> Result<(), Box<dyn Error>> {
        let neuron_cfgs = vec![
            NeuronCfg {
                id: String::from("M0Z0"),
                processor_cfg: ProcessorCfg::Stdp(
                    StdpProcessorCfg {
                        bias: 1,
                        threshold: 2,
                    },
                    vec![
                        stdp_synapse_cfg!(TimeDelta::new(0, 1000).unwrap())?,
                        stdp_synapse_cfg!(TimeDelta::new(0, 1000).unwrap())?,
                    ],
                ),
            },
            NeuronCfg {
                id: String::from("M0Z1"),
                processor_cfg: ProcessorCfg::Stdp(
                    StdpProcessorCfg {
                        bias: 1,
                        threshold: 2,
                    },
                    vec![stdp_synapse_cfg!(TimeDelta::new(0, 1000).unwrap())?],
                ),
            },
        ];
        let cfg = NetworkCfg {
            inputs: 2,
            outputs: 1,
            neurons: neuron_cfgs,
            links: vec![
                LinkCfg::Input {
                    input_port: 0,
                    dst_id: "M0Z0".to_string(),
                    dst_synapse_idx: 0,
                },
                LinkCfg::Input {
                    input_port: 1,
                    dst_id: "M0Z0".to_string(),
                    dst_synapse_idx: 1,
                },
                LinkCfg::Inner {
                    src_id: String::from("M0Z0"),
                    dst_id: String::from("M0Z1"),
                    dst_synapse_idx: 0,
                },
                LinkCfg::Output {
                    src_id: String::from("M0Z1"),
                    output_port: 0,
                },
            ],
        };

        let cfg_yaml = serde_yaml::to_string(&cfg).unwrap();

        let expected_string = "inputs: 2\noutputs: 1\nneurons:\n- id: M0Z0\n  bias: 1\n  input_configs:\n  - capacity_max: 3\n    regeneration: 2\n    weight: 1\n    processing_delay: 0\n  - capacity_max: 1\n    regeneration: 1\n    weight: 2\n    processing_delay: 0\n- id: M0Z1\n  bias: 1\n  input_configs:\n  - capacity_max: 1\n    regeneration: 1\n    weight: 1\n    processing_delay: 0\nlinks:\n- !Input\n  input_port: 0\n  dst_id: M0Z0\n  dst_synapse_idx: 0\n- !Input\n  input_port: 1\n  dst_id: M0Z0\n  dst_synapse_idx: 1\n- !Inner\n  src_id: M0Z0\n  dst_id: M0Z1\n  dst_synapse_idx: 0\n- !Output\n  src_id: M0Z1\n  output_port: 0\n";
        assert_eq!(cfg_yaml, expected_string);
        Ok(())
    }

    #[test]
    #[ignore = "need change config structure"]
    fn should_deserialize_from_json_string_into_config_structure() {
        let cfg_json = json!({
            "inputs": 2,
            "outputs": 1,
            "neurons": [
                {
                    "id": "M0Z0",
                    "bias": 1,
                    "input_configs": [
                        {
                            "capacity_max": 1,
                            "regeneration": 1,
                            "weight": 1,
                            "processing_delay": 0
                        },
                        {
                            "capacity_max": 1,
                            "regeneration": 1,
                            "weight": 1,
                            "processing_delay": 0
                        }
                    ]
                },
                {
                    "id": "M0Z1",
                    "bias": 1,
                    "input_configs": [
                        {
                            "capacity_max": 1,
                            "regeneration": 1,
                            "weight": 1,
                            "processing_delay": 0
                        }
                    ]
                }
            ],
            "links": [
                { "Input": {
                        "input_port": 0,
                        "dst_id": "M0Z0",
                        "dst_synapse_idx": 0
                    }
                },
                {
                    "Input": {
                        "input_port": 1,
                        "dst_id": "M0Z0",
                        "dst_synapse_idx": 1
                    }
                },
                {
                    "Inner": {
                        "src_id": "M0Z0",
                        "dst_id": "M0Z1",
                        "dst_synapse_idx": 0
                    }
                },
                {
                    "Output": {
                        "src_id": "M0Z1",
                        "output_port": 0
                    }
                }
            ]
        })
        .to_string();

        let cfg: NetworkCfg<i16> = serde_json::from_str(&cfg_json).unwrap();
        assert_eq!(cfg.inputs, 2);
        assert_eq!(cfg.outputs, 1);
        assert_eq!(cfg.neurons.len(), 2);
        let neuron_cfg_0 = cfg.neurons.iter().nth(0).unwrap();
        assert_eq!(neuron_cfg_0.id, "M0Z0");
        // assert_eq!(neuron_cfg_0.input_configs.len(), 2);
        let neuron_cfg_1 = cfg.neurons.iter().nth(1).unwrap();
        assert_eq!(neuron_cfg_1.id, "M0Z1");
        // assert_eq!(neuron_cfg_1.input_configs.len(), 1);
    }

    #[test]
    #[ignore = "need change config structure"]
    fn should_deserialize_from_yaml_string_into_config_structure() {
        let cfg_yaml = "
            inputs: 2
            outputs: 1
            neurons:
            - id: M0Z0
              bias: 1
              input_configs:
              - capacity_max: 1
                regeneration: 1
                weight: 1
                processing_delay: 0
              - capacity_max: 1
                regeneration: 1
                weight: 1
                processing_delay: 0
            - id: M0Z1
              bias: 1
              input_configs:
              - capacity_max: 1
                regeneration: 1
                weight: 1
                processing_delay: 0
            links:
            - !Input
              input_port: 0
              dst_id: M0Z0
              dst_synapse_idx: 0
            - !Input
              input_port: 1
              dst_id: M0Z0
              dst_synapse_idx: 1
            - !Inner
              src_id: M0Z0
              dst_id: M0Z1
              dst_synapse_idx: 0
            - !Output
              src_id: M0Z1
              output_port: 0
        ";

        let cfg: NetworkCfg<i16> = serde_yaml::from_str(&cfg_yaml).unwrap();
        assert_eq!(cfg.inputs, 2);
        assert_eq!(cfg.outputs, 1);
        assert_eq!(cfg.neurons.len(), 2);
        let neuron_cfg_0 = cfg.neurons.iter().nth(0).unwrap();
        assert_eq!(neuron_cfg_0.id, "M0Z0");
        // assert_eq!(neuron_cfg_0.input_configs.len(), 2);
        let neuron_cfg_1 = cfg.neurons.iter().nth(1).unwrap();
        assert_eq!(neuron_cfg_1.id, "M0Z1");
        // assert_eq!(neuron_cfg_1.input_configs.len(), 1);
    }
}
