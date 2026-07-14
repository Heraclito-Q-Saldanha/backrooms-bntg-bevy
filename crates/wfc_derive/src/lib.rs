mod tiled;

use std::collections;
use std::fs;
use std::path;

use quote::quote;

#[proc_macro_derive(Tiled, attributes(file))]
pub fn tiled(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
	let input = syn::parse_macro_input!(item as syn::DeriveInput);
	let ident = input.ident.clone();

	let syn::Data::Enum(data) = input.data else { panic!("Only enums") };

	let mut variants = Vec::new();
	for variant in data.variants {
		let discriminant = match variant.discriminant {
			Some((_, syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Int(lit), .. }))) => lit.base10_parse::<i64>().unwrap(),
			Some((_, expr)) => panic!("Unsupported discriminant expression: {expr:?}"),
			None => panic!("Each Tiled variant must have an explicit integer discriminant"),
		};
		variants.push((variant.ident, discriminant));
	}

	let mut counts = collections::HashMap::<i64, u16>::new();
	let mut allowed_left = collections::HashMap::<i64, collections::HashSet<i64>>::new();
	let mut allowed_right = collections::HashMap::<i64, collections::HashSet<i64>>::new();
	let mut allowed_up = collections::HashMap::<i64, collections::HashSet<i64>>::new();
	let mut allowed_down = collections::HashMap::<i64, collections::HashSet<i64>>::new();

	let file_path = input.attrs.iter().find_map(|attr| {
		let syn::Meta::NameValue(value) = &attr.meta else {
			return None;
		};
		let Some(name) = value.path.get_ident() else {
			return None;
		};
		if name != "file" {
			return None;
		}
		let syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(path), .. }) = &value.value else {
			return None;
		};
		Some(path.value())
	});

	let Some(file_path) = file_path else {
		panic!("The Tiled derive requires a #[file = \"path/to/map.json\"] attribute");
	};

	let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
	let path = path::PathBuf::from(manifest_dir).join(file_path);
	let data = fs::read_to_string(&path).unwrap();
	let tile_map = serde_json::from_str::<tiled::TileMap>(&data).unwrap();

	for layer in tile_map.layers {
		let width = layer.width;
		let height = layer.height;
		let data = layer.data;

		for y in 0..height {
			for x in 0..width {
				let index = y * width + x;
				let value = *data.get(index).unwrap_or(&0);
				*counts.entry(value).or_insert(0) += 1;

				if x > 0 {
					let left_value = *data.get(index - 1).unwrap_or(&0);
					allowed_left.entry(value).or_default().insert(left_value);
				}
				if x + 1 < width {
					let right_value = *data.get(index + 1).unwrap_or(&0);
					allowed_right.entry(value).or_default().insert(right_value);
				}
				if y > 0 {
					let up_value = *data.get(index - width).unwrap_or(&0);
					allowed_up.entry(value).or_default().insert(up_value);
				}
				if y + 1 < height {
					let down_value = *data.get(index + width).unwrap_or(&0);
					allowed_down.entry(value).or_default().insert(down_value);
				}
			}
		}
	}

	let variant_lookup: collections::HashMap<i64, syn::Ident> = variants.iter().cloned().map(|(ident, value)| (value, ident)).collect();
	let build_expr = |values: &collections::HashSet<i64>| {
		let neighbors: Vec<_> = values.iter().filter_map(|value| variant_lookup.get(value).cloned()).map(|variant_ident| quote! { Self::#variant_ident }).collect();

		if neighbors.is_empty() {
			quote! { &[] }
		} else {
			quote! { &[ #(#neighbors),* ] }
		}
	};

	let mut weights = quote! {};
	let mut all = quote! {};
	let mut left = quote! {};
	let mut right = quote! {};
	let mut up = quote! {};
	let mut down = quote! {};

	for (variant_ident, value) in variants {
		let weight = counts.get(&value).copied().unwrap_or(1);
		let left_expr = build_expr(allowed_left.get(&value).unwrap_or(&collections::HashSet::new()));
		let right_expr = build_expr(allowed_right.get(&value).unwrap_or(&collections::HashSet::new()));
		let up_expr = build_expr(allowed_up.get(&value).unwrap_or(&collections::HashSet::new()));
		let down_expr = build_expr(allowed_down.get(&value).unwrap_or(&collections::HashSet::new()));

		weights = quote! {
			#weights
			Self::#variant_ident => #weight,
		};

		all = quote! {
			#all
			Self::#variant_ident,
		};

		left = quote! {
			#left
			Self::#variant_ident => #left_expr,
		};
		right = quote! {
			#right
			Self::#variant_ident => #right_expr,
		};
		up = quote! {
			#up
			Self::#variant_ident => #up_expr,
		};
		down = quote! {
			#down
			Self::#variant_ident => #down_expr,
		};
	}

	quote! {
		impl ::wfc::Tile for #ident {
			fn weight(&self) -> u16 {
				match self {
					#weights
				}
			}

			fn all() -> &'static [Self] {
				&[
					#all
				]
			}

			fn allowed_left(&self) -> &'static [Self] {
				match self {
					#left
				}
			}

			fn allowed_right(&self) -> &'static [Self] {
				match self {
					#right
				}
			}

			fn allowed_up(&self) -> &'static [Self] {
				match self {
					#up
				}
			}

			fn allowed_down(&self) -> &'static [Self] {
				match self {
					#down
				}
			}
		}
	}
	.into()
}

#[proc_macro_derive(Tile, attributes(weight, constraint))]
pub fn tile(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
	let input = syn::parse_macro_input!(item as syn::DeriveInput);
	let ident = input.ident;
	let syn::Data::Enum(data_enum) = input.data else { panic!("Only enuns is suported") };

	let mut weights = quote! {};
	let mut all = quote! {};

	let mut left = quote! {};
	let mut right = quote! {};
	let mut up = quote! {};
	let mut down = quote! {};

	for var in data_enum.variants {
		let mut weight: u16 = 1;
		let ident = var.ident;

		let mut left_value = quote! {};
		let mut right_value = quote! {};
		let mut up_value = quote! {};
		let mut down_value = quote! {};

		for attr in var.attrs {
			let syn::Meta::List(list) = attr.meta else {
				panic!("Only list attributes is suported");
			};

			let attr_ident = list.path.get_ident().unwrap();

			match attr_ident.to_string().as_str() {
				"weight" => {
					let tokens = list.tokens.into();
					weight = syn::parse_macro_input!(tokens as syn::LitInt).base10_parse().unwrap();
				}
				"constraint" => {
					let tokens = list.tokens.clone();
					let parsed: syn::MetaNameValue = syn::parse2(tokens).unwrap();
					let direction = parsed.path.get_ident().unwrap().to_string();
					if let syn::Expr::Array(arr) = &parsed.value {
						match direction.as_str() {
							"all" => {
								for element in &arr.elems {
									left_value = quote! {
										#left_value
										Self:: #element,
									};
									right_value = quote! {
										#right_value
										Self:: #element,
									};
									up_value = quote! {
										#up_value
										Self:: #element,
									};
									down_value = quote! {
										#down_value
										Self:: #element,
									};
								}
							}
							"left" => {
								for element in &arr.elems {
									left_value = quote! {
										#left_value
										Self:: #element,
									};
								}
							}
							"right" => {
								for element in &arr.elems {
									right_value = quote! {
										#right_value
										Self:: #element,
									};
								}
							}
							"up" => {
								for element in &arr.elems {
									up_value = quote! {
										#up_value
										Self:: #element,
									};
								}
							}
							"down" => {
								for element in &arr.elems {
									down_value = quote! {
										#down_value
										Self:: #element,
									};
								}
							}
							_ => {}
						}
					}
				}
				_ => {}
			}
		}

		left = quote! {
			#left
			Self::#ident => &[#left_value],
		};
		right = quote! {
			#right
			Self::#ident => &[#right_value],
		};
		up = quote! {
			#up
			Self::#ident => &[#up_value],
		};
		down = quote! {
			#down
			Self::#ident => &[#down_value],
		};

		weights = quote! {
			#weights
			Self::#ident => #weight,
		};

		all = quote! {
			#all
			Self:: #ident,
		}
	}

	quote! {
		impl ::wfc::Tile for #ident {
			fn weight(&self) -> u16 {
				match self {
					#weights
				}
			}

			fn all() -> &'static [Self] {
				&[
					#all
				]
			}

			fn allowed_left(&self) -> &'static [Self] {
				match self {
					#left
				}
			}

			fn allowed_right(&self) -> &'static [Self] {
				match self {
					#right
				}
			}

			fn allowed_up(&self) -> &'static [Self] {
				match self {
					#up
				}
			}

			fn allowed_down(&self) -> &'static [Self] {
				match self {
					#down
				}
			}
		}
	}
	.into()
}
