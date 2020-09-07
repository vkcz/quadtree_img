pub type Color = image::Rgba<u8>;

/// Trait for types that describe how to convert from an arbitrary number
/// of a fixed size to four bytes of RGBA.
pub trait Palette: Default {
	/// The bit width of each palette color's number.
	///
	/// Must be `1 <= WIDTH <= 32`, because 0 bits wouldn't really be a palette
	/// and more than 32 bits would be more efficiently represented
	/// as direct RGBA.
	fn width(&self) -> u8;
	/// Uses an instance of the implementing type to convert a number
	/// representing a palette entry into an RGBA value.
	///
	/// If `c` is outside the range of the palette, an `Err` should
	/// be returned.
	fn to_rgba(&self, c: u32) -> Result<Color, ()>;
	/// Returns a reference to the slice listing the colors in the palette,
	/// only if that is applicable and possible given the way the colors
	/// are stored.
	fn get_slice(&self) -> Option<&[Color]>;
}

/// Marker trait for `Palette` implementors that can be made from lists of
/// dynamic length (`Vec`s, that is).
pub trait DynamicPalette: Palette + From<Vec<Color>> {}

/// Used internally to assist `generic_palette_struct`.
macro_rules! generic_palette_doc {
	($e:expr) => { concat!("A simple implementer of `Palette`; ", $e, " bits.") };
}

/// Used internally to generate structs that implement  `Palette`.
macro_rules! generic_palette_struct {
	(@inner $i:ident $n:expr, $e:expr) => {
		#[doc = $e]
		#[derive(Debug)]
		pub struct $i {
			pub colors: [Color; 1 << $n],
		}
		impl Palette for $i {
			fn width(&self) -> u8 { $n }
			fn to_rgba(&self, c: u32) -> Result<Color, ()> {
				self.colors.get(c as usize).ok_or(()).map(|x| *x)
			}
			fn get_slice(&self) -> Option<&[Color]> {
				Some(&self.colors)
			}
		}
		impl Default for $i {
			fn default() -> Self {
				Self { colors: [image::Rgba([0; 4]); 1 << $n] }
			}
		}
	};
	($i:ident $n:expr, $e:expr) => {
		generic_palette_struct!(@inner $i $n, generic_palette_doc![$e]);
	};
}

generic_palette_struct!(GenericPalette1 1, "one");
generic_palette_struct!(GenericPalette2 2, "two");
generic_palette_struct!(GenericPalette3 3, "three");
generic_palette_struct!(GenericPalette4 4, "four");
generic_palette_struct!(GenericPalette5 5, "five");

/// Used internally to generate a different sort of struct that implements `Palette`.
macro_rules! palette_view_struct {
	(@inner $i:ident $n:expr, $e:expr) => {
		#[doc = $e]
		#[derive(Debug)]
		pub struct $i {
			pub colors: Box<[Color]>,
		}
		impl Palette for $i {
			fn width(&self) -> u8 { $n }
			fn to_rgba(&self, c: u32) -> Result<Color, ()> {
				if c > 1 << $n {
					Err(())
				} else {
					Ok(*(self.colors.get(c as usize).unwrap_or(&image::Rgba([0; 4]))))
				}
			}
			fn get_slice(&self) -> Option<&[Color]> {
				if self.colors.len() >= 1 << $n {
					Some(&self.colors[..1 << $n])
				} else {
					None
				}
			}
		}
		impl Default for $i {
			fn default() -> Self {
				Self { colors: Box::new([image::Rgba([0; 4]); 0]) }
			}
		}
		impl From<Vec<Color>> for $i {
			fn from(inp: Vec<Color>) -> Self {
				Self { colors: inp.into_boxed_slice() }
			}
		}
	};
	($i:ident $n:expr, $e:expr) => {
		palette_view_struct!(@inner $i $n, concat!(
			"A view into a slice for implementing `Palette` with a width of ",
		$e));
	};
}

palette_view_struct!(PaletteView1 1, "one");
palette_view_struct!(PaletteView2 2, "two");
palette_view_struct!(PaletteView3 3, "three");
palette_view_struct!(PaletteView4 4, "four");
palette_view_struct!(PaletteView5 5, "five");
palette_view_struct!(PaletteView6 6, "six");
palette_view_struct!(PaletteView7 7, "seven");
palette_view_struct!(PaletteView8 8, "eight");

/// A list of colors forming a palette, of a width determined at runtime.
#[derive(Debug)]
pub struct DynamicPaletteView {
	pub colors: Box<[Color]>
}

impl Palette for DynamicPaletteView {
	fn width(&self) -> u8 {
		(31 - (self.colors.len() as u32).leading_zeros()) as u8
	}
	fn to_rgba(&self, c: u32) -> Result<Color, ()> {
		Ok(*(self.colors.get(c as usize).unwrap_or(&image::Rgba([0; 4]))))
	}
	fn get_slice(&self) -> Option<&[Color]> {
		Some(&self.colors[..1 << self.width()])
	}
}

impl Default for DynamicPaletteView {
	fn default() -> Self {
		DynamicPaletteView { colors: Default::default() }
	}
}

impl From<Vec<Color>> for DynamicPaletteView {
	fn from(v: Vec<Color>) -> Self {
		DynamicPaletteView { colors: v.into_boxed_slice() }
	}
}

impl DynamicPalette for DynamicPaletteView {}