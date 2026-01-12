#![feature(iter_map_windows)]

use std::ops::RangeInclusive;

pub mod bd_path;
pub mod bin_file;
pub mod binary;
pub mod cache;
pub mod cached_bin_file;
pub mod dyn_binary;
pub mod foreign;
pub mod index_file;
pub mod prelude;
pub mod table;

const fn add_size(a: usize, b: isize) -> usize {
    if b < 0 {
        a - b.unsigned_abs()
    } else {
        a + b as usize
    }
}

trait A: Sized {
    fn len(&self) -> Option<usize>;
    fn overlap(&self, other: &Self) -> Option<Self>;
}
impl A for RangeInclusive<usize> {
    fn len(&self) -> Option<usize> {
        assert!(*self.end() >= *self.start());
        let t = *self.end() - *self.start();
        if t == usize::MAX { None } else { Some(t + 1) }
    }

    fn overlap(&self, other: &Self) -> Option<Self> {
        let start = (*self.start()).max(*other.start());
        let end = (*self.end()).min(*other.end());
        if start <= end {
            Some(start..=end)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test_big_table;
#[cfg(test)]
mod test_bin_file;
#[cfg(test)]
mod test_cache;
#[cfg(test)]
mod test_index;
#[cfg(test)]
mod test_table;
