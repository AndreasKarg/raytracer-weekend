use std::ops::Range;

pub trait TypedRng {
    fn random_f32(&mut self) -> f32;
    fn random_range_f32(&mut self, range: Range<f32>) -> f32;
    fn random_range_usize(&mut self, range: Range<usize>) -> usize;
}
