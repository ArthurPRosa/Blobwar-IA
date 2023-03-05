//! rust alpha - beta implementation for the blobwar game.
#![deny(missing_docs)]
#![warn(clippy::all)]
#![feature(test)]

pub mod board;
pub mod configuration;
pub(crate) mod positions;
pub(crate) mod shmem;
pub mod strategy;

extern crate test;
