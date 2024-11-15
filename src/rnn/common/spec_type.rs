use regex::Regex;

/// The types of specification.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum SpecificationType {
    Neuron,
    Network,
}

impl SpecificationType {
    /// Check if id is valid for component with specified spec type
    pub fn is_id_valid(&self, id: &str) -> bool {
        let rex_pattern = match *self {
            Self::Neuron => r"^M\d+Z\d+$",
            Self::Network => r"^M\d+$",
        };

        Regex::new(rex_pattern).is_ok_and(|rex| rex.is_match(id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod is_id_valid_test_suite {
        use super::*;

        mod for_neuron {
            use super::*;

            #[test]
            fn positive_test() {
                assert!(SpecificationType::Neuron.is_id_valid("M10Z0"));
            }

            #[test]
            fn negative_test_test() {
                assert!(!SpecificationType::Neuron.is_id_valid("M10Z0A0"));
            }
        }

        mod for_network {
            use super::*;

            #[test]
            fn positive_test() {
                assert!(SpecificationType::Network.is_id_valid("M10"));
            }

            #[test]
            fn negative_test_test() {
                assert!(!SpecificationType::Network.is_id_valid("M10Z0A0"));
            }
        }
    }
}
