use std::any::Any;

use crate::rnn::common::{group_type::GroupType, grouped::Grouped};

pub trait NeuralElement: Grouped + Any {
  fn get_group_type(&self) -> GroupType {
    GroupType::Neural
  }

  fn as_any(&self) -> &dyn Any;

  fn as_mut_any(&mut self) -> &mut dyn Any;
}