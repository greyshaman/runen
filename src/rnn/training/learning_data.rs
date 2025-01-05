use std::{
    collections::{hash_map::Entry, HashMap, HashSet},
    hash::Hash,
};

use serde::{Deserialize, Serialize};

use super::{reconstructible::Reconstructible, splittable::Splittable};

/// It is proposed to store a set of initial facts and their corresponding solutions in the form
/// of a HashMap<Solution, Vec<Fact>>, where the examples are indexed by the solution that
/// the neural network should receive during the learning process.
#[derive(Debug, Serialize, Deserialize)]
pub struct LearningData<F, S>
where
    F: Splittable + Eq + Hash,
    S: Reconstructible + Eq + Hash,
{
    /// Name of training data
    name: String,

    /// Input layer dimension
    input_dimension: usize,

    /// Output layer dimension
    output_dimension: usize,

    /// The set of trining data
    knowledge: HashMap<S, HashSet<F>>,
}

impl<F, S> LearningData<F, S>
where
    F: Splittable + Eq + Hash + Clone,
    S: Reconstructible + Eq + Hash,
{
    pub fn new(name: &str) -> Self {
        LearningData {
            name: name.to_string(),
            input_dimension: 1,
            output_dimension: 1,
            knowledge: HashMap::new(),
        }
    }

    pub fn add_knowledge(&mut self, facts: &[F], solution: S) -> &mut Self {
        let dim = solution.dimension();
        if dim > self.output_dimension {
            self.output_dimension = dim;
        }
        match self.knowledge.entry(solution) {
            Entry::Occupied(mut entry) => {
                for fact in facts {
                    let dimension = fact.dimension();
                    if dimension > self.input_dimension {
                        self.input_dimension = dimension;
                    }
                    entry.get_mut().insert(fact.clone());
                }
            }
            Entry::Vacant(entry) => {
                let fact_set: HashSet<F> =
                    facts.into_iter().fold(HashSet::new(), |mut set, fact| {
                        let dimension = fact.dimension();
                        if dimension > self.input_dimension {
                            self.input_dimension = dimension;
                        }
                        set.insert(fact.clone());
                        set
                    });
                entry.insert(fact_set);
            }
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use crate::rnn::common::signal::Signal;

    use super::*;

    #[derive(Debug, PartialEq, Eq, Hash, Clone)]
    struct Byte(Signal);

    #[derive(Debug, PartialEq, Eq, Hash)]
    struct DecimalDigits(Signal);

    #[derive(Debug, PartialEq, Eq, Hash)]
    enum DecimalSolutions {
        Zero,
        One,
        Two,
        Three,
        Fourth,
        Five,
        Six,
        Seven,
        Eight,
        Nine,
    }

    impl Splittable for Byte {
        fn dimension(&self) -> usize {
            let value = self.0;
            if value & 0b1000_0000 != 0 {
                8
            } else if value & 0b0100_0000 != 0 {
                7
            } else if value & 0b0010_0000 != 0 {
                6
            } else if value & 0b0001_0000 != 0 {
                5
            } else if value & 0b0000_1000 != 0 {
                4
            } else if value & 0b0000_0100 != 0 {
                3
            } else if value & 0b0000_0010 != 0 {
                2
            } else {
                1
            }
        }

        fn split(&self) -> Vec<u8> {
            let mut value = self.0;
            let mut parts: Vec<u8> = vec![];
            for _ in 0..8 {
                let bit = if value & 0x01 == 0x01 { 1 } else { 0 };
                parts.push(bit);
                value >>= 1;
            }

            parts
        }
    }

    impl Reconstructible<Signal> for DecimalDigits {
        fn reconstruct(raw_data: Vec<Signal>) -> Option<DecimalDigits> {
            if raw_data.len() > 0 {
                Some(DecimalDigits(raw_data[0]))
            } else {
                None
            }
        }

        fn dimension(&self) -> usize {
            1
        }
    }

    impl Reconstructible<Signal> for DecimalSolutions {
        fn reconstruct(raw_data: Vec<Signal>) -> Option<DecimalSolutions> {
            if raw_data.len() != 10 {
                return None;
            }

            let mut max_index: Option<usize> = None;
            let mut max_value: Option<Signal> = None;
            for (index, signal) in raw_data.into_iter().enumerate() {
                if let Some(max_signal) = max_value {
                    if max_signal < signal {
                        max_value = Some(signal);
                        max_index = Some(index);
                    } else if max_signal == signal {
                        return None;
                    }
                } else {
                    max_value = Some(signal);
                    max_index = Some(index);
                }
            }
            match max_index {
                Some(0) => Some(DecimalSolutions::Zero),
                Some(1) => Some(DecimalSolutions::One),
                Some(2) => Some(DecimalSolutions::Two),
                Some(3) => Some(DecimalSolutions::Three),
                Some(4) => Some(DecimalSolutions::Fourth),
                Some(5) => Some(DecimalSolutions::Five),
                Some(6) => Some(DecimalSolutions::Six),
                Some(7) => Some(DecimalSolutions::Seven),
                Some(8) => Some(DecimalSolutions::Eight),
                Some(9) => Some(DecimalSolutions::Nine),
                _ => None,
            }
        }

        fn dimension(&self) -> usize {
            10
        }
    }

    #[test]
    fn new_training_data_should_have_same_name_from_constructor_param() {
        let name = "tester";
        let data: LearningData<Byte, DecimalDigits> = LearningData::new(name);

        assert_eq!(data.name, name.to_string());
    }

    #[test]
    fn new_training_data_should_have_input_dimension_equal_one() {
        let data: LearningData<Byte, DecimalDigits> = LearningData::new("test");

        assert_eq!(data.input_dimension, 1);
    }

    #[test]
    fn new_training_data_should_have_output_dimension_equal_one() {
        let data: LearningData<Byte, DecimalDigits> = LearningData::new("test");

        assert_eq!(data.output_dimension, 1);
    }

    #[test]
    fn new_training_data_should_have_empty_() {
        let data: LearningData<Byte, DecimalDigits> = LearningData::new("test");

        assert_eq!(data.knowledge.len(), 0);
    }

    #[test]
    fn after_adding_knowledge_dimension_should_change_for_decimal_digits() {
        let mut data: LearningData<Byte, DecimalDigits> =
            LearningData::new("Binary2DecimalConverter");

        data.add_knowledge(&[Byte(0)], DecimalDigits(0));
        assert_eq!(data.input_dimension, 1);
        assert_eq!(data.output_dimension, 1);
        assert_eq!(data.knowledge.len(), 1);

        data.add_knowledge(&[Byte(1)], DecimalDigits(1));
        assert_eq!(data.input_dimension, 1);
        assert_eq!(data.output_dimension, 1);
        assert_eq!(data.knowledge.len(), 2);

        data.add_knowledge(&[Byte(0b0000_0010)], DecimalDigits(2));
        assert_eq!(data.input_dimension, 2);
        assert_eq!(data.output_dimension, 1);
        assert_eq!(data.knowledge.len(), 3);

        data.add_knowledge(&[Byte(0b1000_0000)], DecimalDigits(128));
        assert_eq!(data.input_dimension, 8);
        assert_eq!(data.output_dimension, 1);
        assert_eq!(data.knowledge.len(), 4);
    }

    #[test]
    fn after_adding_knowledge_dimension_should_change_for_decimal_solutions() {
        let mut data: LearningData<Byte, DecimalSolutions> =
            LearningData::new("Binary2DecimalSolutionConverter");

        data.add_knowledge(&[Byte(0)], DecimalSolutions::Zero)
            .add_knowledge(&[Byte(0b0000_0001)], DecimalSolutions::One)
            .add_knowledge(&[Byte(0b0000_0010)], DecimalSolutions::Two)
            .add_knowledge(&[Byte(0b0000_0011)], DecimalSolutions::Three);

        assert_eq!(data.input_dimension, 2);
        assert_eq!(data.output_dimension, 10);
        assert_eq!(data.knowledge.len(), 4);
    }
}
