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
        SpecificationType::Neuron => ('Z', r"^M\d+$"),
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
        return Err(Box::new(RnnError::ExpectedDataNotPresent(String::from(
            "Empty string present",
        ))));
    }

    let r_pattern: &str = match spec_type {
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
        fn positive_test_for_neuron_id() {
            let pattern = r"^M\d+Z\d+$";
            let sample = "M0Z34";
            assert!(is_match_to_regexp(sample, pattern));
        }

        #[test]
        fn negative_test_for_neuron_id() {
            let pattern = r"^M\d+Z\d+$";
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

        mod for_network {
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

        mod for_neuron {
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
    }

    mod get_component_id_fraction_test_suite {
        use super::*;

        mod for_neuron {
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
    }
}
