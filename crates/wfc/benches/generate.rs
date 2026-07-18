use criterion::*;
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
			let seed = i as u64;
			let _map = Map2D::<ExampleTile>::generate(size, seed);
		}
	});
}

fn benchmark(criterion: &mut Criterion) {
	criterion.bench_function("generate", generate);
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
