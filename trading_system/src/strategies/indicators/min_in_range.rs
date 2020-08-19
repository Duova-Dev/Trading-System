use std::f64::INFINITY;

use super::NextI;

// finds max in last far_n elements that arent in the last near_n elements
pub struct MinimumInRange {
    size: usize,
    vec: Vec<f64>,
    slow_index: usize,
    curr_index: usize,
    min_index: usize,
}

impl MinimumInRange {
    pub fn new(far_n: u64, near_n: u64) -> Self {
        // convert lengths to usize for indexing
        let far_n = far_n as usize;
        let near_n = near_n as usize;

        // far_n must be greater than near_n
        assert_eq!(far_n > near_n, true);

        MinimumInRange {
            size: far_n,
            vec: vec![INFINITY; far_n],
            slow_index: (far_n - near_n) % far_n,
            curr_index: 0,
            min_index: 0,
        }
    }
    fn find_min_index(&self) -> usize {
        let mut min = INFINITY;

        let mut i = (self.curr_index + 1) % self.size;
        let mut index = 0;
        loop {
            if self.vec[i] < min {
                min = self.vec[i];
                index = i;
            }

            i = (i + 1) % self.size;
            if i == (self.slow_index + 1) % self.size {
                break;
            }
        }

        return index;
    }

    pub fn _test() {
        let mut mir = MinimumInRange::new(4u64, 2u64);
        assert_eq!(mir.next(1f64), INFINITY);
        assert_eq!(mir.next(2f64), INFINITY);
        assert_eq!(mir.next(3f64), 1f64);
        assert_eq!(mir.next(4f64), 1f64);
        assert_eq!(mir.next(2f64), 2f64);
        assert_eq!(mir.next(3f64), 3f64);
        assert_eq!(mir.next(1f64), 2f64);
        assert_eq!(mir.next(-1f64), 2f64);
        assert_eq!(mir.next(10f64), 1f64);
        assert_eq!(mir.next(5f64), -1f64);
        assert_eq!(mir.next(4f64), -1f64);
        assert_eq!(mir.next(-7f64), 5f64);

        let mut mir = MinimumInRange::new(3, 0);
        dbg!(mir.next(1f64));
        dbg!(mir.next(2f64));
        dbg!(mir.next(7f64));
        dbg!(mir.next(4f64));
        dbg!(mir.next(-10f64));
        dbg!(mir.next(3f64));
        dbg!(mir.next(2f64));
    }
}

impl NextI for MinimumInRange {
    fn next(&mut self, input: f64) -> f64 {
        // increment curr_index and slow_index
        self.curr_index = (self.curr_index + 1) % self.size;
        self.slow_index = (self.slow_index + 1) % self.size;

        // insert input
        self.vec[self.curr_index] = input;

        if self.curr_index == self.min_index {
            // if the min was replaced, find new min
            self.min_index = self.find_min_index();
        } else if self.vec[self.slow_index] < self.vec[self.min_index] {
            // if new slow_index is min, set min
            self.min_index = self.slow_index;
        }

        if self.vec[self.min_index] == INFINITY{
            return 0f64;
        }
        return self.vec[self.min_index];
    }
}
