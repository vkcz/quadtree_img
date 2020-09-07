use bitvec::vec::BitVec;

use super::error::*;
use super::quantize::palette::{DynamicPalette, Palette};

/// A `BitVec` variant ideal for encoding and decoding quadtrees.
type QuadtreeEncodeBitVec = BitVec<bitvec::order::Msb0, u8>;

/// A type for doing things
type DecodeQueue = Vec<(Vec<(bool, u32)>, usize)>;

impl<P: Palette + Default> super::QuadtreeNode<P> {
	/// Converts the `QuadtreeNode` into a binary data format.
	///
	/// Takes the bit width of the palette and converts each node into a
	/// palette color, plus an extra bit at the start to indicate containing
	/// subsections; each node's number will be immediately followed by the
	/// numbers for its subsections.
	///
	/// Palette color numbers are bitwise big-endian.
	pub fn encode_v1(
		&self,
		buffer: &mut QuadtreeEncodeBitVec,
		palette: &P
	) -> Result<(), EncodeError> {
		// Validate color value
		if self.color >= 1 << palette.width() {
			return Err(EncodeError::ColorOutOfRange);
		}
		// Bit to indicate subsections
		buffer.push(self.sections.is_some());
		// Color number
		for bit_ind in 0..palette.width() {
			buffer.push(self.color & (1 << (palette.width() - bit_ind - 1)) != 0);
		}
		// Recursion
		if let Some(ref sects) = self.sections {
			for section in sects.iter() {
				section.encode_v1(buffer, palette)?;
			}
		}
		Ok(())
	}

	/// Reads a `BitVec` of the sort that would be output from `.encode_v1()`
	/// and parses a quadtree from it.
	///
	/// Successful return value is the index to which the parser has progressed,
	/// to assist with the recursive algorithm.
	///
	/// 0 should be passed for `curr_ind` by outside callers, unless they
	/// know what they're doing and have a good reason otherwise.
	pub fn decode_v1(
		&mut self,
		buffer: &QuadtreeEncodeBitVec,
		palette: &P,
		mut curr_ind: usize
	) -> Result<usize, DecodeError> {
		// Validate data quantity
		if buffer.len() - curr_ind < (palette.width()) as usize {
			return Err(DecodeError::InsufficientData);
		}
		// Extract current node
		let mut n = 0;
		for bit_ind in 0..(palette.width()) {
			n |= (buffer[curr_ind + bit_ind as usize + 1] as u32) << (palette.width() - bit_ind - 1);
		}
		self.color = n;
		// Recursion
		let should_recurse = buffer[curr_ind];
		curr_ind += 1 + palette.width() as usize;
		if should_recurse {
			self.sections = Some(Default::default());
			for sect_ind in 0..4 {
				curr_ind = self.sections.as_mut().unwrap()[sect_ind]
					.decode_v1(buffer, palette, curr_ind)?;
			}
		}
		Ok(curr_ind)
	}

	/// Reads a `BitVec` of the sort that would be output from `.encode_v2()`
	/// and parses a quadtree from it.
	///
	/// Not yet implemented. I have no idea what I'm doing.
	/// Big TODO.
	pub fn decode_v2(
		&mut self,
		buffer: &QuadtreeEncodeBitVec,
		palette: &P,
		queue: Option<&mut DecodeQueue>,
	) -> Result<DecodeQueue, DecodeError> {
		// To get rid of unused variable warnings
		dbg!(buffer, queue, palette.width());
		Err(DecodeError::InsufficientData)
	}

	/// Encodes the quadtree and a palette into QTI data.
	pub fn to_qti(&self, palette: &P) -> Result<Vec<u8>, EncodeError> {
		let mut ret = Vec::new();
		// Header (version 1)
		ret.extend_from_slice(b"QuTrIm\x01");
		let mut palette_vec = palette.get_slice()
			.map(|x| x.to_owned())
			.unwrap_or_else(|| (0..palette.width() << 1)
				.map(|n| palette.to_rgba(n as u32).unwrap())
				.collect::<Vec<_>>());
		palette_vec.resize(1 << palette.width(), image::Rgba([0; 4]));
		let palette_len = std::cmp::max((1 << palette.width()) - palette_vec.iter()
			.rev()
			.take_while(|c| **c == image::Rgba([0; 4]))
			.count(),
			(9 * (1 << palette.width()) + 15) / 16);
		let approx_len = (palette_len as f64 * 16. / (1 << palette.width()) as f64)
			.ceil() as u32 * (1 << palette.width()) / 16;
		// Length indicator
		ret.push((((approx_len * 16) / (1 << palette.width()) - 9) << 5) as u8 |
			(palette.width() - 1));
		// Palette
		for c in 0..approx_len {
			ret.extend_from_slice(&palette.to_rgba(c).unwrap().0);
		}
		// Quadtree
		let mut bit_buf = QuadtreeEncodeBitVec::new();
		self.encode_v1(&mut bit_buf, palette)?;
		ret.extend_from_slice(bit_buf.as_slice());
		Ok(ret)
	}
}

impl<'a, P: DynamicPalette + Default + std::fmt::Debug> super::QuadtreeNode<P> {
	/// Derives a palette and quadtree from the data of a QTI file.
	pub fn from_qti(source: &[u8]) -> Result<(super::QuadtreeNode<P>, P), DecodeError> {
		// Verify header (version 1 is required for compatibility)
		if &source[..6] != b"QuTrIm" {
			return Err(DecodeError::MissingHeader);
		}
		let pal_size = (source[7] & 0x1f) + 1;
		let pal_len = (
			((source[7] >> 5) as f64 + 9.) *
			(pal_size as f64 - 4.).exp2()
		) as u32;
		assert!(pal_len.count_ones() <= 4);
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
		pal.resize(1 << pal_size, image::Rgba([0; 4]));
		let palette = P::from(pal);
		// Decode tree
		let tree_bits = QuadtreeEncodeBitVec::from(&source[8 + 4 * pal_len as usize..]);
		let mut tree: super::QuadtreeNode<P> = Default::default();
		match source[6] {
			1 => { // Version one, documented in older versions of qti_spec
				tree.decode_v1(&tree_bits, &palette, 0)?;
				Ok((tree, palette))
			},
			2 => { // Version two (current) -- DOES NOT WORK; TODO
				tree.decode_v2(&tree_bits, &palette, None)?;
				Ok((tree, palette))
			},
			_ => Err(DecodeError::MissingHeader)
		}
	}
}