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
        SpecificationType::Synapse => ('A', r"^M\d+Z\d+$"),
        SpecificationType::Dendrite => ('C', r"^M\d+Z\d+$"),
        SpecificationType::Neurosoma => ('G', r"^M\d+Z\d+$"),
        SpecificationType::Axon => ('E', r"^M\d+Z\d+$"),
        SpecificationType::InputTerminator => ('I', r"^M\d+Y\d+$"),
        SpecificationType::OutputTerminator => ('O', r"^M\d+X\d+$"),
        SpecificationType::Neuron => ('Z', r"^M\d+$"),
        SpecificationType::Receptor => ('Y', r"^M\d+$"),
        SpecificationType::Activator => ('X', r"^M\d+$"),
        SpecificationType::Network => ('M', r"^$"),
    };

    if !is_match_to_regexp(container_id, &regex_pattern) {
        return Err(Box::new(RnnError::NotSupportedArgValue));
    }
    Ok(format!(
        "{}{}{}",
        container_id, specification_prefix, entity_num_id
    ))
}

/// Extract number fraction of component Id
pub fn get_component_id_fraction(
    id: &str,
    spec_type: &SpecificationType,
) -> Result<usize, Box<dyn Error>> {
    if id.len() == 0 {
        return Err(Box::new(RnnError::NotPresent(String::from(
            "Empty string present",
        ))));
    }

    let r_pattern: &str = match spec_type {
        SpecificationType::Synapse => r"^M\d+Z\d+A(\d+)$",
        SpecificationType::Dendrite => r"^M\d+Z\d+C(\d+)$",
        SpecificationType::Neurosoma => r"^M\d+Z\d+G(\d+)$",
        SpecificationType::Axon => r"^M\d+Z\d+E(\d+)$",
        SpecificationType::Neuron => r"^M\d+Z(\d+)$",
        _ => return Err(Box::new(RnnError::NotSupportedArgValue)),
    };

    let rex = Regex::new(r_pattern)?;
    let captures = rex
        .captures(id)
        .ok_or_else(|| Box::new(RnnError::NotSupportedArgValue))?;

    if &captures.len() < &2 {
        return Err(Box::new(RnnError::PatternNotFound));
    }
    let result = captures[1].parse::<usize>()?;
    Ok(result)
}

pub fn check_id_on_siblings(id: &str, spec_type: &SpecificationType) -> bool {
    spec_type.is_siblings_allowed()
        || get_component_id_fraction(id, spec_type).is_ok_and(|id_fraction| id_fraction == 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    mod is_match_to_regexp_test_suite {
        use super::*;

        #[test]
        fn positive_test_for_acceptor_id() {
            let pattern = r"^M\d+Z\d+C\d+$";
            let sample = "M0Z34C151";
            assert!(is_match_to_regexp(sample, pattern));
        }

        #[test]
        fn negative_test_for_acceptor_id() {
            let pattern = r"^M\d+Z\d+C\d+$";
            let sample = "Z34C151";
            assert!(!is_match_to_regexp(sample, pattern));
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

        mod for_media {
            use super::*;

            #[test]
            fn should_generate_correct_id_for_initial_entity() {
                let expected_id = "M0";
                assert_eq!(
                    gen_id_by_spec_type("", 0, &SpecificationType::Network)
                        .unwrap()
                        .as_str(),
                    expected_id
                );
            }

            #[test]
            fn should_generate_correct_id_for_sequence_entity() {
                let expected_id = "M5";
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

        mod for_container {
            use super::*;

            #[test]
            fn should_generate_correct_id_for_initial_entity() {
                let expected_id = "M1Z0";
                assert_eq!(
                    gen_id_by_spec_type("M1", 0, &SpecificationType::Neuron)
                        .unwrap()
                        .as_str(),
                    expected_id
                );
            }

            #[test]
            fn should_generate_correct_id_for_sequence_entity() {
                let expected_id = "M5Z5";
                assert_eq!(
                    gen_id_by_spec_type("M5", 5, &SpecificationType::Neuron)
                        .unwrap()
                        .as_str(),
                    expected_id
                );
            }

            #[test]
            fn should_throw_error_with_incorrect_container_id_for_sequence_entity() {
                assert!(gen_id_by_spec_type("", 5, &SpecificationType::Neuron).is_err());
            }
        }

        mod for_collector {
            use super::*;

            #[test]
            fn should_generate_correct_id_for_initial_entity() {
                let expected_id = "M1Z0C0";
                assert_eq!(
                    gen_id_by_spec_type("M1Z0", 0, &SpecificationType::Dendrite)
                        .unwrap()
                        .as_str(),
                    expected_id
                );
            }

            #[test]
            fn should_generate_correct_id_for_sequence_entity() {
                let expected_id = "M5Z5C5";
                assert_eq!(
                    gen_id_by_spec_type("M5Z5", 5, &SpecificationType::Dendrite)
                        .unwrap()
                        .as_str(),
                    expected_id
                );
            }

            #[test]
            fn should_panic_with_incorrect_container_id_for_sequence_entity() {
                assert!(gen_id_by_spec_type("E0", 5, &SpecificationType::Dendrite).is_err());
            }
        }

        mod for_acceptor {
            use super::*;

            #[test]
            fn should_generate_correct_id_for_initial_entity() {
                let expected_id = "M1Z0A0";
                assert_eq!(
                    gen_id_by_spec_type("M1Z0", 0, &SpecificationType::Synapse)
                        .unwrap()
                        .as_str(),
                    expected_id
                );
            }

            #[test]
            fn should_generate_correct_id_for_sequence_entity() {
                let expected_id = "M5Z5A5";
                assert_eq!(
                    gen_id_by_spec_type("M5Z5", 5, &SpecificationType::Synapse)
                        .unwrap()
                        .as_str(),
                    expected_id
                );
            }

            #[test]
            fn should_panic_with_incorrect_container_id_for_sequence_entity() {
                assert!(gen_id_by_spec_type("E0", 5, &SpecificationType::Synapse).is_err());
            }
        }

        mod for_aggregator {
            use super::*;

            #[test]
            fn should_generate_correct_id_for_initial_entity() {
                let expected_id = "M1Z0G0";
                assert_eq!(
                    gen_id_by_spec_type("M1Z0", 0, &SpecificationType::Neurosoma)
                        .unwrap()
                        .as_str(),
                    expected_id
                );
            }

            #[test]
            fn should_generate_correct_id_for_sequence_entity() {
                let expected_id = "M5Z5G5";
                assert_eq!(
                    gen_id_by_spec_type("M5Z5", 5, &SpecificationType::Neurosoma)
                        .unwrap()
                        .as_str(),
                    expected_id
                );
            }

            #[test]
            fn should_panic_with_incorrect_container_id_for_sequence_entity() {
                assert!(gen_id_by_spec_type("E0", 5, &SpecificationType::Neurosoma).is_err());
            }
        }

        mod for_emitter {
            use super::*;

            #[test]
            fn should_generate_correct_id_for_initial_entity() {
                let expected_id = "M1Z0E0";
                assert_eq!(
                    gen_id_by_spec_type("M1Z0", 0, &SpecificationType::Axon)
                        .unwrap()
                        .as_str(),
                    expected_id
                );
            }

            #[test]
            fn should_generate_correct_id_for_sequence_entity() {
                let expected_id = "M5Z5E5";
                assert_eq!(
                    gen_id_by_spec_type("M5Z5", 5, &SpecificationType::Axon)
                        .unwrap()
                        .as_str(),
                    expected_id
                );
            }

            #[test]
            fn should_panic_with_incorrect_container_id_for_sequence_entity() {
                assert!(gen_id_by_spec_type("E0", 5, &SpecificationType::Axon).is_err());
            }
        }
    }

    mod get_component_id_fraction_test_suite {
        use super::*;

        mod for_container {
            use super::*;

            #[test]
            fn positive_test() {
                assert_eq!(
                    get_component_id_fraction("M0Z1", &SpecificationType::Neuron).unwrap(),
                    1
                );
            }

            #[test]
            fn negative_test() {
                assert!(get_component_id_fraction("M1", &SpecificationType::Neuron).is_err());
            }
        }

        mod for_acceptor {
            use super::*;

            #[test]
            fn positive_test() {
                assert_eq!(
                    get_component_id_fraction("M0Z1A12", &SpecificationType::Synapse).unwrap(),
                    12
                );
            }

            #[test]
            fn negative_test() {
                assert!(get_component_id_fraction("M1Z3X4", &SpecificationType::Synapse).is_err());
            }
        }

        mod for_collector {
            use super::*;

            #[test]
            fn positive_test() {
                assert_eq!(
                    get_component_id_fraction("M0Z1C12", &SpecificationType::Dendrite).unwrap(),
                    12
                );
            }

            #[test]
            fn negative_test() {
                assert!(get_component_id_fraction("M1Z3X4", &SpecificationType::Dendrite).is_err());
            }
        }

        mod for_aggregator {
            use super::*;

            #[test]
            fn positive_test() {
                assert_eq!(
                    get_component_id_fraction("M0Z1G12", &SpecificationType::Neurosoma).unwrap(),
                    12
                );
            }

            #[test]
            fn negative_test() {
                assert!(
                    get_component_id_fraction("M1Z3X4", &SpecificationType::Neurosoma).is_err()
                );
            }
        }

        mod for_emitter {
            use super::*;

            #[test]
            fn positive_test() {
                assert_eq!(
                    get_component_id_fraction("M0Z1E12", &SpecificationType::Axon).unwrap(),
                    12
                );
            }

            #[test]
            fn negative_test() {
                assert!(get_component_id_fraction("M1Z3X4", &SpecificationType::Axon).is_err());
            }
        }
    }

    mod check_id_on_siblings_test_suite {
        use super::*;

        #[test]
        fn siblings_accepted_for_first_acceptor() {
            assert!(check_id_on_siblings("M1Z3A0", &SpecificationType::Synapse));
        }

        #[test]
        fn siblings_accepted_for_next_acceptor() {
            assert!(check_id_on_siblings("M1Z3A1", &SpecificationType::Synapse));
        }
        #[test]
        fn siblings_accepted_for_first_aggregator() {
            assert!(check_id_on_siblings(
                "M1Z3G0",
                &SpecificationType::Neurosoma
            ));
        }

        #[test]
        fn siblings_denied_for_next_aggregator() {
            assert!(!check_id_on_siblings(
                "M1Z3G1",
                &SpecificationType::Neurosoma
            ));
        }
    }
}
