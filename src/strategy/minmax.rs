//! Implementation of the min max algorithm.
use rayon::prelude::{ParallelBridge, ParallelIterator};

use super::Strategy;
use crate::configuration::{Configuration, Movement};
use crate::shmem::AtomicMove;
use std::fmt;

/// Min-Max algorithm with a given recursion depth.
pub struct MinMax(pub u8);

/// .
///
/// # Panics
///
/// Panics if .
pub fn min_max_rec(
    player: bool,
    state: &Configuration,
    depth: u8,
) -> Option<(i8, Option<Movement>)> {
    if depth == 0 {
        return Some((
            state.value()
                * if state.current_player == player {
                    1
                } else {
                    -1
                },
            None,
        ));
    }

    let ok_moves = state.movements().filter(|mov| state.check_move(mov));

    let mut check_moves_size = ok_moves.peekable();

    // If no move is doable, return the value
    if check_moves_size.peek() == None {
        return Some((
            state.value()
                * if state.current_player == player {
                    1
                } else {
                    -1
                },
            None,
        ));
    }

    if depth > 1 {
        let nodes = check_moves_size.par_bridge().filter_map(|mov| {
            Some((
                min_max_rec(player, &state.play(&mov), depth - 1)?.0,
                Some(mov),
            ))
        });
        if state.current_player == player {
            nodes.min_by_key(|a: &(i8, Option<Movement>)| a.0)
        } else {
            nodes.max_by_key(|a: &(i8, Option<Movement>)| a.0)
        }
    } else {
        let nodes = check_moves_size.filter_map(|mov| {
            Some((
                min_max_rec(player, &state.play(&mov), depth - 1)?.0,
                Some(mov),
            ))
        });
        if state.current_player == player {
            nodes.min_by_key(|a: &(i8, Option<Movement>)| a.0)
        } else {
            nodes.max_by_key(|a: &(i8, Option<Movement>)| a.0)
        }
    }
}

impl Strategy for MinMax {
    fn compute_next_move(&mut self, state: &Configuration) -> Option<Movement> {
        if let Some((_, mov)) = min_max_rec(state.current_player, state, self.0) {
            mov
        } else {
            None
        }
    }
}

impl fmt::Display for MinMax {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Min - Max (max level: {})", self.0)
    }
}

/// Anytime min max algorithm.
/// Any time algorithms will compute until a deadline is hit and the process is killed.
/// They are therefore run in another process and communicate through shared memory.
/// This function is intended to be called from blobwar_iterative_deepening.
pub fn min_max_anytime(state: &Configuration) {
    let mut movement = AtomicMove::connect().expect("failed connecting to shmem");
    for depth in 1..100 {
        movement.store(MinMax(depth).compute_next_move(state));
    }
}
