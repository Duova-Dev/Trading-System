pub mod adx_indicator;
pub mod max_in_range;
pub mod min_in_range;

pub trait NextI {
    fn next(&mut self, input: f64) -> f64;
}
