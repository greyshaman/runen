use std::any::{Any, TypeId};

use super::identity::Identity;

/// The Component abstract entity
pub trait Component: Identity {}