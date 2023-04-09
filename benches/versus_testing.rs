#[macro_use]
extern crate bencher;
use bencher::Bencher;
use blobwar::{
    configuration::*,
    strategy::{Greedy, MinMax, AlphaBeta, Strategy},
};

fn bench_MinMax(b: &mut Bencher) {
    let board = Default::default();
    let mut game = Configuration::new(&board);
    let mut player_one = AlphaBeta(3); // replace the strategy if you want.
    let mut player_two = AlphaBeta(3);
    b.iter(|| {
        while !game.game_over() {
            let play_attempt = if game.current_player {
                player_two.compute_next_move(&game)
            } else {
                player_one.compute_next_move(&game)
            };
            if let Some(ref next_move) = play_attempt {
                assert!(&game.check_move(next_move));
                game.apply_movement(next_move);
            } else {
                game.current_player = !game.current_player;
            }
        }
    });
}

benchmark_group!(benches, bench_MinMax);
benchmark_main!(benches);
