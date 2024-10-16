use std::error::Error;

use regex::Regex;

use super::{rnn_error::RnnError, spec_type::SpecificationType};

pub fn gen_id(container_id: &str, entity_id: usize, my_spec_type: SpecificationType) -> String {
  let specification_prefix =
    match my_spec_type {
      SpecificationType::Acceptor => 'A',
      SpecificationType::Collector => 'C',
      SpecificationType::Aggregator => 'G',
      SpecificationType::Emitter => 'E',
      SpecificationType::Media => 'M',
      SpecificationType::Composer => 'Z',
      _ => 'N',
    };

  let mut id = String::from(container_id);
  id.push(specification_prefix);
  id.push_str(entity_id.to_string().as_str());
  id
}

pub fn get_component_id_fraction(id: &str, spec_type: &SpecificationType)
  -> Result<usize, Box<dyn Error>> {
  if id.len() == 0 { return Err(Box::new(RnnError::NotPresent(String::from("Empty string present")))) }

  let r_pattern: &str = match spec_type {
    SpecificationType::Acceptor => r"Z\d+A(\d+)",
    SpecificationType::Collector => r"Z\d+C(\d+)",
    SpecificationType::Aggregator => r"Z\d+G(\d+)",
    SpecificationType::Emitter => r"Z\d+E(\d+)",
    SpecificationType::Composer => r"Z(\d+)",
    _ => return Err(Box::new(RnnError::NotSupportedArgValue)),
  };

  let rex = Regex::new(r_pattern)?;
  let captures = rex.captures(id).unwrap();
  if &captures.len() < &2 { return Err(Box::new(RnnError::PatternNotFound)); }
  let result = captures[1].parse::<usize>()?;
  Ok(result)
}
