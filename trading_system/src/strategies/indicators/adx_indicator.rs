use ta::indicators::ExponentialMovingAverage;
use ta::Next;

use super::max_in_range::MaxmimumInRange;
use super::min_in_range::MinimumInRange;
use super::NextI;

pub struct AverageDirectionalMovementIndex {
    max_in_prev_range: MaxmimumInRange,
    min_in_prev_range: MinimumInRange,
    max_in_curr_range: MaxmimumInRange,
    min_in_curr_range: MinimumInRange,
    pdi_ema: ExponentialMovingAverage,
    ndi_ema: ExponentialMovingAverage,
    adx_ema: ExponentialMovingAverage,
}

impl AverageDirectionalMovementIndex {
    pub fn new(period_length: u64, num_periods: u64) -> Self {
        AverageDirectionalMovementIndex {
            max_in_prev_range: MaxmimumInRange::new(2 * period_length, period_length),
            min_in_prev_range: MinimumInRange::new(2 * period_length, period_length),
            max_in_curr_range: MaxmimumInRange::new(period_length, 0),
            min_in_curr_range: MinimumInRange::new(period_length, 0),
            pdi_ema: ExponentialMovingAverage::new((num_periods * period_length) as u32).unwrap(),
            ndi_ema: ExponentialMovingAverage::new((num_periods * period_length) as u32).unwrap(),
            adx_ema: ExponentialMovingAverage::new((num_periods * period_length) as u32).unwrap(),
        }
    }
}

impl NextI for AverageDirectionalMovementIndex {
    fn next(&mut self, input: f64) -> f64 {
        let prev_high = self.max_in_prev_range.next(input);
        let prev_low = self.min_in_prev_range.next(input);

        let curr_high = self.max_in_curr_range.next(input);
        let curr_low = self.min_in_curr_range.next(input);

        let up_move = curr_high - prev_high;
        let down_move = prev_low - curr_low;

        let pdm = if up_move > down_move && up_move > 0f64 {
            up_move
        } else {
            0f64
        };
        let ndm = if down_move > up_move && down_move > 0f64 {
            down_move
        } else {
            0f64
        };

        let pdi = 100f64 * self.pdi_ema.next(pdm);
        let ndi = 100f64 * self.ndi_ema.next(ndm);

        let adx = ((pdi - ndi) / (pdi + ndi)).abs();

        let curr_adx = 100f64 * self.adx_ema.next(adx);

        return curr_adx;
    }
}
