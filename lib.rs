pub mod quantize;

use bitvec::vec::BitVec;

use quantize::{Palette, DynamicPalette};

/// A `BitVec` variant ideal for encoding and decoding quadtrees.
type QuadtreeEncodeBitVec = BitVec<bitvec::order::Msb0, u8>;

/// Node in a quadtree for storing an image.
///
/// May contain subnodes (branch node) or no subnodes and just a color
/// (leaf node).
///
/// It must always contain a color, such that tree descent
/// can stop at any level and give a meaningful preview, among other
/// possible reasons.
#[derive(Clone, Debug, Default)]
pub struct QuadtreeNode<P: Palette + Default> {
	pub color: u32,
	pub sections: Option<[Box<QuadtreeNode<P>>; 4]>,
	_pal: std::marker::PhantomData<P>
}

/// Reason why a quadtree couldn't be rendered to an image buffer.
#[derive(Debug)]
pub enum DrawError {
	/// The image buffer's dimensions are not equal; the image is not a square.
	NonSquare,
	/// The image buffer's dimensions are not powers of two.
	NonPowerOfTwo,
	/// A color specified in the quadtree is outside the range of the palette.
	ColorOutOfRange,
}

/// Reason why an image couldn't be turned into a quadtree.
#[derive(Debug)]
pub enum AnalyzeError {
	/// The image buffer's dimensions are not equal; the image is not a square.
	NonSquare,
	/// The image buffer's dimensions are not powers of two.
	NonPowerOfTwo,
}

/// Reason why a quadtree couldn't be encoded.
#[derive(Debug)]
pub enum EncodeError {
	/// A color specified in the quadtree is outside the range of the palette.
	ColorOutOfRange,
}

/// Reason why a quadtree encoding couldn't be decoded.
#[derive(Debug)]
pub enum DecodeError {
	/// A node number was exepcted but not found.
	InsufficientData,
	/// There was no valid QTI file header.
	MissingHeader,
	/// `GenericPalette` could not stored a palette of the necessary size.
	PaletteTooLarge,
}

/// Reason why an "image" of palette colors couldn't be made into a quadtree.
#[derive(Debug)]
pub enum MountError {
	/// The size of the "image" buffer is not a power of 4.
	InvalidSize,
	/// A pixel has a color outside the extent of the palette.
	ColorOutOfRange,
}

impl<P: Palette + Default> QuadtreeNode<P> {
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
	pub fn to_image(
		&self,
		img: &mut image::RgbaImage,
		palette: &P,
		size: Option<u32>,
		start_pos: Option<(u32, u32)>
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
					)?;
				}
			}
		}

		Ok(())
	}

	/// Analyzes a traditional image into a quadtree, "rounding" pixel colors
	/// to the nearest entries in the palette.
	///
	/// See documentation on `mount` for the meaning of `sensitivity`.
	pub fn from_image(
		&mut self,
		img: &image::RgbaImage,
		palette: &P,
		sensitivity: usize
	) -> Result<(), AnalyzeError> {
		// Validate image size
		if img.width() != img.height() {
			return Err(AnalyzeError::NonSquare);
		}
		if !img.width().is_power_of_two() {
			return Err(AnalyzeError::NonPowerOfTwo);
		}

		let palettified = quantize::quantize_to_palette(img, palette, 0.);
		match self.mount(&palettified, palette, None, None, sensitivity) {
			Ok(_) => (),
			Err(_) => unreachable!("error in mounting")
		}
		Ok(())
	}

	/// Takes a "square" of color numbers to match the given palette
	/// and arranges it into an efficient quadtree.
	///
	/// The "square" must be a `Vec<u32>` with the length being a power of 4.
	/// This is because powers of 4 are squares of powers of 2.
	///
	/// `sensitivity` should be a number from 0 to 16384 indicating how much
	/// of a square (fraction out of 16384) must be the same color in order
	/// to disregard subesctions. If you don't know what you're doing, pass
	/// 16384.
	///
	/// For outside callers: leave `size` and `start_pos` as `None`.
	pub fn mount(
		&mut self,
		image: &[u32],
		palette: &P,
		size: Option<usize>,
		start_pos: Option<(usize, usize)>,
		sensitivity: usize
	) -> Result<(), MountError> {
		if !image.len().is_power_of_two() || image.len().trailing_zeros() % 2 == 1 {
			return Err(MountError::InvalidSize);
		}
		// Square root
		let row_len = image.len() >> (image.len().trailing_zeros() >> 1);
		// Find most common color in corresponding section.
		let size = size.unwrap_or(row_len);
		let start_pos = start_pos.unwrap_or((0, 0));
		let abundance_map = (start_pos.1..start_pos.1 + size).flat_map(|row| image[
			(row * row_len + start_pos.0)..(row * row_len + start_pos.0 + size)
			].iter())
			.fold(std::collections::HashMap::new(), |mut h, n| {
				*h.entry(n).or_insert(0) += 1;
				h
			});
		let abundance_res = abundance_map.iter()
			.map(|e| (e.1, e.0))
			.max()
			.unwrap();
		self.color = **abundance_res.1;
		// Validate color. This should be validated for every pixel, but
		// due to recursion that goes down through every pixel, it will be handled.
		if self.color > 1 << P::WIDTH {
			return Err(MountError::ColorOutOfRange);
		}
		// Recursion
		if size > 1 && *abundance_res.0 < (sensitivity * size * size) / 16384 {
			self.sections = Some([ // refactor?
				Box::new(Default::default()),
				Box::new(Default::default()),
				Box::new(Default::default()),
				Box::new(Default::default()),
			]);
			for sect_ind in 0..4 {
				self.sections.as_mut().unwrap()[sect_ind]
					.mount(
						image,
						palette,
						Some(size / 2),
						Some((
							start_pos.0 + (sect_ind & 1) * (size / 2),
							start_pos.1 + (sect_ind >> 1) * (size / 2),
						)),
						sensitivity
					)?;
			}
		}
		Ok(())
	}

	/// Converts the `QuadtreeNode` into a binary data format.
	///
	/// Takes the bit width of the palette and converts each node into a
	/// palette color, plus an extra bit at the start to indicate containing
	/// subsections; each node's number will be immediately followed by the
	/// numbers for its subsections.
	///
	/// Palette color numbers are bitwise big-endian.
	pub fn encode(
		&self,
		buffer: &mut QuadtreeEncodeBitVec,
		palette: &P
	) -> Result<(), EncodeError> {
		// Validate color value
		if self.color >= 1 << P::WIDTH {
			return Err(EncodeError::ColorOutOfRange);
		}
		// Bit to indicate subsections
		buffer.push(self.sections.is_some());
		// Color number
		for bit_ind in 0..P::WIDTH {
			buffer.push(self.color & (1 << (P::WIDTH - bit_ind - 1)) != 0);
		}
		// Recursion
		if let Some(ref sects) = self.sections {
			for section in sects.iter() {
				section.encode(buffer, palette)?;
			}
		}
		Ok(())
	}

	/// Reads a `BitVec` of the sort that would be output from `.encode()`
	/// and parses a quadtree from it.
	///
	/// Successful return value is the index to which the parser has progressed,
	/// to assist with the recursive algorithm.
	///
	/// 0 should be passed for `curr_ind` by outside callers, unless they
	/// know what they're doing and have a good reason otherwise.
	pub fn decode(
		&mut self,
		buffer: &QuadtreeEncodeBitVec,
		palette: &P,
		mut curr_ind: usize
	) -> Result<usize, DecodeError> {
		// Validate data quantity
		if buffer.len() - curr_ind < P::WIDTH as usize {
			return Err(DecodeError::InsufficientData);
		}
		// Extract current node
		let mut n = 0;
		for bit_ind in 0..P::WIDTH {
			n |= (buffer[curr_ind + bit_ind as usize + 1] as u32) << (P::WIDTH - bit_ind - 1);
		}
		self.color = n;
		// Recursion
		let should_recurse = buffer[curr_ind];
		curr_ind += 1 + P::WIDTH as usize;
		if should_recurse {
			self.sections = Some([ // refactor?
				Box::new(Default::default()),
				Box::new(Default::default()),
				Box::new(Default::default()),
				Box::new(Default::default()),
			]);
			for sect_ind in 0..4 {
				curr_ind = self.sections.as_mut().unwrap()[sect_ind]
					.decode(buffer, palette, curr_ind)?;
			}
		}
		Ok(curr_ind)
	}

	/// "Trims" the tree by removing leaf nodes.
	///
	/// Only leaf nodes past a depth of `depth` will be removed.
	pub fn trim(&mut self, depth: isize) {
		if let Some(sections) = &mut self.sections {
			if depth <= 0 && sections.iter().all(|s| s.sections.is_none()) {
				self.sections = None;
			} else {
				sections.iter_mut().for_each(|s| s.trim(depth - 1));
			}
		}
	}

	/// Encodes the quadtree and a palette into QTI data.
	pub fn to_qti(&self, palette: &P) -> Result<Vec<u8>, EncodeError> {
		let mut ret = Vec::new();
		// Header (version 1)
		ret.extend_from_slice(b"QuTrIm\x01");
		let mut palette_vec = palette.get_slice()
			.map(|x| x.to_owned())
			.unwrap_or_else(|| (0..P::WIDTH << 1)
				.map(|n| palette.to_rgba(n as u32).unwrap())
				.collect::<Vec<_>>());
		palette_vec.resize(1 << P::WIDTH, image::Rgba([0; 4]));
		let palette_len = std::cmp::max((1 << P::WIDTH) - palette_vec.iter()
			.rev()
			.take_while(|c| **c == image::Rgba([0; 4]))
			.count(),
			(9 * (1 << P::WIDTH) + 15) / 16);
		let approx_len = (palette_len as f64 * 16. / (1 << P::WIDTH) as f64)
			.ceil() as u32 * (1 << P::WIDTH) / 16;
		// Length indicator
		ret.push((((approx_len * 16) / (1 << P::WIDTH) - 9) << 5) as u8 |
			(P::WIDTH - 1));
		// Palette
		for c in 0..approx_len {
			ret.extend_from_slice(&palette.to_rgba(c).unwrap().0);
		}
		// Quadtree
		let mut bit_buf = QuadtreeEncodeBitVec::new();
		self.encode(&mut bit_buf, palette)?;
		ret.extend_from_slice(bit_buf.as_slice());
		Ok(ret)
	}
}

impl<'a, P: DynamicPalette + Default> QuadtreeNode<P> {
	/// Derives a palette and quadtree from the data of a QTI file.
	pub fn from_qti(source: &[u8]) -> Result<(QuadtreeNode<P>, P), DecodeError> {
		// Verify header (version 1 is required for compatibility)
		if &source[..7] != b"QuTrIm\x01" {
			return Err(DecodeError::MissingHeader);
		}
		let pal_size = (source[7] & 0x1f) + 1;
		let pal_len = (
			((source[7] >> 5) as f64 + 9.) *
			(pal_size as f64 - 4.).exp2()
		) as u32;
		assert!(pal_len.count_ones() <= 4);
		// Base two logarithm
		if pal_size > P::WIDTH {
			return Err(DecodeError::PaletteTooLarge);
		};
		// Extract palette
		let mut pal = vec![];
		for offset in (0..pal_len).map(|n| n as usize * 4 + 8) {
			pal.push(image::Rgba([
				source[offset],
				source[offset + 1],
				source[offset + 2],
				source[offset + 3],
			]));
		}
		let palette = P::from(pal);
		// Decode tree
		let tree_bits = QuadtreeEncodeBitVec::from(source);
		let mut tree: QuadtreeNode<P> = Default::default();
		tree.decode(&tree_bits, &palette, 64 + 32 * pal_len as usize)?;
		Ok((tree, palette))
	}
}