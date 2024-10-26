pub mod mocks {
    use std::{cell::RefCell, rc::Rc};

    use crate::rnn::common::component::Component;
    use crate::rnn::common::connectable::Connectable;
    use crate::rnn::common::container::Container;
    use crate::rnn::common::identity::Identity;
    use crate::rnn::common::signal_msg::SignalMessage;
    use crate::rnn::common::spec_type::SpecificationType;
    use crate::rnn::common::specialized::Specialized;

    #[derive(Debug, Default)]
    pub struct MockComponent {
        id: String,
        pub signal: i16,
        pub source_id: String,
    }

    impl Component for MockComponent {
        fn receive(&mut self, signal_msg: Box<SignalMessage>) {
            let SignalMessage(signal, source_id) = *signal_msg;
            self.signal = signal;
            self.source_id = *source_id;
        }

        fn send(&self, _signal: i16) {
            unimplemented!()
        }

        fn get_container(&self) -> Option<Rc<RefCell<dyn Container>>> {
            None
        }

        fn as_any(&self) -> &dyn std::any::Any {
            self
        }

        fn as_mut_any(&mut self) -> &mut dyn std::any::Any {
            self
        }
    }

    impl Identity for MockComponent {
        fn get_id(&self) -> String {
            self.id.clone()
        }
    }

    impl Connectable for MockComponent {}

    impl Specialized for MockComponent {
        fn get_spec_type(&self) -> SpecificationType {
            SpecificationType::Synapse
        }
    }
}
