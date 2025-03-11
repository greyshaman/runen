use std::hash::Hash;

use crate::rnn::neural::neural_network::NeuralNetwork;

use super::{
    default_reconstructible::DefaultReconstructible, default_splittable::DefaultSplittable,
    learning_data::LearningData, reconstructible::Reconstructible, splittable::Splittable,
    trainer::Trainer,
};

pub struct DefaultTrainer {
    learning_data: LearningData<DefaultSplittable, DefaultReconstructible>,
    network: NeuralNetwork,
}

// impl<F, S> Trainer<F, S> for DefaultTrainer
// where
//     F: Splittable + Eq + Hash,
//     S: Reconstructible + Eq + Hash,
// {
//     type Network = NeuralNetwork;

//     // fn new(
//     //     l_data: LearningData<F, S>,
//     //     net: Option<Self::Network>,
//     // ) -> Result<Self, Box<dyn std::error::Error>> {
//     //     let mut network = if let Some(n) = net {
//     //         n
//     //     } else {
//     //         NeuralNetwork::new()?
//     //     };

//     //     // TODO adjust network dimensions according the l_data

//     //     Ok(DefaultTrainer {
//     //         network,
//     //         learning_data: l_data,
//     //     })
//     // }

//     // fn next_generation(&mut self) -> Self::Network {
//     //     todo!()
//     // }

//     // fn add_knowledge(&mut self, l_data: LearningData<F, S>) {
//     //     todo!()
//     // }
// }
