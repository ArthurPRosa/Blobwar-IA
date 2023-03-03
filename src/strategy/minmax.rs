//! Implementation of the min max algorithm.
use itertools::Itertools;
use rayon::prelude::{ParallelIterator, IntoParallelIterator};

use super::Strategy;
use crate::configuration::{Configuration, Movement};
use crate::shmem::AtomicMove;
use std::fmt;

/// Min-Max algorithm with a given recursion depth.
pub struct MinMax(pub u8);

fn min_max_rec(state: &Configuration, depth: u8) -> Option<(i8, Option<Movement>)> {
    let ok_moves = state.par_movements().filter(|mov| state.check_move(mov));

    // If no move is doable, return the value (right now the game can't end)

    let nodes =
        ok_moves.filter_map(|mov| Some((min_max_rec(&state.play(&mov), depth - 1)?.0, Some(mov))));

    // if state.current_player {
    //     nodes.min_by_key(|a: &Option<(i8, Option<Movement>)>| a.unwrap_or((i8::MAX, None)).0)?

    // } else {
    //     nodes.max_by_key(|a: &Option<(i8, Option<Movement>)>| a.unwrap_or((i8::MIN, None)).0)?
    // }
    nodes.min_by_key(|a: &(i8, Option<Movement>)| a.0)
}

impl Strategy for MinMax {
    fn compute_next_move(&mut self, state: &Configuration) -> Option<Movement> {
        if let Some((_, mov)) = min_max_rec(state, self.0) {
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
///
///function minimax(node, depth, maximizingPlayer) is
///    if depth = 0 or node is a terminal node then <br>
///    return the heuristic value of node <br>
///if maximizingPlayer then <br>
///    value := −∞ <br>
///    for each child of node do <br>
///        value := max(value, minimax(child, depth − 1, FALSE)) <br>
///else (* minimizing player *) <br>
///    value := +∞ <br>
///    for each child of node do <br>
///        value := min(value, minimax(child, depth − 1, TRUE)) <br>
///return value <br>
pub fn min_max_anytime(state: &Configuration) {
    let mut movement = AtomicMove::connect().expect("failed connecting to shmem");
    for depth in 1..100 {
        movement.store(MinMax(depth).compute_next_move(state));
    }
}
