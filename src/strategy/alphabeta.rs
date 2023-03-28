//! Alpha - Beta algorithm.
use std::fmt;

use super::Strategy;
use crate::configuration::{Configuration, Movement};
use crate::shmem::AtomicMove;

fn alpha_beta_rec(
    player: bool,
    state: &Configuration,
    depth: u8,
    alpha: i8,
    beta: i8,
) -> i8 {
    if depth == 0 {
        return state.value() 
    }

    let ok_moves = state.movements().filter(|mov| state.check_move(mov));

    let mut check_moves_size = ok_moves.peekable();

    // If no move is doable, return the value (right now the game can't end)
    if check_moves_size.peek() == None {
        return state.value()
    }

    if state.current_player == player {
        check_moves_size
            .try_fold((alpha, beta, i8::MAX), move |(alpha, beta, v), mov| {
                let resval = alpha_beta_rec(player, &state.play(&mov), depth - 1, alpha, beta);
                let mut new_v = v;
                let mut new_beta = beta;
                if resval < v {
                    new_v = resval
                };
                if new_v <= alpha {
                    return Err((alpha, beta, new_v));
                };
                if new_v < beta {
                    new_beta = new_v
                };
                Ok((alpha, new_beta, new_v))
            })
            .unwrap_or_else(|(alpha, beta, v)| (alpha, beta, v))
            .2
    } else {
        check_moves_size
            .try_fold((alpha, beta, i8::MAX), move |(alpha, beta, v), mov| {
                let resval = alpha_beta_rec(player, &state.play(&mov), depth - 1, alpha, beta);
                let mut new_v = v;
                let mut new_alpha = alpha;
                if resval > v {
                    new_v = resval
                };
                if new_v >= beta {
                    return Err((alpha, beta, new_v));
                };
                if new_v > alpha {
                    new_alpha = new_v
                };
                Ok((new_alpha, beta, new_v))
            })
            .unwrap_or_else(|(alpha, beta, v)| (alpha, beta, v))
            .2
    }
}

/// Anytime alpha beta algorithm.
/// Any time algorithms will compute until a deadline is hit and the process is killed.
/// They are therefore run in another process and communicate through shared memory.
/// This function is intended to be called from blobwar_iterative_deepening.
pub fn alpha_beta_anytime(state: &Configuration) {
    let mut movement = AtomicMove::connect().expect("failed connecting to shmem");
    for depth in 1..100 {
        let chosen_movement = AlphaBeta(depth).compute_next_move(state);
        movement.store(chosen_movement);
    }
}

/// Alpha - Beta algorithm with given maximum number of recursions.
pub struct AlphaBeta(pub u8);

impl fmt::Display for AlphaBeta {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Alpha - Beta (max level: {})", self.0)
    }
}

impl Strategy for AlphaBeta {
    fn compute_next_move(&mut self, state: &Configuration) -> Option<Movement> {
        state
            .movements()
            .filter(|mov| state.check_move(mov))
            .try_fold((i8::MIN, i8::MAX, i8::MAX, None), |(alpha, beta, v, _), mov| {
                let resval = alpha_beta_rec(state.current_player, &state.play(&mov), self.0 - 1, alpha, beta);
                let mut new_v = v;
                let mut new_beta = beta;
                if resval < v {
                    new_v = resval
                };
                if new_v <= alpha {
                    return Err((alpha, beta, new_v, Some(mov)));
                };
                if new_v < beta {
                    new_beta = new_v
                };
                Ok((alpha, new_beta, new_v, Some(mov)))
            })
            .unwrap_or_else(|(alpha, beta, v, mov)| (alpha, beta, v, mov)).3
    }
}
