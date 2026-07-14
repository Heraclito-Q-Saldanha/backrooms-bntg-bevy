use wfc::Tile;

#[derive(Debug, Clone, Copy, PartialEq, Eq, wfc::Tiled)]
#[file = "tests/tiled_macro.json"]
enum ExampleTile {
	Empty = 0,
	Dirt = 1,
	Stone = 2,
}

#[test]
fn tiled_derive() {
	assert!(ExampleTile::Empty.weight() == 3);
	assert_eq!(ExampleTile::all().len(), 3);
	assert_eq!(ExampleTile::Dirt.allowed_up(), &[ExampleTile::Stone]);
	assert_eq!(ExampleTile::Dirt.allowed_right(), &[ExampleTile::Stone]);
	assert_eq!(ExampleTile::Stone.allowed_left(), &[ExampleTile::Dirt]);
}
