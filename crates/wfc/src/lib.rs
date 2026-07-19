pub mod map;
pub mod probability;

pub use bevy_math::*;
pub use wfc_derive::*;

pub trait Tile: Sized + Clone + PartialEq {
	fn weight(&self) -> u16;

	fn all() -> &'static [Self];

	fn is_allowed_left(&self, tile: &Self) -> bool;
	fn is_allowed_right(&self, tile: &Self) -> bool;
	fn is_allowed_up(&self, tile: &Self) -> bool;
	fn is_allowed_down(&self, tile: &Self) -> bool;
}
