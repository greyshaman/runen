use std::hash::Hash;

use crate::rnn::layouts::neural_network::NeuralNetwork;

use super::{
    learning_data::LearningData, reconstructible::Reconstructible, splittable::Splittable,
};

pub trait Learn<F, S>
where
    S: Reconstructible + Eq + Hash,
    F: Splittable + Eq + Hash,
{
    fn learn(data: LearningData<F, S>) -> Vec<NeuralNetwork>;
}
