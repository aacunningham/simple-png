#![warn(missing_docs)]

//! # A PNG encoder/decoder
//! simple-png makes it easy to work with and modify PNG images.
//! Example usage:
//! ```
//! use simple_png::PNG;
//!
//! let png_data = std::fs::read("tests/png-suite/basn0g01.png")?;
//! let image = PNG::decode(&png_data)?;
//! std::fs::write("./new-image.png", image.encode())?;
//! # Ok::<(), anyhow::Error>(())
//! ```
mod chunks;
mod filters;
mod interlacing;
mod pixel;
mod png;
mod scanlines;
mod utils;

pub use pixel::Pixel;
pub use png::PNG;
