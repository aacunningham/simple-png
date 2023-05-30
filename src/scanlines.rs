use std::ops::RangeFrom;

use crate::{
    chunks::ihdr::IHDRChunk,
    interlacing::{Adam7Iter, PixelIndicesIter},
    utils::div_ceil,
};

pub(crate) trait ScanlineIterator<'a>: Iterator<Item = (&'a [u8], Vec<usize>)> {}
impl<'a> ScanlineIterator<'a> for NormalScanline<'a> {}
impl<'a> ScanlineIterator<'a> for Adam7ScanlineIter<'a> {}

pub(crate) struct NormalScanline<'a> {
    scanline_iter: std::slice::Chunks<'a, u8>,
    counter: RangeFrom<usize>,
    width: usize,
}
impl<'a> NormalScanline<'a> {
    pub(crate) fn new(image_data: &'a [u8], header: &'a IHDRChunk) -> Self {
        Self {
            scanline_iter: image_data
                .chunks(calculate_scanline_width(header.width, header.pixel_width())),
            counter: 0..,
            width: header.width as usize,
        }
    }
}
impl<'a> Iterator for NormalScanline<'a> {
    type Item = (&'a [u8], Vec<usize>);
    fn next(&mut self) -> Option<Self::Item> {
        let next_scanline = self.scanline_iter.next()?;
        let mut v = Vec::with_capacity(self.width);
        for _ in 0..self.width {
            v.push(self.counter.next()?)
        }
        Some((next_scanline, v))
    }
}

pub(crate) struct Adam7ScanlineIter<'a> {
    image_data: &'a [u8],
    header: &'a IHDRChunk,
    inner_iter: Option<Adam7Iter>,
    scanline_iter: Option<
        std::iter::Zip<std::iter::Take<std::slice::Chunks<'a, u8>>, ChunkIter<PixelIndicesIter>>,
    >,
}
impl<'a> Adam7ScanlineIter<'a> {
    pub(crate) fn new(image_data: &'a [u8], header: &'a IHDRChunk) -> Self {
        let inner_iter = Some(Adam7Iter::new(
            header.width as usize,
            header.height as usize,
        ));
        Self {
            image_data,
            header,
            inner_iter,
            scanline_iter: None,
        }
    }
}
impl<'a> Iterator for Adam7ScanlineIter<'a> {
    type Item = (&'a [u8], Vec<usize>);
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(scanline) = self.scanline_iter.as_mut().and_then(Iterator::next) {
            return Some(scanline);
        }
        while let Some(sub_image) = self.inner_iter.as_mut().and_then(|in_iter| in_iter.next()) {
            let scanline_length =
                calculate_scanline_width(sub_image.width as u32, self.header.pixel_width());
            let (sub_image_data, rest): (&'a [u8], &'a [u8]) =
                self.image_data.split_at(scanline_length * sub_image.height);
            let height = sub_image.height;
            let pixel_indices = sub_image.pixel_indices.vec_chunks(sub_image.width);
            self.scanline_iter = Some(
                sub_image_data
                    .chunks(scanline_length)
                    .take(height)
                    .zip(pixel_indices),
            );
            self.image_data = rest;
            return self.scanline_iter.as_mut().and_then(Iterator::next);
        }
        None
    }
}

struct ChunkIter<I> {
    inner: I,
    size: usize,
}
impl<V, I> Iterator for ChunkIter<I>
where
    I: Iterator<Item = V>,
{
    type Item = Vec<I::Item>;
    fn next(&mut self) -> Option<Self::Item> {
        let mut results = Vec::with_capacity(self.size);
        for _ in 0..self.size {
            results.push(self.inner.next()?);
        }
        Some(results)
    }
}
trait IteratorExt<S> {
    fn vec_chunks(self, n: usize) -> ChunkIter<S>;
}
impl<T> IteratorExt<T> for T {
    fn vec_chunks(self, size: usize) -> ChunkIter<T> {
        ChunkIter { inner: self, size }
    }
}

const fn calculate_scanline_width(image_width: u32, pixel_width: u8) -> usize {
    div_ceil(image_width as usize * pixel_width as usize, 8) + 1
}
