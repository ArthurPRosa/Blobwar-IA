//! rust alpha - beta implementation for the blobwar game.
#![deny(missing_docs)]
#![warn(clippy::all)]
use std::time::Duration;

pub mod board;
pub mod configuration;
pub(crate) mod positions;
pub(crate) mod shmem;
pub mod strategy;

#[cfg(tests)]
mod tests {
    use super::*;

    #[test]
    fn time_greedy() {
        assert!(true, true);
    }

    #[test]
    fn time_minmax() {
        assert!(true, true);
    }

    #[test]
    fn time_alphabeta() {
        assert!(true, true);
    }
}


