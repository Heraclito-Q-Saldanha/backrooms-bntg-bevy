pub mod map;

pub use wfc_derive::*;

pub trait Tile: Sized {
	fn weight(&self) -> u16;

	fn all() -> &'static [Self];

	fn allowed_left(&self) -> &'static [Self];
	fn allowed_right(&self) -> &'static [Self];
	fn allowed_up(&self) -> &'static [Self];
	fn allowed_down(&self) -> &'static [Self];
}
