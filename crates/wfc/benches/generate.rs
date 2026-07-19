use criterion::*;
use rand::SeedableRng;
use wfc::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, wfc::Tiled)]
#[file = "tests/tiled_macro.json"]
enum ExampleTile {
	Empty = 0,
	Dirt = 1,
	Stone = 2,
}

fn generate(bencher: &mut Bencher) {
	bencher.iter(|| {
		for i in [32, 64, 128, 256] {
			let size = I64Vec2::new(i, i);
			let mut seed = [0; 32usize];
			seed[0] = i as u8;
			let mut rng = rand::rngs::SmallRng::from_seed(seed);
			let _map = map::Map2D::<ExampleTile>::generate(size, &mut rng);
		}
	});
}

fn benchmark(criterion: &mut Criterion) {
	criterion.bench_function("generate", generate);
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
