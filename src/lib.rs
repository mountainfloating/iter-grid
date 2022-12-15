// #![no_std]
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
use core::{ops::{RangeBounds, Index, IndexMut, Range}, iter::{StepBy, Skip, Take }};

/// ToGrid ist implemented for all iterators.
/// Provides the grid function to wrap iterators with the Grid struct which contains the main functionality.
pub trait IntoGrid<I>
{
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
pub struct Grid<I>
{
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

impl<I> Grid<I>
{
    pub fn index_from_flat(&self, index:usize)->(usize,usize){
        assert!(self.columns!=0, "Columns set to 0! Cant calculate index");
        let c = index%self.columns;
        (c,(index-c)/self.columns)
    }
    pub fn index_to_flat(&self, col:usize, row:usize)->usize{
        self.columns*row+col
    }
}

impl<'a,I> Grid<I>
where
    I: IntoIterator + FromIterator<I::Item>,
    I::IntoIter: Clone ,
{
    pub fn transpose(self)->Grid<I>{
        let columns = self.columns;
        let iter = self.inner.into_iter();
        let new_columns = iter.clone().grid(columns).iter_col(0).count();
        Grid { columns: new_columns, inner: (0..columns).flat_map(|col|iter.clone().grid(columns).iter_col(col)).collect() }
    }
}
impl<I> Grid<I>
where
    I: IntoIterator,
{
    // pub fn count_rows(&self)->usize{
    //     self.inner.clone().count()/self.columns
    // }
    // pub fn iter_sub<R: RangeBounds<usize>>(
    //      self,
    //      col_bounds: R,
    //      row_bounds: R,
    //  ) ->Grid<impl IntoIterator<Item = I::Item>>{
    //      let columns = self.columns;
    //     let col_range = self.extract_range(&col_bounds, columns);
    //     Grid{
    //           columns: col_range.end-col_range.start,
    //           inner: self
    //           .iter_rows(row_bounds)
    //           .transpose()
    //           .iter_rows(col_bounds),
    //     }
    // }
    pub fn iter_col(self, col: usize) -> StepBy<Skip<I::IntoIter>>{
        assert!(col<self.columns);
        self.inner.into_iter().skip(col).step_by(self.columns)
    }

    // pub fn iter_cols<R: RangeBounds<usize>>(&self, bounds: R) ->  Grid<impl IntoIterator<Item = I::Item>>{
    //     let bounds = self.extract_range(&bounds, self.columns);
    //     assert!(bounds.end <= self.columns);
    //     let col_new = bounds.end-bounds.start;
    //     (0..)..unzip().map(|row_idx| self.inner.clone().skip(n))
    // }
    pub fn iter_row(self, row: usize) -> Take<Skip<I::IntoIter>>{
        self.inner
            .into_iter()
            .skip(row*self.columns)
            .take(self.columns)
    }
    // pub fn iter_rows<R: RangeBounds<usize>>(self, bounds: R) -> Grid<Take<Skip<I>>> {
    //     let bounds = self.extract_range(&bounds, usize::MAX);
    //     let inner=  self.inner
    //         .skip(bounds.start.saturating_mul(self.columns))
    //         .take((bounds.end - bounds.start).saturating_mul(self.columns));
    //     Grid{
    //         columns: self.columns,
    //         inner,
    //     }
    // }
    /// * * x
    /// * x *
    /// x * *
    /// 
    pub fn iter_diag_bwd(self,col:usize,row:usize) -> impl Iterator<Item = I::Item>{
        let mut iter = self.inner.into_iter().skip((row-(self.columns-col))*self.columns-1);
        (0..self.columns).rev().filter_map(move |col_off|iter.nth(col_off))
    }
    /// x * *
    /// * x *
    /// * * x
    /// 
    pub fn iter_diag_fwd(self,col:usize,row:usize) -> impl Iterator<Item = I::Item>{
        let skip = self.index_to_flat(row-col, 0);
        let mut iter = self.inner.into_iter().skip(skip);
        (0..self.columns).filter_map(move |col_off|iter.nth(col_off))
    }

    fn extract_range<R: RangeBounds<usize>>(
        &self,
        bounds: &R,
        max: usize,
    ) -> Range<usize> {
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


impl<I> Grid<I>
where
    I: Index<usize>,
    I::Output:Sized
{
    pub fn get(&self, col: usize, row: usize) -> &I::Output {
        assert!(col < self.columns);
        let index = self.index_to_flat(col, row);
        &self.inner[index]
    }
}
impl<I> Grid<I>
where
    I: IndexMut<usize>,
    I::Output:Sized
{
    pub fn get_mut(&mut self, col: usize, row: usize) -> &mut I::Output {
        assert!(col < self.columns);
        let index = self.index_to_flat(col, row);
        &mut self.inner[index]
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
        let mut store:Vec<usize> = file
              .lines()
                  .flat_map(|line| line.split(',').map(|s| s.parse::<usize>().unwrap())).collect();
        // store.iter_mut()
        //   .grid(5)
        //   .iter_col(3)
        //   .for_each(|i| *i = 0);

        //  store.iter_mut()
        //    .grid(5)
        //    .iter_row(2)
        //    .for_each(|i| *i = 0);
        let store = store.grid(5).transpose().inner;
        println!("{store:?}")

          // prints: [1, 2, 3, 0, 5, 6, 7, 8, 0, 10, 11, 12, 13, 0, 15]
      }
}
