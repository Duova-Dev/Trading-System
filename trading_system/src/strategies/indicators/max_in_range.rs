use std::f64::INFINITY;

use super::NextI;

// finds max in last far_n elements that arent in the last near_n elements
pub struct MaxmimumInRange {
    size: usize,
    vec: Vec<f64>,
    slow_index: usize,
    curr_index: usize,
    max_index: usize,
}

impl MaxmimumInRange {
    pub fn new(far_n: u64, near_n: u64) -> Self {
        // convert lengths to usize for indexing
        let far_n = far_n as usize;
        let near_n = near_n as usize;

        // far_n must be greater than near_n
        assert_eq!(far_n > near_n, true);

        MaxmimumInRange {
            size: far_n,
            vec: vec![-INFINITY; far_n],
            slow_index: (far_n - near_n) % far_n,
            curr_index: 0,
            max_index: 0,
        }
    }
    fn find_max_index(&self) -> usize {
        let mut max = -INFINITY;

        let mut i = (self.curr_index + 1) % self.size;
        let mut index = 0;
        loop {
            if self.vec[i] > max {
                max = self.vec[i];
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
        let mut mir = MaxmimumInRange::new(4u64, 2u64);
        assert_eq!(mir.next(1f64), -INFINITY);
        assert_eq!(mir.next(2f64), -INFINITY);
        assert_eq!(mir.next(3f64), 1f64);
        assert_eq!(mir.next(4f64), 2f64);
        assert_eq!(mir.next(2f64), 3f64);
        assert_eq!(mir.next(3f64), 4f64);
        assert_eq!(mir.next(1f64), 4f64);
        assert_eq!(mir.next(-1f64), 3f64);
        assert_eq!(mir.next(10f64), 3f64);
        assert_eq!(mir.next(5f64), 1f64);
        assert_eq!(mir.next(4f64), 10f64);
        assert_eq!(mir.next(-7f64), 10f64);

        let mut mir = MaxmimumInRange::new(3, 0);
        dbg!(mir.next(1f64));
        dbg!(mir.next(2f64));
        dbg!(mir.next(7f64));
        dbg!(mir.next(4f64));
        dbg!(mir.next(-10f64));
        dbg!(mir.next(3f64));
        dbg!(mir.next(2f64));
    }
}

impl NextI for MaxmimumInRange {
    fn next(&mut self, input: f64) -> f64 {
        // increment curr_index and slow_index
        self.curr_index = (self.curr_index + 1) % self.size;
        self.slow_index = (self.slow_index + 1) % self.size;

        // insert input
        self.vec[self.curr_index] = input;

        if self.curr_index == self.max_index {
            // if the max was replaced, find new max
            self.max_index = self.find_max_index();
        } else if self.vec[self.slow_index] > self.vec[self.max_index] {
            // if new slow_index is max, set max
            self.max_index = self.slow_index;
        }

        if self.vec[self.max_index] == -INFINITY{
            return 0f64;
        }
        return self.vec[self.max_index];
    }
}
