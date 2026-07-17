use rand::distr::*;
use rand::seq::*;
use rand::*;
use std::collections;

pub use bevy_math::*;
pub use wfc_derive::*;

pub trait Tile: Sized {
	fn weight(&self) -> u16;

	fn all() -> &'static [Self];

	fn allowed_left(&self) -> &'static [Self];
	fn allowed_right(&self) -> &'static [Self];
	fn allowed_up(&self) -> &'static [Self];
	fn allowed_down(&self) -> &'static [Self];
}

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Debug, Clone)]
pub struct Map2D<T> {
	data: Vec<T>,
	size: I64Vec2,
}

impl<T: Clone> Map2D<T> {
	pub fn new(size: I64Vec2, default: T) -> Self {
		let data = vec![default; (size.x * size.y) as usize];
		Self { data, size }
	}
	pub fn from_vec(data: Vec<T>, size: I64Vec2) -> Self {
		let len = (size.x * size.y) as usize;
		assert!(data.len() == len);
		Self { data, size }
	}
	#[inline(always)]
	pub fn index(&self, position: I64Vec2) -> usize {
		((self.size.x * position.y) + position.x) as usize
	}
	#[inline(always)]
	pub fn size(&self) -> I64Vec2 {
		self.size
	}
	#[inline(always)]
	pub fn position(&self, index: usize) -> I64Vec2 {
		let index = index as i64;
		I64Vec2 {
			x: index % self.size.x,
			y: index / self.size.x,
		}
	}
	#[inline(always)]
	pub fn is_in_range(&self, position: I64Vec2) -> bool {
		position.x >= 0 && position.y >= 0 && position.x < self.size.x && position.y < self.size.y
	}
	#[inline(always)]
	pub fn set_tile(&mut self, position: I64Vec2, tile: T) {
		if !self.is_in_range(position) {
			return;
		}
		let index = self.index(position);
		*unsafe { self.data.get_unchecked_mut(index) } = tile;
	}
	#[inline(always)]
	pub fn get_tile(&self, position: I64Vec2) -> Option<T> {
		if !self.is_in_range(position) {
			return None;
		}
		let index = self.index(position);
		Some(unsafe { self.data.get_unchecked(index) }.clone())
	}
}

impl<T: Tile + Clone + 'static + PartialEq> Map2D<T> {
	pub fn generate(size: I64Vec2, seed: u64) -> Result<Map2D<T>, ()> {
		let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
		let mut state = Map2D::new(size, T::all().to_vec());

		loop {
			let Some(min) = state.data.iter().filter(|set| set.len() > 1).map(Vec::len).min() else {
				break;
			};
			let candidates: Vec<_> = state.data.iter().enumerate().filter(|(_, set)| set.len() == min).map(|(index, _)| state.position(index)).collect();
			let Some(position) = candidates.choose(&mut rng).cloned() else {
				break;
			};

			let Some(current) = state.get_tile(position) else {
				return Err(());
			};

			if current.is_empty() {
				return Err(());
			}

			let dist = distr::weighted::WeightedIndex::new(current.iter().map(|tile| tile.weight())).map_err(|_| ())?;
			let tile = current[dist.sample(&mut rng)].clone();

			state.set_tile(position, vec![tile]);

			let mut queue = collections::VecDeque::new();
			queue.push_back(position);

			while let Some(position) = queue.pop_front() {
				let Some(current) = state.get_tile(position) else {
					return Err(());
				};

				let right = position + i64vec2(1, 0);
				let left = position + i64vec2(-1, 0);
				let up = position + i64vec2(0, 1);
				let down = position + i64vec2(0, -1);

				if let Some(mut neighbor) = state.get_tile(right) {
					let len = neighbor.len();

					neighbor.retain(|candidate| current.iter().any(|tile| tile.allowed_right().contains(candidate)));

					if neighbor.is_empty() {
						return Err(());
					}

					if len != neighbor.len() {
						state.set_tile(right, neighbor);
						queue.push_back(right);
					}
				}
				if let Some(mut neighbor) = state.get_tile(left) {
					let len = neighbor.len();

					neighbor.retain(|candidate| current.iter().any(|tile| tile.allowed_left().contains(candidate)));

					if neighbor.is_empty() {
						return Err(());
					}

					if len != neighbor.len() {
						state.set_tile(left, neighbor);
						queue.push_back(left);
					}
				}
				if let Some(mut neighbor) = state.get_tile(up) {
					let len = neighbor.len();

					neighbor.retain(|candidate| current.iter().any(|tile| tile.allowed_up().contains(candidate)));

					if neighbor.is_empty() {
						return Err(());
					}

					if len != neighbor.len() {
						state.set_tile(up, neighbor);
						queue.push_back(up);
					}
				}
				if let Some(mut neighbor) = state.get_tile(down) {
					let len = neighbor.len();

					neighbor.retain(|candidate| current.iter().any(|tile| tile.allowed_down().contains(candidate)));

					if neighbor.is_empty() {
						return Err(());
					}

					if len != neighbor.len() {
						state.set_tile(down, neighbor);
						queue.push_back(down);
					}
				}
			}
		}

		let data = state.data.into_iter().flatten().collect();

		Ok(Self { data, size })
	}
}
