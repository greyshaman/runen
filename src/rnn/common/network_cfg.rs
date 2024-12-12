use serde::{Deserialize, Serialize};

use super::input_cfg::InputCfg;

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

#[derive(Debug, Serialize, Deserialize)]
pub struct NeuronCfg {
    pub id: String,
    pub input_configs: Vec<InputCfg>,
}

/// The network config structure used to describe neuron set and connections between them.
#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkCfg {
    inputs: usize,
    outputs: usize,
    neurons: Vec<NeuronCfg>,
    links: Vec<LinkCfg>,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn should_serialize_config_into_json_string() {
        let neuron_cfgs = vec![
            NeuronCfg {
                id: String::from("M0Z0"),
                input_configs: vec![
                    InputCfg::new(2, 2, 1).unwrap(),
                    InputCfg::new(1, 1, 2).unwrap(),
                ],
            },
            NeuronCfg {
                id: String::from("M0Z1"),
                input_configs: vec![InputCfg::new(1, 1, 1).unwrap()],
            },
            NeuronCfg {
                id: String::from("M0Z2"),
                input_configs: vec![InputCfg::new(3, 2, 1).unwrap()],
            },
            NeuronCfg {
                id: String::from("M0Z3"),
                input_configs: vec![
                    InputCfg::new(1, 1, 1).unwrap(),
                    InputCfg::new(3, 1, 2).unwrap(),
                ],
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

        let expected_string = "{\"inputs\":3,\"outputs\":2,\"neurons\":[{\"id\":\"M0Z0\",\"input_configs\":[{\"capacity_max\":2,\"regeneration\":2,\"weight\":1},{\"capacity_max\":1,\"regeneration\":1,\"weight\":2}]},{\"id\":\"M0Z1\",\"input_configs\":[{\"capacity_max\":1,\"regeneration\":1,\"weight\":1}]},{\"id\":\"M0Z2\",\"input_configs\":[{\"capacity_max\":3,\"regeneration\":2,\"weight\":1}]},{\"id\":\"M0Z3\",\"input_configs\":[{\"capacity_max\":1,\"regeneration\":1,\"weight\":1},{\"capacity_max\":3,\"regeneration\":1,\"weight\":2}]}],\"links\":[{\"Input\":{\"input_port\":0,\"dst_id\":\"M0Z0\",\"dst_synapse_idx\":0}},{\"Input\":{\"input_port\":1,\"dst_id\":\"M0Z0\",\"dst_synapse_idx\":1}},{\"Input\":{\"input_port\":2,\"dst_id\":\"M0Z1\",\"dst_synapse_idx\":0}},{\"Inner\":{\"src_id\":\"M0Z0\",\"dst_id\":\"M0Z2\",\"dst_synapse_idx\":0}},{\"Inner\":{\"src_id\":\"M0Z1\",\"dst_id\":\"M0Z3\",\"dst_synapse_idx\":0}},{\"Inner\":{\"src_id\":\"M0Z1\",\"dst_id\":\"M0Z3\",\"dst_synapse_idx\":1}},{\"Output\":{\"src_id\":\"M0Z2\",\"output_port\":0}},{\"Output\":{\"src_id\":\"M0Z3\",\"output_port\":1}}]}";
        assert_eq!(cfg_json, expected_string);
    }

    #[test]
    fn should_serialize_config_into_yaml_string() {
        let neuron_cfgs = vec![
            NeuronCfg {
                id: String::from("M0Z0"),
                input_configs: vec![
                    InputCfg::new(3, 2, 1).unwrap(),
                    InputCfg::new(1, 1, 2).unwrap(),
                ],
            },
            NeuronCfg {
                id: String::from("M0Z1"),
                input_configs: vec![InputCfg::new(1, 1, 1).unwrap()],
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

        let expected_string = "inputs: 2\noutputs: 1\nneurons:\n- id: M0Z0\n  input_configs:\n  - capacity_max: 3\n    regeneration: 2\n    weight: 1\n  - capacity_max: 1\n    regeneration: 1\n    weight: 2\n- id: M0Z1\n  input_configs:\n  - capacity_max: 1\n    regeneration: 1\n    weight: 1\nlinks:\n- !Input\n  input_port: 0\n  dst_id: M0Z0\n  dst_synapse_idx: 0\n- !Input\n  input_port: 1\n  dst_id: M0Z0\n  dst_synapse_idx: 1\n- !Inner\n  src_id: M0Z0\n  dst_id: M0Z1\n  dst_synapse_idx: 0\n- !Output\n  src_id: M0Z1\n  output_port: 0\n";
        assert_eq!(cfg_yaml, expected_string);
    }

    #[test]
    fn should_deserialize_from_json_string_into_config_structure() {
        let cfg_json = json!({
            "inputs": 2,
            "outputs": 1,
            "neurons": [
                {
                    "id": "M0Z0",
                    "input_configs": [
                        {
                            "capacity_max": 1,
                            "regeneration": 1,
                            "weight": 1
                        },
                        {
                            "capacity_max": 1,
                            "regeneration": 1,
                            "weight": 1
                        }
                    ]
                },
                {
                    "id": "M0Z1",
                    "input_configs": [
                        {
                            "capacity_max": 1,
                            "regeneration": 1,
                            "weight": 1
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

        let cfg: NetworkCfg = serde_json::from_str(&cfg_json).unwrap();
        assert_eq!(cfg.inputs, 2);
        assert_eq!(cfg.outputs, 1);
        assert_eq!(cfg.neurons.len(), 2);
        let neuron_cfg_0 = cfg.neurons.iter().nth(0).unwrap();
        assert_eq!(neuron_cfg_0.id, "M0Z0");
        assert_eq!(neuron_cfg_0.input_configs.len(), 2);
        let neuron_cfg_1 = cfg.neurons.iter().nth(1).unwrap();
        assert_eq!(neuron_cfg_1.id, "M0Z1");
        assert_eq!(neuron_cfg_1.input_configs.len(), 1);
    }

    #[test]
    fn should_deserialize_from_yaml_string_into_config_structure() {
        let cfg_yaml = "
            inputs: 2
            outputs: 1
            neurons:
            - id: M0Z0
              input_configs:
              - capacity_max: 1
                regeneration: 1
                weight: 1
              - capacity_max: 1
                regeneration: 1
                weight: 1
            - id: M0Z1
              input_configs:
              - capacity_max: 1
                regeneration: 1
                weight: 1
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

        let cfg: NetworkCfg = serde_yaml::from_str(&cfg_yaml).unwrap();
        assert_eq!(cfg.inputs, 2);
        assert_eq!(cfg.outputs, 1);
        assert_eq!(cfg.neurons.len(), 2);
        let neuron_cfg_0 = cfg.neurons.iter().nth(0).unwrap();
        assert_eq!(neuron_cfg_0.id, "M0Z0");
        assert_eq!(neuron_cfg_0.input_configs.len(), 2);
        let neuron_cfg_1 = cfg.neurons.iter().nth(1).unwrap();
        assert_eq!(neuron_cfg_1.id, "M0Z1");
        assert_eq!(neuron_cfg_1.input_configs.len(), 1);
    }
}
