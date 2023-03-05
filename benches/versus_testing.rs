#[macro_use]
extern crate bencher;
use bencher::Bencher;
use blobwar::{configuration::Configuration, strategy::{MinMax, Greedy}};

fn bench_MinMax(b: &mut Bencher) {
    let board = Default::default();
    let mut game = Configuration::new(&board);
    b.iter(|| game.battle(Greedy(), Greedy()));
}

benchmark_group!(benches, bench_MinMax);
benchmark_main!(benches);
