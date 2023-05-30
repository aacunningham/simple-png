use std::{iter::StepBy, ops::Range};

use crate::utils::div_ceil;

pub(crate) struct Adam7Iter {
    current_pass: Option<usize>,
    width: usize,
    height: usize,
}
impl Adam7Iter {
    pub(crate) fn new(width: usize, height: usize) -> Self {
        Self {
            current_pass: Some(0),
            width,
            height,
        }
    }

    const STARTING_ROW: [usize; 7] = [0, 0, 4, 0, 2, 0, 1];
    const STARTING_COL: [usize; 7] = [0, 4, 0, 2, 0, 1, 0];
    const ROW_INCREMENT: [usize; 7] = [8, 8, 8, 4, 4, 2, 2];
    const COL_INCREMENT: [usize; 7] = [8, 8, 4, 4, 2, 2, 1];
}
impl Iterator for Adam7Iter {
    type Item = SubImage;
    fn next(&mut self) -> Option<Self::Item> {
        let mut pass = self.current_pass?;
        while pass < 7 {
            let pass_width = div_ceil(
                self.width.saturating_sub(Self::STARTING_COL[pass]),
                Self::COL_INCREMENT[pass],
            );
            let pass_height = div_ceil(
                self.height.saturating_sub(Self::STARTING_ROW[pass]),
                Self::ROW_INCREMENT[pass],
            );
            // If either is zero, the sub image has no pixels and we can go to the next image.
            if pass_width == 0 || pass_height == 0 {
                pass += 1;
                continue;
            }
            if pass == 6 {
                self.current_pass = None;
            } else {
                self.current_pass = Some(pass + 1);
            }
            return Some(SubImage {
                width: pass_width,
                height: pass_height,
                pixel_indices: PixelIndicesIter::new(
                    (Self::STARTING_ROW[pass]..self.height).step_by(Self::ROW_INCREMENT[pass]),
                    (Self::STARTING_COL[pass]..self.width).step_by(Self::COL_INCREMENT[pass]),
                    self.height,
                ),
            });
        }
        self.current_pass = None;
        None
    }
}

#[derive(Debug)]
pub(crate) struct SubImage {
    pub(crate) width: usize,
    pub(crate) height: usize,
    pub(crate) pixel_indices: PixelIndicesIter,
}

#[derive(Debug)]
pub(crate) struct PixelIndicesIter {
    rows: StepBy<Range<usize>>,
    current_row: Option<usize>,
    orig_columns: StepBy<Range<usize>>,
    columns: StepBy<Range<usize>>,
    height: usize,
}
impl PixelIndicesIter {
    pub fn new(
        mut rows: StepBy<Range<usize>>,
        columns: StepBy<Range<usize>>,
        height: usize,
    ) -> Self {
        let current_row = rows.next();
        Self {
            rows,
            current_row,
            orig_columns: columns.clone(),
            columns,
            height,
        }
    }
}
impl Iterator for PixelIndicesIter {
    type Item = usize;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(column) = self.columns.next() {
            return Some(self.current_row? * self.height + column);
        }
        self.current_row = Some(self.rows.next()?);
        self.columns = self.orig_columns.clone();
        Some(self.current_row? * self.height + self.columns.next()?)
    }
}

#[cfg(test)]
mod tests {
    use super::Adam7Iter;
    #[test]
    fn adam7iter_correctly_calculates_pass_dimensions() {
        let adam7 = Adam7Iter::new(8, 8);
        let expected_dimensions = [(1, 1), (1, 1), (2, 1), (2, 2), (4, 2), (4, 4), (8, 4)];
        for (pass, expected) in adam7.zip(expected_dimensions) {
            assert_eq!((pass.width, pass.height), expected);
        }

        let adam7 = Adam7Iter::new(9, 9);
        let expected_dimensions = [(2, 2), (1, 2), (3, 1), (2, 3), (5, 2), (4, 5), (9, 4)];
        for (pass, expected) in adam7.zip(expected_dimensions) {
            assert_eq!((pass.width, pass.height), expected);
        }

        let adam7 = Adam7Iter::new(16, 16);
        let expected_dimensions = [(2, 2), (2, 2), (4, 2), (4, 4), (8, 4), (8, 8), (16, 8)];
        for (pass, expected) in adam7.zip(expected_dimensions) {
            assert_eq!((pass.width, pass.height), expected);
        }

        let adam7 = Adam7Iter::new(4, 4);
        let expected_dimensions = [(1, 1), (1, 1), (2, 1), (2, 2), (4, 2)];
        for (pass, expected) in adam7.zip(expected_dimensions) {
            assert_eq!((pass.width, pass.height), expected);
        }
    }

    #[test]
    fn adam7iter_returns_iterator_over_pixel_indices() {
        let adam7 = Adam7Iter::new(8, 8);
        let expected_indices: [&[usize]; 7] = [
            &[0],
            &[4],
            &[32, 36],
            &[2, 6, 34, 38],
            &[16, 18, 20, 22, 48, 50, 52, 54],
            &[1, 3, 5, 7, 17, 19, 21, 23, 33, 35, 37, 39, 49, 51, 53, 55],
            &[
                8, 9, 10, 11, 12, 13, 14, 15, 24, 25, 26, 27, 28, 29, 30, 31, 40, 41, 42, 43, 44,
                45, 46, 47, 56, 57, 58, 59, 60, 61, 62, 63,
            ],
        ];
        for (pass, expected) in adam7.zip(expected_indices) {
            assert_eq!(pass.pixel_indices.collect::<Vec<_>>(), expected);
        }

        let adam7 = Adam7Iter::new(9, 9);
        let expected_lengths = [4, 2, 3, 6, 10, 20, 36];
        for (pass, expected) in adam7.zip(expected_lengths) {
            assert_eq!(pass.pixel_indices.count(), expected);
        }

        let adam7 = Adam7Iter::new(4, 4);
        let expected_lengths = [1, 1, 2, 4, 8];
        for (pass, expected) in adam7.zip(expected_lengths) {
            assert_eq!(pass.pixel_indices.count(), expected);
        }

        let adam7 = Adam7Iter::new(32, 32);
        let expected_lengths = [16, 16, 32, 64, 128, 256, 512];
        for (pass, expected) in adam7.zip(expected_lengths) {
            assert_eq!(pass.pixel_indices.count(), expected);
        }
    }
}
