use super::group_type::GroupType;

/// Grouping entity.
pub trait Grouped {
    fn get_group_type(&self) -> GroupType;
}
