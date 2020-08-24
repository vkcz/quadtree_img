use std::collections::HashMap;

pub type Color = image::Rgba<u8>;
type BigColor = image::Rgba<isize>;

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

fn abs_sub(a: u8, b: u8) -> u8 {
	(a as i16 - b as i16).abs() as u8
}

fn vec4_len_squared(a: u8, b: u8, c: u8, d: u8) -> u32 {
	(a as u32 * a as u32) +
	(b as u32 * b as u32) +
	(c as u32 * c as u32) +
	(d as u32 * d as u32)
}

fn color_distance(a: &Color, b: &Color) -> u32 {
	vec4_len_squared(
		abs_sub(a.0[0], b.0[0]),
		abs_sub(a.0[1], b.0[1]),
		abs_sub(a.0[2], b.0[2]),
		abs_sub(a.0[3], b.0[3]),
	)
}

fn dedup_distance(a: &Color, b: &Color) -> u32 {
	vec4_len_squared(
		abs_sub(a.0[0], b.0[0]),
		abs_sub(a.0[1], b.0[1]),
		abs_sub(a.0[2], b.0[2]),
		abs_sub(a.0[3], b.0[3]) / 4, // alpha is less important
	)
}

fn color_add_big(a: BigColor, b: BigColor) -> BigColor {
	image::Rgba::<isize>([
		a.0[0] + b.0[0],
		a.0[1] + b.0[1],
		a.0[2] + b.0[2],
		a.0[3] + b.0[3],
	])
}

fn color_mul(a: &Color, b: &isize) -> BigColor {
	image::Rgba::<isize>([
		a.0[0] as isize * b,
		a.0[1] as isize * b,
		a.0[2] as isize * b,
		a.0[3] as isize * b,
	])
}

fn color_div(a: BigColor, b: isize) -> Color {
	image::Rgba::<u8>([
		(a.0[0] / b) as u8,
		(a.0[1] / b) as u8,
		(a.0[2] / b) as u8,
		(a.0[3] / b) as u8,
	])
}

/// Selects a palette of a given size and type through a process similar to
/// (but not quite the same as) finding the most commonly used colors in the image.
///
/// `dedup_thresh` indicates the (squared) limit for how "distant" colors can be
/// while still being quantized as one color.
pub fn generate_palette<P: DynamicPalette>(
	img: &image::RgbaImage,
	dedup_thresh: u32
) -> P {
	let mut successes = HashMap::new();
	for pixel in img.pixels() {
		*successes.entry(*pixel).or_insert(0isize) += 1;
	}
	let mut similars: Vec<Vec<(Color, isize)>> = Vec::new();
	for (col, count) in successes.into_iter() {
		let mut found = false;
		for comp in similars.iter_mut() {
			if dedup_distance(&comp[0].0, &col) < dedup_thresh {
				comp.push((col, count));
				found = true;
				break;
			}
		}
		if !found {
			similars.push(vec![(col, count)]);
		}
	}
	let mut rank = Vec::new();
	rank.extend(similars.into_iter().map(|cat| {
		let total = cat.iter().map(|cc| cc.1).sum();
		let col = color_div(
			cat.iter()
				.map(|cc| color_mul(&cc.0, &cc.1))
				.fold(image::Rgba::<isize>([0; 4]), color_add_big),
			total
		);
		(col, total)
	}));
	rank.sort_by_key(|cc: &(Color, isize)| -cc.1);
	P::from(rank.iter().map(|x| x.0).collect())
}

/// Processes an image given a palette so as to convert it to a "rectangle"
/// of pixels each represented by a palette-color-number that most closely
/// matches the original color.
///
/// For the efficiency of the quadtree, the image may be Gaussian-blurred
/// before quantization; the extent to which this is done is controlled by `blur`.
pub fn quantize_to_palette<P: Palette>(
	img: &image::RgbaImage,
	palette: &P
) -> Vec<u32> {
	let palette_colors = palette.get_slice().map(|x| x.to_owned())
		.unwrap_or_else(|| (0..1 << palette.width())
			.map(|n| palette.to_rgba(n as u32).unwrap())
			.collect::<Vec<_>>());
	let mut quant_cache = HashMap::new();
	img.pixels()
		.map(|pix| {
			match quant_cache.get(pix) {
				Some(c) => *c,
				None => {
					let c = palette_colors.iter()
						.enumerate()
						.map(|(ind, col)| (color_distance(pix, col), ind as u32))
						.min().unwrap().1;
					quant_cache.insert(pix, c);
					c
				}
			}
		})
		.collect::<Vec<_>>()
}
