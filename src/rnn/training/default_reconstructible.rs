use super::reconstructible::Reconstructible;

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct DefaultReconstructible(u8);

impl Reconstructible<u8> for DefaultReconstructible {
    fn reconstruct(&self, raw_data: Vec<u8>) -> Option<Self> {
        if raw_data.len() == 1 {
            Some(DefaultReconstructible(raw_data[0]))
        } else {
            None
        }
    }

    fn dimension(&self) -> usize {
        1
    }
}
