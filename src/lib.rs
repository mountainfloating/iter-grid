#![no_std]

//! Grid indexing implemented for Iterators
//!
//! Provides an ideomatic abstraction for two dimensional Indexes.
//! Intended to be simple and flexible.
//! ```rust
//! use iter_grid::ToGrid;
//!
//! let file:&str = "1,2,3,4,5\n6,7,8,9,10\n11,12,13,14,15";
//! let mut store = file.lines()
//!     .flat_map(|line|line.split(',').map(|s|s.parse().unwrap()))
//!     .collect::<Vec<_>>();
//! let grid = store.iter_mut().grid(5);
//! grid.iter_col(3).for_each(|i| *i= 0);
//! println!("{:?}", store);
//! // prints: [1, 2, 3, 0, 5, 6, 7, 8, 0, 10, 11, 12, 13, 0, 15]
//! ```
use core::{ops::RangeBounds, iter::{StepBy, Skip, Take}};
/// ToGrid ist implemented for all iterators.
/// Provides the grid function to wrap iterators with the Grid struct which contains the main functionality.
pub trait ToGrid
where
    Self: Iterator + Sized,
{
    fn grid(self, columns: usize) -> Grid<Self>;
}

impl<I> ToGrid for I
where
    I: Iterator,
{
    fn grid(self, columns: usize) -> Grid<Self> {
        Grid {
            columns,
            inner: self,
        }
    }
}
///The Grid struct wraps an Iterator and provies two dimensional access over its contents.
#[derive(Debug, Clone)]
pub struct Grid<I>
where
    I: Iterator,
{
    pub columns: usize,
    inner: I,
}

impl<I> Iterator for Grid<I>
where
    I: Iterator,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

impl<I> Grid<I>
where
    I: Iterator + Clone,
{
    pub fn transpose(self) -> impl Iterator<Item = I::Item> {
        (0..self.columns).flat_map(move|col| 
            self.clone().iter_col(col))
    }
}

impl<I> Grid<I>
where
    I: Iterator,
{
    pub fn index_from_flat(&self, index:usize)->(usize,usize){
        assert!(self.columns!=0, "Columns set to 0! Cant calculate index");
        let c = index%self.columns;
        (c,(index-c)/self.columns)
    }
    pub fn index_to_flat(&self, col:usize, row:usize)->usize{
        self.columns*row+col
    }
    pub fn get(self, col: usize, row: usize) -> Option<I::Item> {
        assert!(col < self.columns);
        let index = self.index_to_flat(col, row);
        self.inner.skip(index).next()
    }
    pub fn iter_sub<R: RangeBounds<usize>>(
        self,
        col_bounds: R,
        row_bounds: R,
    ) -> impl Iterator<Item = I::Item> {
        let columns = self.columns;
        self.iter_rows(row_bounds)
            .grid(columns)
            .iter_cols(col_bounds)
    }
    pub fn iter_col(self, col: usize) -> StepBy<Skip<I>>{
        let step = self.columns;
        self.inner.skip(col).step_by(step)
    }
    pub fn iter_cols<R: RangeBounds<usize>>(self, bounds: R) -> impl Iterator<Item = I::Item> {
        let bounds = self.extract_range(bounds, self.columns);
        assert!(bounds.end <= self.columns);
        self.inner.enumerate().filter_map(move |(pos, item)| {
            if bounds.contains(&(pos % self.columns)) {
                Some(item)
            } else {
                None
            }
        })
    }
    pub fn iter_row(self, row: usize) -> Take<Skip<I>>{
        self.inner
            .skip(row.saturating_mul(self.columns))
            .take(self.columns)
    }
    pub fn iter_rows<R: RangeBounds<usize>>(self, bounds: R) -> impl Iterator<Item = I::Item> {
        let bounds = self.extract_range(bounds, usize::MAX);
        self.inner
            .skip(bounds.start.saturating_mul(self.columns))
            .take((bounds.end - bounds.start).saturating_mul(self.columns))
    }

    /// * * x
    /// * x *
    /// x * *
    /// 
    pub fn iter_diag_bwd(self,col:usize,row:usize) -> impl Iterator<Item = I::Item>{
        let mut iter = self.inner.skip((row-(self.columns-col))*self.columns-1);
        (0..self.columns).rev().filter_map(move |col_off|iter.nth(col_off))
    }

    /// x * *
    /// * x *
    /// * * x
    /// 
    pub fn iter_diag_fwd(self,col:usize,row:usize) -> impl Iterator<Item = I::Item>{
        let skip = self.index_to_flat(row-col, 0);
        let mut iter = self.inner.skip(skip);
        (0..self.columns).filter_map(move |col_off|iter.nth(col_off))
    }

    fn extract_range<R: RangeBounds<usize>>(
        &self,
        bounds: R,
        max: usize,
    ) -> core::ops::Range<usize> {
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
    // use super::*;

    // #[test]
    // fn test_get() {
    //     let file: &str = "1,2,3,4,5\n6,7,8,9,10\n11,12,13,14,15";
    //     let mut store = file
    //         .lines()
    //         .flat_map(|line| line.split(',').map(|s| s.parse().unwrap()))
    //         .collect::<Vec<_>>();
    //     let grid = store.iter_mut().grid(5);
    //     grid.iter_col(3).for_each(|i| *i = 0);
    //     println!("{:?}", store);
    //     // prints: [1, 2, 3, 0, 5, 6, 7, 8, 0, 10, 11, 12, 13, 0, 15]
    // }
}
