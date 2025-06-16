#![doc = include_str!("../README.md")]
#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]

mod filter;
mod hash;
mod utils;

pub use crate::filter::RangeFilter;
pub use crate::hash::{PairwiseIndependentHasher, ParamError};
