use wfc::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, wfc::Tiled)]
#[file = "tests/tiled_macro.json"]
enum ExampleTile {
	Empty = 0,
	Dirt = 1,
	Stone = 2,
}

#[test]
fn tiled_derive() {
	assert_eq!(ExampleTile::all().len(), 3);

	assert!(ExampleTile::Empty.weight() == 3);
	assert!(ExampleTile::Dirt.is_allowed_up(&ExampleTile::Stone));
	assert!(ExampleTile::Dirt.is_allowed_right(&ExampleTile::Stone));
	assert!(ExampleTile::Stone.is_allowed_left(&ExampleTile::Dirt));

	let wave = probability::Cell::Wave([ExampleTile::Dirt, ExampleTile::Stone].to_vec());

	assert!(wave.is_allowed_right(&ExampleTile::Stone));
	assert!(wave.is_allowed_left(&ExampleTile::Dirt));
}
