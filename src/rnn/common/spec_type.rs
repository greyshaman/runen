use regex::Regex;

/// The types of component specification.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum SpecificationType {
    Network,
    InputLayer,
    HiddenLayer,
    OutputLayer,
    InputPort,
    Neuron,
    OutputPort,
}

impl SpecificationType {
    /// Check if id is valid for component with specified spec type
    ///
    /// # Examples
    ///
    /// ```
    /// use librunen::rnn::common::spec_type::SpecificationType;
    ///
    /// let spec_type = SpecificationType::Neuron;
    ///
    /// assert!(spec_type.is_id_valid("N_350::HL_444::Z_231233"));
    /// ```
    pub fn is_id_valid(&self, id: &str) -> bool {
        let rex_pattern = match *self {
            Self::Network => r"^N_\d+$",
            Self::InputLayer => r"^N_\d+::IL_\d+$",
            Self::HiddenLayer => r"^N_\d+::HL_\d+$",
            Self::OutputLayer => r"^N_\d+::OL_\d+$",
            Self::InputPort => r"^N_\d+::IL_\d+::IP_\d+$",
            Self::OutputPort => r"^N_\d+::OL_\d+::OP_\d+$",
            Self::Neuron => r"^N_\d+::HL_\d+::Z_\d+$",
        };

        Regex::new(rex_pattern).is_ok_and(|rex| rex.is_match(id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod is_id_valid_test_suite {
        use super::*;

        mod for_network {
            use super::*;

            #[test]
            fn positive_test() {
                assert!(SpecificationType::Network.is_id_valid("N_10"));
            }

            #[test]
            fn negative_test_test() {
                assert!(!SpecificationType::Network.is_id_valid("M10Z0A0"));
            }
        }

        mod for_input_layer {
            use super::*;

            #[test]
            fn positive_test() {
                assert!(SpecificationType::InputLayer.is_id_valid("N_10::IL_0"));
            }

            #[test]
            fn negative_test_test() {
                assert!(!SpecificationType::InputLayer.is_id_valid("M10Z0A0"));
            }
        }

        mod for_hidden_layer {
            use super::*;

            #[test]
            fn positive_test() {
                assert!(SpecificationType::HiddenLayer.is_id_valid("N_10::HL_0"));
            }

            #[test]
            fn negative_test_test() {
                assert!(!SpecificationType::HiddenLayer.is_id_valid("M10Z0A0"));
            }
        }

        mod for_output_layer {
            use super::*;

            #[test]
            fn positive_test() {
                assert!(SpecificationType::OutputLayer.is_id_valid("N_10::OL_0"));
            }

            #[test]
            fn negative_test_test() {
                assert!(!SpecificationType::OutputLayer.is_id_valid("M10Z0A0"));
            }
        }

        mod for_input_port {
            use super::*;

            #[test]
            fn positive_test() {
                assert!(SpecificationType::InputPort.is_id_valid("N_10::IL_0::IP_0"));
            }

            #[test]
            fn negative_test_test() {
                assert!(!SpecificationType::InputPort.is_id_valid("M10Z0A0"));
            }
        }

        mod for_output_port {
            use super::*;

            #[test]
            fn positive_test() {
                assert!(SpecificationType::OutputPort.is_id_valid("N_10::OL_0::OP_0"));
            }

            #[test]
            fn negative_test_test() {
                assert!(!SpecificationType::OutputPort.is_id_valid("M10Z0A0"));
            }
        }

        mod for_neuron {
            use super::*;

            #[test]
            fn positive_test() {
                assert!(SpecificationType::Neuron.is_id_valid("N_10::HL_0::Z_0"));
            }

            #[test]
            fn negative_test_test() {
                assert!(!SpecificationType::Neuron.is_id_valid("M10Z0A0"));
            }
        }
    }
}
