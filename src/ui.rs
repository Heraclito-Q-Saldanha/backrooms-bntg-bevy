use bevy::color::palettes::*;
use bevy::prelude::*;

pub fn button(label: &str, normal: Color, hover: Color) -> impl Scene {
	bsn! {
		Button
		Node {
			width: px(250),
			height: px(50),
			border: px(2),
			border_radius: BorderRadius::all(px(5)),
			justify_content: JustifyContent::Center,
			align_items: AlignItems::Center,
		}
		BorderColor::from(tailwind::ZINC_100)
		BackgroundColor(normal)
		Children [(
			Text(label)
			TextColor(tailwind::ZINC_100)
			TextShadow
		)]
		on(move |trigger: On<Pointer<Enter>>, mut query: Query<&mut BackgroundColor>|{
			if let Ok(mut bg) = query.get_mut(trigger.entity) {
				bg.0 = hover;
			}
		})
		on(move |trigger: On<Pointer<Leave>>, mut query: Query<&mut BackgroundColor>|{
			if let Ok(mut bg) = query.get_mut(trigger.entity) {
				bg.0 = normal;
			}
		})
	}
}
