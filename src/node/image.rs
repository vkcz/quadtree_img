use super::error::*;
use super::quantize::palette::{Color, Palette};

fn color_lerp(a: Color, b: Color, n: f64) -> Color {
	image::Rgba::<u8>([
		(((b.0[0] as f64) - (a.0[0] as f64)) * n + a.0[0] as f64) as u8,
		(((b.0[1] as f64) - (a.0[1] as f64)) * n + a.0[1] as f64) as u8,
		(((b.0[2] as f64) - (a.0[2] as f64)) * n + a.0[2] as f64) as u8,
		(((b.0[3] as f64) - (a.0[3] as f64)) * n + a.0[3] as f64) as u8,
	])
}

impl<P: Palette + Default> super::QuadtreeNode<P> {
	/// Attempts to generate an image into the supplied buffer
	/// from this quadtree node and its "branches" and "leaves".
	///
	/// Will return an `Err` if the color in a quadtree node does not
	/// fit in the provided palette, or if the image dimensions are not
	/// square or not a power of two.
	///
	/// The `size` and `start_pos` arguments are for internal recursive
	/// use; `None` should be passed by outside callers (unless you
	/// **really** know what you're doing).
	///
	/// `gradient` indicates whether leaf nodes will be presented as
	/// solid squares of color or bilinear gradients between the leaf
	/// nodes below the relevant branch.
	pub fn to_image(
		&self,
		img: &mut image::RgbaImage,
		palette: &P,
		size: Option<u32>,
		start_pos: Option<(u32, u32)>,
		gradient: bool
	) -> Result<(), DrawError> {
		// Check input validity
		if img.width() != img.height() {
			return Err(DrawError::NonSquare);
		}
		if !img.width().is_power_of_two() ||
			!size.map(u32::is_power_of_two).unwrap_or(true) {
			return Err(DrawError::NonPowerOfTwo);
		}

		// Draw current node
		let curr_size = size.unwrap_or_else(|| img.width());
		let curr_pos = start_pos.unwrap_or((0, 0));
		match palette.to_rgba(self.color) {
			Ok(c) => image::imageops::replace(
				img,
				&image::RgbaImage::from_pixel(curr_size, curr_size, c),
				curr_pos.0,
				curr_pos.1,
			),
			Err(_) => return Err(DrawError::ColorOutOfRange),
		}

		// Recursion
		if curr_size > 1 {
			if let Some(ref sects) = self.sections {
				if gradient && sects.iter().all(|s| s.sections.is_none()) {
					for row in curr_pos.1..(curr_pos.1 + curr_size) {
						for col in curr_pos.0..(curr_pos.0 + curr_size) {
							let sect_colors = sects.iter()
								.map(|s| palette.to_rgba(s.color))
								.fold(Ok(Vec::new()), |v, n| match (v, n) {
									(Ok(mut l), Ok(c)) => { l.push(c); Ok(l) },
									_ => Err(DrawError::ColorOutOfRange)
								})?;
							let x_n = ((col - curr_pos.0) as f64) / curr_size as f64;
							let y_n = ((row - curr_pos.1) as f64) / curr_size as f64;
							let imm_c = color_lerp(
								color_lerp(sect_colors[0], sect_colors[1], x_n),
								color_lerp(sect_colors[2], sect_colors[3], x_n),
								y_n
							);
							img.put_pixel(col, row, imm_c);
						}
					}
				} else {
					let positions = [
						(curr_pos.0, curr_pos.1),
						(curr_pos.0 + curr_size / 2, curr_pos.1),
						(curr_pos.0, curr_pos.1 + curr_size / 2),
						(curr_pos.0 + curr_size / 2, curr_pos.1 + curr_size / 2),
					];
					for (ind, section) in sects.iter().enumerate() {
						section.to_image(
							img,
							palette,
							Some(curr_size / 2),
							Some(positions[ind]),
							gradient
						)?;
					}
				}
			}
		}

		Ok(())
	}

	/// Analyzes a traditional image into a quadtree, "rounding" pixel colors
	/// to the nearest entries in the palette.
	///
	/// See documentation on `mount` for the meaning of `sensitivity`.
	///
	/// `blur` is the amount of Gaussian blur to apply to the image before
	/// quadtreeifying (to remove noise).
	///
	/// `gradient` indicates whether or not to generate the quadtree in a way
	/// such that the resultant restored image will be of higher quality
	/// (in theory) if `gradient` is passed as `true` to `to_image`.
	pub fn from_image(
		&mut self,
		img: &image::RgbaImage,
		palette: &P,
		sensitivity: usize,
		blur: f32,
		gradient: bool
	) -> Result<(), AnalyzeError> {
		// Validate image size
		if img.width() != img.height() {
			return Err(AnalyzeError::NonSquare);
		}
		if !img.width().is_power_of_two() {
			return Err(AnalyzeError::NonPowerOfTwo);
		}

		let img_tr = if blur == 0. { img.to_owned() } else { image::imageops::blur(img, blur) };
		let palettified = super::quantize::quantize_to_palette(
			&img_tr,
			palette
		);
		match self.mount(&palettified, palette, None, None, sensitivity, gradient) {
			Ok(_) => (),
			Err(_) => unreachable!("error in mounting")
		}
		Ok(())
	}
}