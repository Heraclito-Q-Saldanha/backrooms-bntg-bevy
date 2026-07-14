use bevy::prelude::*;
use bevy::ui::*;

pub fn change_bg_on_pointer_if_enable<E: std::fmt::Debug + Clone + Reflect>(bg_color: Color) -> impl Scene {
	on(move |trigger: On<Pointer<E>>, mut query: Query<&mut BackgroundColor, Without<InteractionDisabled>>| {
		if let Ok(mut bg) = query.get_mut(trigger.entity) {
			bg.0 = bg_color;
		}
	})
}

pub fn change_bg_on_pointer_if_disable<E: std::fmt::Debug + Clone + Reflect>(bg_color: Color) -> impl Scene {
	on(move |trigger: On<Pointer<E>>, mut query: Query<&mut BackgroundColor, With<InteractionDisabled>>| {
		if let Ok(mut bg) = query.get_mut(trigger.entity) {
			bg.0 = bg_color;
		}
	})
}
