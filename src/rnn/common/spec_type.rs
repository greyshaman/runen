use regex::Regex;

/// The types of specification.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum SpecificationType {
    Synapse,
    Dendrite,
    Neurosoma,
    Axon,
    InputTerminator,
    OutputTerminator,
    Neuron,
    Receptor,
    Activator,
    Network,
}

impl SpecificationType {
    pub fn is_siblings_allowed(&self) -> bool {
        match *self {
            SpecificationType::Neurosoma | SpecificationType::Axon => false,
            _ => true,
        }
    }

    pub fn is_id_valid(&self, id: &str) -> bool {
        let rex_pattern = match *self {
            Self::Synapse => r"^M\d+Z\d+A\d+$",
            Self::Dendrite => r"^M\d+Z\d+C\d+$",
            Self::Neurosoma => r"^M\d+Z\d+G\d+$",
            Self::Axon => r"^M\d+Z\d+E\d+$",
            Self::InputTerminator => r"^M\d+Y\d+I\d+$",
            Self::OutputTerminator => r"^M\d+X\d+O\d+$",
            Self::Neuron => r"^M\d+Z\d+$",
            Self::Receptor => r"^M\d+Y\d+$",
            Self::Activator => r"^M\d+X\d+$",
            Self::Network => r"^M\d+$",
        };

        Regex::new(rex_pattern).is_ok_and(|rex| rex.is_match(id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod is_siblings_allowed_test_suite {
        use super::*;

        #[test]
        fn not_allowed_for_aggregator() {
            assert!(!SpecificationType::Neurosoma.is_siblings_allowed())
        }

        #[test]
        fn not_allowed_for_emitter() {
            assert!(!SpecificationType::Axon.is_siblings_allowed())
        }

        #[test]
        fn allowed_for_acceptor() {
            assert!(SpecificationType::Synapse.is_siblings_allowed())
        }

        #[test]
        fn allowed_for_collector() {
            assert!(SpecificationType::Dendrite.is_siblings_allowed())
        }

        #[test]
        fn allowed_for_input_terminator() {
            assert!(SpecificationType::InputTerminator.is_siblings_allowed())
        }

        #[test]
        fn allowed_for_output_terminator() {
            assert!(SpecificationType::OutputTerminator.is_siblings_allowed())
        }

        #[test]
        fn allowed_for_container() {
            assert!(SpecificationType::Neuron.is_siblings_allowed())
        }

        #[test]
        fn allowed_for_receptor() {
            assert!(SpecificationType::Receptor.is_siblings_allowed())
        }

        #[test]
        fn allowed_for_activator() {
            assert!(SpecificationType::Activator.is_siblings_allowed())
        }

        #[test]
        fn allowed_for_media() {
            assert!(SpecificationType::Network.is_siblings_allowed())
        }
    }

    mod is_id_valid_test_suite {
        use super::*;

        mod for_acceptor {
            use super::*;

            #[test]
            fn positive_test() {
                assert!(SpecificationType::Synapse.is_id_valid("M10Z0A0"));
            }

            #[test]
            fn negative_test_test() {
                assert!(!SpecificationType::Synapse.is_id_valid("M10J0A0"));
            }
        }

        mod for_collector {
            use super::*;

            #[test]
            fn positive_test() {
                assert!(SpecificationType::Dendrite.is_id_valid("M10Z0C0"));
            }

            #[test]
            fn negative_test_test() {
                assert!(!SpecificationType::Dendrite.is_id_valid("M10J0A0"));
            }
        }

        mod for_aggregator {
            use super::*;

            #[test]
            fn positive_test() {
                assert!(SpecificationType::Neurosoma.is_id_valid("M10Z0G0"));
            }

            #[test]
            fn negative_test_test() {
                assert!(!SpecificationType::Neurosoma.is_id_valid("M10J0A0"));
            }
        }

        mod for_emitter {
            use super::*;

            #[test]
            fn positive_test() {
                assert!(SpecificationType::Axon.is_id_valid("M10Z0E0"));
            }

            #[test]
            fn negative_test_test() {
                assert!(!SpecificationType::Axon.is_id_valid("M10J0A0"));
            }
        }

        mod for_iput_terminator {
            use super::*;

            #[test]
            fn positive_test() {
                assert!(SpecificationType::InputTerminator.is_id_valid("M10Y0I0"));
            }

            #[test]
            fn negative_test_test() {
                assert!(!SpecificationType::InputTerminator.is_id_valid("M10J0A0"));
            }
        }

        mod for_output_terminator {
            use super::*;

            #[test]
            fn positive_test() {
                assert!(SpecificationType::OutputTerminator.is_id_valid("M10X0O0"));
            }

            #[test]
            fn negative_test_test() {
                assert!(!SpecificationType::OutputTerminator.is_id_valid("M10J0A0"));
            }
        }

        mod for_container {
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

        mod for_media {
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
