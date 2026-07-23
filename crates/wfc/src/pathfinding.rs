use crate::*;

use ::pathfinding::prelude::*;

pub trait Walkable {
	fn is_walkable(&self) -> bool;
}

impl<T: Walkable + Tile> map::Map2D<T> {
	pub fn astar(&self, start: I64Vec2, goal: I64Vec2) -> Option<Vec<I64Vec2>> {
		let successors = |position: &I64Vec2| {
			const DIRS: [I64Vec2; 4] = [I64Vec2::new(1, 0), I64Vec2::new(-1, 0), I64Vec2::new(0, 1), I64Vec2::new(0, -1)];

			DIRS.into_iter()
				.filter_map(|dir| {
					let next = position + dir;

					match self.get_cell(next) {
						Some(tile) if tile.is_walkable() => Some((next, 1)),
						_ => None,
					}
				})
				.collect::<Vec<_>>()
		};
		let heuristic = |position: &I64Vec2| (goal.x - position.x).abs() + (goal.y - position.y).abs();
		let success = |position: &I64Vec2| *position == start;

		match astar(&start, successors, heuristic, success) {
			Some((result, _)) => Some(result),
			None => None,
		}
	}
}
