pub mod mocks {
    use std::{cell::RefCell, rc::Rc};
    use std::any::Any;

    use as_any::AsAny;
    use as_any_derive::AsAny;

    use crate::rnn::common::component::Component;
    use crate::rnn::common::connectable::Connectable;
    use crate::rnn::common::container::Container;
    use crate::rnn::common::identity::Identity;
    use crate::rnn::common::signal_msg::SignalMessage;
    use crate::rnn::common::spec_type::SpecificationType;
    use crate::rnn::common::specialized::Specialized;

    #[derive(Debug, Default, AsAny)]
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
