use bevy::prelude::*;

pub fn change_bg_on_pointer<E: std::fmt::Debug + Clone + Reflect>(bg_color: Color) -> impl Scene {
	on(move |trigger: On<Pointer<E>>, mut query: Query<&mut BackgroundColor>| {
		if let Ok(mut bg) = query.get_mut(trigger.entity) {
			bg.0 = bg_color;
		}
	})
}
