use std::error::Error;

use regex::Regex;

use super::{group_type::GroupType, rnn_error::RnnError, spec_type::SpecificationType};

/// Function generate id in format "Z123A2" based on container Id and provided available Id
pub fn gen_id_by_spec_type(container_id: &str, entity_num_id: usize, my_spec_type: &SpecificationType) -> String {
  let specification_prefix =
    match my_spec_type {
      SpecificationType::Acceptor => 'A',
      SpecificationType::Collector => 'C',
      SpecificationType::Aggregator => 'G',
      SpecificationType::Emitter => 'E',
      SpecificationType::Media => 'M',
      SpecificationType::Container => 'Z',
      _ => 'U',
    };

  format!("{}{}{}", container_id, specification_prefix, entity_num_id)
}

pub fn gen_id_by_group_type(parent_id: &str, entry_num_id: usize, group_type: &GroupType) -> String {
  let group_prefix = match group_type {
    GroupType::Neural => 'Z',
    GroupType::Cyber => 'Y',
  };

  format!("{}{}{}", parent_id, group_prefix, entry_num_id)
}

/// Extract number fraction of component Id
pub fn get_component_id_fraction(id: &str, spec_type: &SpecificationType)
  -> Result<usize, Box<dyn Error>> {
  if id.len() == 0 { return Err(Box::new(RnnError::NotPresent(String::from("Empty string present")))) }

  let r_pattern: &str = match spec_type {
    SpecificationType::Acceptor => r"Z\d+A(\d+)",
    SpecificationType::Collector => r"Z\d+C(\d+)",
    SpecificationType::Aggregator => r"Z\d+G(\d+)",
    SpecificationType::Emitter => r"Z\d+E(\d+)",
    SpecificationType::Container => r"Z(\d+)",
    _ => return Err(Box::new(RnnError::NotSupportedArgValue)),
  };

  let rex = Regex::new(r_pattern)?;
  let captures = rex.captures(id).unwrap();
  if &captures.len() < &2 { return Err(Box::new(RnnError::PatternNotFound)); }
  let result = captures[1].parse::<usize>()?;
  Ok(result)
}
