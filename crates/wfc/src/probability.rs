use crate::*;

use rand::distr::*;
use rand::seq::*;
use rand::*;
use std::collections;

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Debug, Clone)]
pub struct ProbabilityMap<T> {
	data: map::Map2D<Cell<T>>,
	size: I64Vec2,
}

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Debug, Clone)]
pub enum Cell<T> {
	Wave(Vec<T>),
	Collapsed(T),
}

impl<T: Tile + 'static> ProbabilityMap<T> {
	pub fn new(size: I64Vec2) -> Self {
		let default = Cell::Wave(T::all().to_vec());
		let data = map::Map2D::new(size, default);

		Self { data, size }
	}
	pub fn step<R: rand::Rng>(&mut self, rng: &mut R) -> Result<bool, ()> {
		if let Some(position) = self.next_cell(rng) {
			self.collapse(position, rng)?;
			self.propagate(position)?;

			Ok(false)
		} else {
			Ok(true)
		}
	}
	pub fn generate<R: rand::Rng>(&mut self, rng: &mut R) -> Result<(), ()> {
		while !self.step(rng)? {}
		Ok(())
	}
	pub fn into_map(self) -> Result<map::Map2D<T>, ()> {
		let size = self.size;
		let data = self
			.data
			.into_vec()
			.into_iter()
			.map(|cell| match cell {
				Cell::Collapsed(cell) => Ok(cell),
				Cell::Wave(_) => Err(()),
			})
			.collect::<Result<Vec<_>, _>>()?;

		Ok(map::Map2D::from_vec(data, size))
	}
	fn next_cell<R: Rng>(&self, rng: &mut R) -> Option<I64Vec2> {
		let mut indexes = Vec::new();
		let mut min = usize::MAX;

		for (index, cell) in self.data.iter().enumerate() {
			let Cell::Wave(wave) = cell else { continue };

			let len = wave.len();
			if len < min {
				min = len;
				indexes.clear();
			}
			if len == min {
				indexes.push(index);
			}
		}

		let Some(index) = indexes.choose(rng) else {
			return None;
		};

		Some(self.data.position(*index))
	}
	fn collapse<R: Rng>(&mut self, position: I64Vec2, rng: &mut R) -> Result<(), ()> {
		let Some(Cell::Wave(current)) = self.data.get_cell(position) else {
			return Err(());
		};

		let weights = current.iter().map(|cell| cell.weight());
		let distribution = distr::weighted::WeightedIndex::new(weights).map_err(|_| ())?;
		let index = distribution.sample(rng);
		let cell = current[index].clone();

		self.data.set_cell(position, Cell::Collapsed(cell));

		Ok(())
	}
	fn propagate(&mut self, position: I64Vec2) -> Result<(), ()> {
		let mut queue = collections::VecDeque::new();

		queue.push_back(position);

		while let Some(position) = queue.pop_front() {
			let Some(current) = self.data.get_cell(position) else {
				break;
			};

			let current = current.clone();

			let right = position + i64vec2(1, 0);
			let left = position + i64vec2(-1, 0);
			let up = position + i64vec2(0, 1);
			let down = position + i64vec2(0, -1);

			if let Some(Cell::Wave(neighbor)) = self.data.get_cell_mut(right) {
				let len = neighbor.len();

				neighbor.retain(|candidate| current.is_allowed_right(candidate));

				if neighbor.is_empty() {
					return Err(());
				}

				if len != neighbor.len() {
					queue.push_back(right);
				}
			}
			if let Some(Cell::Wave(neighbor)) = self.data.get_cell_mut(left) {
				let len = neighbor.len();

				neighbor.retain(|candidate| current.is_allowed_left(candidate));

				if neighbor.is_empty() {
					return Err(());
				}

				if len != neighbor.len() {
					queue.push_back(left);
				}
			}
			if let Some(Cell::Wave(neighbor)) = self.data.get_cell_mut(up) {
				let len = neighbor.len();

				neighbor.retain(|candidate| current.is_allowed_up(candidate));

				if neighbor.is_empty() {
					return Err(());
				}

				if len != neighbor.len() {
					queue.push_back(up);
				}
			}
			if let Some(Cell::Wave(neighbor)) = self.data.get_cell_mut(down) {
				let len = neighbor.len();

				neighbor.retain(|candidate| current.is_allowed_down(candidate));

				if neighbor.is_empty() {
					return Err(());
				}

				if len != neighbor.len() {
					queue.push_back(down);
				}
			}
		}

		Ok(())
	}
}

impl<T: Tile + 'static> Cell<T> {
	#[inline(always)]
	pub const fn is_collesed(&self) -> bool {
		matches!(self, Cell::Collapsed(_))
	}
	#[inline(always)]
	pub fn is_allowed_up(&self, cell: &T) -> bool {
		match self {
			Self::Wave(w) => {
				for i in w {
					if T::is_allowed_up(i, cell) {
						return true;
					}
				}
				false
			}
			Self::Collapsed(c) => T::is_allowed_up(c, cell),
		}
	}
	#[inline(always)]
	pub fn is_allowed_down(&self, cell: &T) -> bool {
		match self {
			Self::Wave(w) => {
				for i in w {
					if T::is_allowed_down(i, cell) {
						return true;
					}
				}
				false
			}
			Self::Collapsed(c) => T::is_allowed_down(c, cell),
		}
	}
	#[inline(always)]
	pub fn is_allowed_left(&self, cell: &T) -> bool {
		match self {
			Self::Wave(w) => {
				for i in w {
					if T::is_allowed_left(i, cell) {
						return true;
					}
				}
				false
			}
			Self::Collapsed(c) => T::is_allowed_left(c, cell),
		}
	}
	#[inline(always)]
	pub fn is_allowed_right(&self, cell: &T) -> bool {
		match self {
			Self::Wave(w) => {
				for i in w {
					if T::is_allowed_right(i, cell) {
						return true;
					}
				}
				false
			}
			Self::Collapsed(c) => T::is_allowed_right(c, cell),
		}
	}
}
