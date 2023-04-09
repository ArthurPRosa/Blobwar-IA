//! Alpha - Beta algorithm.
use std::fmt;

use rayon::prelude::{ParallelBridge, ParallelIterator};

use super::Strategy;
use crate::configuration::{Configuration, Movement};
use crate::shmem::AtomicMove;

fn alpha_beta_par_rec(
    player: bool,
    state: &Configuration,
    depth: u8,
    alpha: i8,
    beta: i8,
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
        let (_, _, val, mov, _) = if state.current_player == player {
            check_moves_size
                .par_bridge()
                .fold(
                    || (alpha, beta, i8::MAX, None, true),
                    |(alpha, beta, v, old_mov, should_continue), new_mov| {
                        //We use try_fold to be able to break from the fold, and we simply return the result from the last Ok or first Err with the identity in unwrap_or_else
                        if !should_continue {
                            return (alpha, beta, v, old_mov, false);
                        } //Softly break out of the fold

                        if let Some((resval, _)) = alpha_beta_par_rec(
                            player,
                            &state.play(&new_mov),
                            depth - 1,
                            alpha,
                            beta,
                        ) {
                            let new_v = v.min(resval);
                            if new_v < alpha {
                                return (alpha, beta, new_v, Some(new_mov), false);
                            };

                            let (new_beta, best_mov) = if new_v < beta {
                                (new_v, Some(new_mov))
                            } else {
                                (beta, old_mov)
                            };

                            (alpha, new_beta, new_v, best_mov, true)
                        } else {
                            (alpha, beta, v, None, true)
                        }
                    },
                )
                .reduce(
                    || (alpha, beta, i8::MAX, None, true),
                    |(alpha, beta, old_v, old_mov, _), (_, _, v, mov, _)| {
                        if v < old_v {
                            (alpha, beta, v, mov, true)
                        } else {
                            (alpha, beta, old_v, old_mov, true)
                        }
                    },
                )
        } else {
            check_moves_size
                .par_bridge()
                .fold(
                    || (alpha, beta, i8::MIN, None, true),
                    |(alpha, beta, v, old_mov, should_continue), new_mov| {
                        //We use try_fold to be able to break from the fold, and we simply return the result from the last Ok or first Err with the identity in unwrap_or_else
                        if !should_continue {
                            return (alpha, beta, v, old_mov, false);
                        } //Softly break out of the fold

                        if let Some((resval, _)) = alpha_beta_par_rec(
                            player,
                            &state.play(&new_mov),
                            depth - 1,
                            alpha,
                            beta,
                        ) {
                            let new_v = v.max(resval);
                            if new_v > beta {
                                return (alpha, beta, new_v, Some(new_mov), false);
                            }

                            let (new_alpha, best_mov) = if new_v > alpha {
                                (new_v, Some(new_mov))
                            } else {
                                (alpha, old_mov)
                            };

                            (new_alpha, beta, new_v, best_mov, true)
                        } else {
                            (alpha, beta, v, None, true)
                        }
                    },
                )
                .reduce(
                    || (alpha, beta, i8::MIN, None, true),
                    |(alpha, beta, old_v, old_mov, _), (_, _, v, mov, _)| {
                        if v > old_v {
                            (alpha, beta, v, mov, true)
                        } else {
                            (alpha, beta, old_v, old_mov, true)
                        }
                    },
                )
        };
        Some((val, mov))
    } else {
        let (_, _, val, mov) = if state.current_player == player {
            check_moves_size
                .try_fold(
                    (alpha, beta, i8::MAX, None),
                    |(alpha, beta, v, old_mov), new_mov| {
                        //We use try_fold to be able to break from the fold, and we simply return the result from the last Ok or first Err with the identity in unwrap_or_else
                        if let Some((resval, _)) = alpha_beta_par_rec(
                            player,
                            &state.play(&new_mov),
                            depth - 1,
                            alpha,
                            beta,
                        ) {
                            let new_v = v.min(resval);
                            if new_v < alpha {
                                return Err((alpha, beta, new_v, Some(new_mov)));
                            };

                            let (new_beta, best_mov) = if new_v < beta {
                                (new_v, Some(new_mov))
                            } else {
                                (beta, old_mov)
                            };

                            Ok((alpha, new_beta, new_v, best_mov))
                        } else {
                            Ok((alpha, beta, v, None))
                        }
                    },
                )
                .unwrap_or_else(|a| a)
        } else {
            check_moves_size
                .try_fold(
                    (alpha, beta, i8::MIN, None),
                    |(alpha, beta, v, old_mov), new_mov| {
                        //We use try_fold to be able to break from the fold, and we simply return the result from the last Ok or first Err with the identity in unwrap_or_else
                        if let Some((resval, _)) = alpha_beta_par_rec(
                            player,
                            &state.play(&new_mov),
                            depth - 1,
                            alpha,
                            beta,
                        ) {
                            let new_v = v.max(resval);
                            if new_v > beta {
                                return Err((alpha, beta, new_v, Some(new_mov)));
                            }

                            let (new_alpha, best_mov) = if new_v > alpha {
                                (new_v, Some(new_mov))
                            } else {
                                (alpha, old_mov)
                            };

                            Ok((new_alpha, beta, new_v, best_mov))
                        } else {
                            Ok((alpha, beta, v, None))
                        }
                    },
                )
                .unwrap_or_else(|a| a)
        };
        Some((val, mov))
    }
}

/// Anytime alpha beta algorithm.
/// Any time algorithms will compute until a deadline is hit and the process is killed.
/// They are therefore run in another process and communicate through shared memory.
/// This function is intended to be called from blobwar_iterative_deepening.
pub fn alpha_beta_par_anytime(state: &Configuration) {
    let mut movement = AtomicMove::connect().expect("failed connecting to shmem");
    for depth in 2..100 {
        let chosen_movement = AlphaBetaPar(depth).compute_next_move(state);
        movement.store(chosen_movement);
    }
}

/// Alpha - Beta algorithm with given maximum number of recursions.
pub struct AlphaBetaPar(pub u8);

impl fmt::Display for AlphaBetaPar {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Alpha - Beta Par (max level: {})", self.0)
    }
}

impl Strategy for AlphaBetaPar {
    fn compute_next_move(&mut self, state: &Configuration) -> Option<Movement> {
        let (_, mov) = alpha_beta_par_rec(state.current_player, &state, self.0, i8::MIN, i8::MAX)?;
        mov
    }
}
