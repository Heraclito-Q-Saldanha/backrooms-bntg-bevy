use std::ops;

use crate::*;

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Debug, Clone)]
pub struct Map2D<T> {
	data: Vec<T>,
	size: I64Vec2,
}

impl<T: Clone> Map2D<T> {
	pub fn new(size: I64Vec2, default: T) -> Self {
		let capacity = (size.x * size.y) as usize;

		assert!(capacity >= 1);

		let data = vec![default; capacity];
		Self { data, size }
	}
	#[inline(always)]
	pub fn size(&self) -> I64Vec2 {
		self.size
	}
	#[inline(always)]
	pub fn index(&self, position: I64Vec2) -> usize {
		((self.size.x * position.y) + position.x) as usize
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
	pub fn set_cell(&mut self, position: I64Vec2, tile: T) {
		if !self.is_in_range(position) {
			return;
		}
		let index = self.index(position);
		*unsafe { self.data.get_unchecked_mut(index) } = tile;
	}
	#[inline(always)]
	pub fn get_cell(&self, position: I64Vec2) -> Option<&T> {
		if !self.is_in_range(position) {
			return None;
		}
		let index = self.index(position);
		Some(unsafe { self.data.get_unchecked(index) })
	}
	#[inline(always)]
	pub fn get_cell_mut(&mut self, position: I64Vec2) -> Option<&mut T> {
		if !self.is_in_range(position) {
			return None;
		}
		let index = self.index(position);
		Some(unsafe { self.data.get_unchecked_mut(index) })
	}
	#[inline(always)]
	pub fn into_vec(self) -> Vec<T> {
		self.data
	}
	#[inline(always)]
	pub fn from_vec(data: Vec<T>, size: I64Vec2) -> Self {
		let capacity = (size.x * size.y) as usize;
		assert!(capacity == data.len());
		Self { data, size }
	}
}

impl<T: Tile + 'static> Map2D<T> {
	pub fn generate<R: rand::Rng>(size: I64Vec2, rng: &mut R) -> Result<Self, ()> {
		let mut probability_map = probability::ProbabilityMap::<T>::new(size);
		probability_map.generate(rng)?;

		Ok(probability_map.into_map()?)
	}
}

impl<T> ops::Deref for Map2D<T> {
	type Target = [T];

	#[inline(always)]
	fn deref(&self) -> &Self::Target {
		&self.data
	}
}
