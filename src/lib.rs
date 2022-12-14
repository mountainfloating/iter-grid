#![no_std]
//! Grids for Iterators
//!
//! Provides a two dimensional abstraction over Iterators.
//! Intended to be simple, flexible and ideomatic.
//! ```rust
//! use iter_grid::IntoGrid;
//!
//! let file:&str = "1,2,3,4,5\n6,7,8,9,10\n11,12,13,14,15";
//! let mut store = file.lines()
//!     .flat_map(|line|line.split(',').map(|s|s.parse().unwrap()))
//!     .collect::<Vec<_>>();
//! store.iter_mut().grid(5).iter_col(3).for_each(|i| *i= 0);
//! store.iter_mut().grid(5).iter_row(1).for_each(|i| *i+= 1);
//! let borrowing_grid = store.iter().grid(5);
//! drop(borrowing_grid);
//! let capturing_grid = store.grid(5);
//! println!("{:?}", capturing_grid);
//! ```
use core::{
    iter::{Skip, StepBy, Take},
    ops::{Index, IndexMut, Range, RangeBounds},
};

/// ToGrid ist implemented for all iterators.
/// Provides the grid function to wrap iterators with the Grid struct which contains the main functionality.
pub trait IntoGrid<I> {
    fn grid(self, columns: usize) -> Grid<I>;
}

impl<I> IntoGrid<I> for I
where
    I: IntoIterator,
{
    fn grid(self, columns: usize) -> Grid<I> {
        Grid {
            columns,
            inner: self,
        }
    }
}

///The Grid struct wraps an Iterator and provies two dimensional access over its contents.
#[derive(Debug, Clone)]
pub struct Grid<I> {
    pub columns: usize,
    inner: I,
}

impl<I> IntoIterator for Grid<I>
where
    I: IntoIterator,
{
    type Item = I::Item;

    type IntoIter = I::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

impl<I> Grid<I> {
    pub fn index_from_flat(&self, index: usize) -> (usize, usize) {
        assert!(self.columns != 0, "Columns set to 0! Cant calculate index");
        let c = index % self.columns;
        (c, (index - c) / self.columns)
    }
    pub fn index_to_flat(&self, col: usize, row: usize) -> usize {
        self.columns * row + col
    }
}

impl<'a, I> Grid<I>
where
    I: IntoIterator + FromIterator<I::Item>,
    I::IntoIter: Clone,
{
    /// 1 2 3    1 4
    /// 4 5 6 => 2 5
    ///          3 6
    ///
    pub fn into_transpose(self) -> Grid<I> {
        let columns = self.columns;
        let iter = self.inner.into_iter();
        let len = iter.clone().count();
        assert!(columns % len == 0);
        Grid {
            columns: len / columns,
            inner: (0..columns)
                .flat_map(|col| iter.clone().grid(columns).iter_col(col))
                .collect(),
        }
    }
}
impl<'a, I> Grid<I>
where
    I: IntoIterator,
    I::IntoIter: Clone + 'a,
{
    /// 1 2 3    1 4
    /// 4 5 6 => 2 5
    ///          3 6
    ///
    pub fn iter_transpose(self) -> impl Iterator<Item = I::Item> + 'a {
        let iter = self.inner.into_iter();
        (0..self.columns).flat_map(move |col| iter.clone().grid(self.columns).iter_col(col))
    }
}
impl<I> Grid<I>
where
    I: IntoIterator,
{
    pub fn get(self, col: usize, row: usize) -> Option<I::Item> {
        let skip = self.index_to_flat(col, row);
        self.into_iter().skip(skip).next()
    }

    // pub fn count_rows(&self)->usize{
    //     self.inner.clone().count()/self.columns
    // }
    pub fn iter_sub<R1, R2>(
        self,
        col_bounds: R1,
        row_bounds: R2,
    ) -> Grid<impl IntoIterator<Item = I::Item>>
    // Grid<FilterMap<Enumerate<Take<Skip<I::IntoIter>>>, impl FnMut((usize, I::Item)) -> Option<I::Item>>>
    where
        R1: RangeBounds<usize>,
        R2: RangeBounds<usize>,
    {
        self.iter_rows(row_bounds).iter_cols(col_bounds)
    }

    ///```rust
    ///
    /// // . x .
    /// // . x .
    /// // . x .
    ///
    /// use iter_grid::IntoGrid;
    /// (0..25).grid(5)
    ///     .iter_col(3)
    ///     .zip([3,8,13,18,23])
    ///     .for_each(|(l, r)| assert!(l == r));
    ///```   
    pub fn iter_col(self, col: usize) -> StepBy<Skip<I::IntoIter>> {
        assert!(col < self.columns);
        self.inner.into_iter().skip(col).step_by(self.columns)
    }

    ///```rust
    ///
    /// // . . .
    /// // x x x
    /// // . . .
    ///
    /// use iter_grid::IntoGrid;
    /// (0..25).grid(5)
    ///     .iter_row(3)
    ///     .zip(15..20)
    ///     .for_each(|(l, r)| assert!(l == r));
    ///```
    pub fn iter_row(self, row: usize) -> Take<Skip<I::IntoIter>> {
        self.inner
            .into_iter()
            .skip(row * self.columns)
            .take(self.columns)
    }

    ///```rust
    ///
    /// // . . x
    /// // . x .
    /// // x . .
    ///
    /// use iter_grid::IntoGrid;
    /// (0..25).grid(5)
    ///     .iter_diag_bwd(0,1)
    ///     .zip([1,5])
    ///     .for_each(|(l, r)| assert!(l == r));
    /// (0..25).grid(5)
    ///     .iter_diag_bwd(3,2)
    ///     .zip([9,13,17,21])
    ///     .for_each(|(l, r)| assert!(l == r));
    ///```
    pub fn iter_diag_bwd(self, col: usize, row: usize) -> StepBy<Skip<I::IntoIter>> {
        let skip = if col > row {
            // lower part
            self.index_to_flat(self.columns - 1, row - (self.columns - 1 - col))
        } else {
            // upper part
            self.index_to_flat(row - col, 0)
        };
        self.inner.into_iter().skip(skip).step_by(self.columns - 1)
    }
    ///```rust
    ///
    /// // x . .
    /// // . x .
    /// // . . x
    ///
    /// use iter_grid::IntoGrid;
    /// (0..25).grid(5)
    ///     .iter_diag_fwd(1,2)
    ///     .zip([5,11,17,23])
    ///     .for_each(|(l, r)| assert!(l == r));
    /// (0..25).grid(5)
    ///     .iter_diag_fwd(4,2)
    ///     .zip([2,8,14])
    ///     .for_each(|(l, r)| assert!(l == r));
    ///```
    pub fn iter_diag_fwd(self, col: usize, row: usize) -> StepBy<Skip<I::IntoIter>> {
        let mut diff = col.abs_diff(row);
        if col < row {
            diff = self.index_to_flat(0, diff)
        } else {
            diff = self.index_to_flat(diff, 0);
        }
        self.inner.into_iter().skip(diff).step_by(self.columns + 1)
    }

    ///```rust
    ///
    /// // * 1 2 *
    /// // * 3 4 *
    /// // * 5 6 *
    /// // * 7 8 *
    ///
    /// use iter_grid::IntoGrid;
    /// (0..25).grid(5)
    ///     .iter_cols(1..=2)
    ///     .into_iter()
    ///     .zip([1,2,6,7,11,12,16,17,21,22])
    ///     .for_each(|(l, r)| assert!(l == r));
    ///```
    pub fn iter_cols<R>(self, bounds: R) -> Grid<impl Iterator<Item = I::Item>>
    where
        // FilterMap<Enumerate<I::IntoIter>,impl FnMut((usize,I::Item))->Option<I::Item>> where
        R: RangeBounds<usize>,
    {
        let bounds = self.extract_range(&bounds, self.columns);
        assert!(bounds.end <= self.columns);
        let new_columns = bounds.end - bounds.start;
        self.inner
            .into_iter()
            .enumerate()
            .filter_map(move |(p, i)| {
                if bounds.contains(&(p % self.columns)) {
                    Some(i)
                } else {
                    None
                }
            })
            .grid(new_columns)
    }

    ///```rust
    /// // * * * *
    /// // 1 2 3 4
    /// // 5 6 7 8
    /// // * * * *
    ///
    /// use iter_grid::IntoGrid;
    /// (0..25).grid(5)
    ///     .iter_rows(1..=2)
    ///     .into_iter()
    ///     .zip((5..15))
    ///     .for_each(|(l, r)| assert!(l == r));
    ///```
    pub fn iter_rows<R: RangeBounds<usize>>(self, bounds: R) -> Grid<Take<Skip<I::IntoIter>>> {
        let bounds = self.extract_range(&bounds, usize::MAX);
        self.inner
            .into_iter()
            .skip(bounds.start * self.columns)
            .take((bounds.end - bounds.start).saturating_mul(self.columns))
            .grid(self.columns)
    }
    fn extract_range<R: RangeBounds<usize>>(&self, bounds: &R, max: usize) -> Range<usize> {
        let start = match bounds.start_bound() {
            core::ops::Bound::Included(p) => *p,
            core::ops::Bound::Excluded(p) => p + 1,
            core::ops::Bound::Unbounded => 0,
        };

        let end = match bounds.end_bound() {
            core::ops::Bound::Included(p) => p + 1,
            core::ops::Bound::Excluded(p) => *p,
            core::ops::Bound::Unbounded => max,
        };
        assert!(end <= max);
        start..end
    }
}
#[cfg(test)]
mod tests {
    extern crate alloc;
    use alloc::vec::Vec;

    use super::*;

    #[test]
    fn test_get() {
        let file: &str = "1,2,3,4,5\n6,7,8,9,10\n11,12,13,14,15";
        let store: Vec<usize> = file
            .lines()
            .flat_map(|line| line.split(',').map(|s| s.parse::<usize>().unwrap()))
            .collect();

        (0..25)
            .grid(5)
            .iter_sub(0..1, 0..1)
            .into_iter()
            .zip([0])
            .for_each(|(l, r)| assert!(l == r));
        // store.iter_mut()
        //   .grid(5)
        //   .iter_col(3)
        //   .for_each(|i| *i = 0);

        //  store.iter_mut()
        //    .grid(5)
        //    .iter_row(2)
        //    .for_each(|i| *i = 0);
        let _store = store.grid(5).iter_transpose();
        // println!("{store:?}")
        // prints: [1, 2, 3, 0, 5, 6, 7, 8, 0, 10, 11, 12, 13, 0, 15]
    }
}
