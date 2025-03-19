use std::error::Error;

use regex::Regex;

use super::{rnn_error::RnnError, spec_type::SpecificationType};

pub fn is_match_to_regexp(sample: &str, rex_pattern: &str) -> bool {
    Regex::new(rex_pattern).is_ok_and(|rex| rex.is_match(sample))
}

/// Function generate id in format "Z123A2" based on container Id and provided available Id
pub fn gen_id_by_spec_type(
    container_id: &str,
    entity_num_id: usize,
    my_spec_type: &SpecificationType,
) -> Result<String, Box<dyn Error>> {
    let (specification_prefix, regex_pattern) = match my_spec_type {
        SpecificationType::Network => ("N", r"^$"),
        SpecificationType::InputLayer => ("IL", r"^N_\d+$"),
        SpecificationType::HiddenLayer => ("HL", r"^N_\d+$"),
        SpecificationType::OutputLayer => ("OL", r"^N_\d+$"),
        SpecificationType::InputPort => ("IP", r"^N_\d+::IL_\d+$"),
        SpecificationType::OutputPort => ("OP", r"^N_\d+::OL_\d+$"),
        SpecificationType::Neuron => ("Z", r"^N_\d+::HL_\d+$"),
    };

    if !is_match_to_regexp(container_id, &regex_pattern) {
        return Err(Box::new(RnnError::NotSupportedArgValue));
    }
    let id = if container_id.is_empty() {
        format!("{}_{}", specification_prefix, entity_num_id)
    } else {
        format!(
            "{}::{}_{}",
            container_id, specification_prefix, entity_num_id
        )
    };

    Ok(id)
}

/// Extract number fraction of component Id
pub fn extract_entity_id_fraction(
    id: &str,
    spec_type: &SpecificationType,
) -> Result<usize, Box<dyn Error>> {
    if id.len() == 0 {
        return Err(Box::new(RnnError::ExpectedDataNotPresent(String::from(
            "Empty string present",
        ))));
    }

    let r_pattern: &str = match spec_type {
        SpecificationType::Network => r"^N_(\d+)$",
        SpecificationType::InputLayer => r"^N_\d+::IL_(\d+)$",
        SpecificationType::HiddenLayer => r"^N_\d+::HL_(\d+)$",
        SpecificationType::OutputLayer => r"^N_\d+::OL_(\d+)$",
        SpecificationType::InputPort => r"^N_\d+::IL_\d+::IP_(\d+)$",
        SpecificationType::Neuron => r"^N_\d+::HL_\d+::Z_(\d+)$",
        SpecificationType::OutputPort => r"^N_\d+::OL_\d+::OP_(\d+)$",
    };

    let rex = Regex::new(r_pattern)?;
    let captures = rex
        .captures(id)
        .ok_or_else(|| Box::new(RnnError::IncorrectIdFormat))?;

    if &captures.len() < &2 {
        return Err(Box::new(RnnError::PatternNotFound));
    }
    let result = captures[1].parse::<usize>()?;
    Ok(result)
}

/// Extract neuron's id part from component id. E.g. from M0Z0C12 -> M0Z0
pub fn extract_neuron_id_from(id: &str) -> Option<String> {
    Regex::new(r"^(M\d+Z\d+).*$")
        .unwrap()
        .captures(id)
        .and_then(|caps| {
            if caps.len() > 1 {
                Some(caps[1].to_string())
            } else {
                None
            }
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    mod is_match_to_regexp_test_suite {
        use super::*;

        #[test]
        fn positive_test_for_network_id() {
            let pattern = r"^N_\d+$";
            let sample = "N_1234";
            assert!(is_match_to_regexp(sample, pattern));
        }

        #[test]
        fn negative_test_for_network_id() {
            let pattern = r"^N_\d+$";
            let sample = "M123";
            assert!(!is_match_to_regexp(sample, pattern));
        }

        #[test]
        fn positive_test_for_input_layer_id() {
            let pattern = r"^N_\d+::IL_\d+$";
            let sample = "N_0::IL_0";
            assert!(is_match_to_regexp(sample, pattern));
        }

        #[test]
        fn positive_test_for_hidden_layer_id() {
            let pattern = r"^N_\d+::HL_\d+$";
            let sample = "N_0::HL_0";
            assert!(is_match_to_regexp(sample, pattern));
        }

        #[test]
        fn positive_test_for_output_layer_id() {
            let pattern = r"^N_\d+::OL_\d+$";
            let sample = "N_0::OL_0";
            assert!(is_match_to_regexp(sample, pattern));
        }

        #[test]
        fn positive_test_for_input_port_id() {
            let pattern = r"^N_\d+::IL_\d+::IP_\d+$";
            let sample = "N_0::IL_0::IP_34";
            assert!(is_match_to_regexp(sample, pattern));
        }

        #[test]
        fn positive_test_for_output_port_id() {
            let pattern = r"^N_\d+::OL_\d+::OP_\d+$";
            let sample = "N_0::OL_0::OP_34";
            assert!(is_match_to_regexp(sample, pattern));
        }

        #[test]
        fn positive_test_for_neuron_id() {
            let pattern = r"^N_\d+::HL_\d+::Z_\d+$";
            let sample = "N_0::HL_0::Z_34";
            assert!(is_match_to_regexp(sample, pattern));
        }

        #[test]
        fn positive_test_for_empty_string_() {
            let pattern = r"^$";
            let sample = "";
            assert!(is_match_to_regexp(sample, pattern));
        }

        #[test]
        fn negative_test_for_empty_string() {
            let pattern = r"^$";
            let sample = "M123";
            assert!(!is_match_to_regexp(sample, pattern));
        }
    }

    mod gen_id_by_spec_type_test_suite {
        use super::*;

        mod for_network {
            use super::*;

            #[test]
            fn should_generate_correct_id_for_network_entry() {
                let expected_id = "N_5";
                assert_eq!(
                    gen_id_by_spec_type("", 5, &SpecificationType::Network)
                        .unwrap()
                        .as_str(),
                    expected_id
                );
            }

            #[test]
            fn should_throw_error_with_incorrect_container_id_for_sequence_entity() {
                assert!(gen_id_by_spec_type("27637263", 5, &SpecificationType::Network).is_err());
            }
        }

        mod for_input_layer {
            use super::*;

            #[test]
            fn should_generate_correct_id_for_input_layer_entry() {
                let expected_id = "N_5::IL_2";
                assert_eq!(
                    gen_id_by_spec_type("N_5", 2, &SpecificationType::InputLayer)
                        .unwrap()
                        .as_str(),
                    expected_id
                );
            }
        }

        mod for_hidden_layer {
            use super::*;

            #[test]
            fn should_generate_correct_id_for_hidden_layer_entry() {
                let expected_id = "N_5::HL_2";
                assert_eq!(
                    gen_id_by_spec_type("N_5", 2, &SpecificationType::HiddenLayer)
                        .unwrap()
                        .as_str(),
                    expected_id
                );
            }
        }

        mod for_output_layer {
            use super::*;

            #[test]
            fn should_generate_correct_id_for_output_layer_entry() {
                let expected_id = "N_5::OL_2";
                assert_eq!(
                    gen_id_by_spec_type("N_5", 2, &SpecificationType::OutputLayer)
                        .unwrap()
                        .as_str(),
                    expected_id
                );
            }
        }

        mod for_input_port {
            use super::*;

            #[test]
            fn should_generate_correct_id_for_input_port_entry() {
                let expected_id = "N_5::IL_2::IP_42";
                assert_eq!(
                    gen_id_by_spec_type("N_5::IL_2", 42, &SpecificationType::InputPort)
                        .unwrap()
                        .as_str(),
                    expected_id
                );
            }
        }

        mod for_output_port {
            use super::*;

            #[test]
            fn should_generate_correct_id_for_output_port_entry() {
                let expected_id = "N_5::OL_2::OP_42";
                assert_eq!(
                    gen_id_by_spec_type("N_5::OL_2", 42, &SpecificationType::OutputPort)
                        .unwrap()
                        .as_str(),
                    expected_id
                );
            }
        }

        mod for_neuron {
            use super::*;

            #[test]
            fn should_generate_correct_id_for_neuron_entry() {
                let expected_id = "N_5::HL_2::Z_42";
                assert_eq!(
                    gen_id_by_spec_type("N_5::HL_2", 42, &SpecificationType::Neuron)
                        .unwrap()
                        .as_str(),
                    expected_id
                );
            }
        }
    }

    mod get_entry_id_fraction_test_suite {
        use super::*;

        mod for_network {
            use super::*;

            #[test]
            fn positive_test() {
                assert_eq!(
                    extract_entity_id_fraction("N_1", &SpecificationType::Network).unwrap(),
                    1
                );
            }
        }

        mod for_neuron {
            use super::*;

            #[test]
            fn positive_test() {
                assert_eq!(
                    extract_entity_id_fraction("N_0::HL_0::Z_1", &SpecificationType::Neuron)
                        .unwrap(),
                    1
                );
            }

            #[test]
            fn negative_test() {
                assert!(
                    extract_entity_id_fraction("N_0::IL_0::Z_1", &SpecificationType::Neuron)
                        .is_err()
                );
            }
        }
    }
}
