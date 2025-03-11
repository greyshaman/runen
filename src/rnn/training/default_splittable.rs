use super::splittable::Splittable;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct DefaultSplittable(u8);

impl Splittable for DefaultSplittable {
    fn split(&self) -> Vec<u8> {
        vec![self.0]
    }

    fn dimension(&self) -> usize {
        1
    }
}
