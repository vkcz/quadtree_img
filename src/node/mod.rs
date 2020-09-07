pub mod error;
pub mod quantize;

/// Node in a quadtree for storing an image.
///
/// May contain subnodes (branch node) or no subnodes and just a color
/// (leaf node).
///
/// It must always contain a color, such that tree descent
/// can stop at any level and give a meaningful preview, among other
/// possible reasons.
#[derive(Clone, Debug, Default)]
pub struct QuadtreeNode<P: quantize::palette::Palette + Default> {
	pub color: u32,
	pub sections: Option<Box<[QuadtreeNode<P>; 4]>>,
	_pal: std::marker::PhantomData<P>
}

impl<P: quantize::palette::Palette + Default> QuadtreeNode<P> {
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
	///
	/// `gradient` has a similar meaning as it does for `from_image`.
	pub fn mount(
		&mut self,
		image: &[u32],
		palette: &P,
		size: Option<usize>,
		start_pos: Option<(usize, usize)>,
		sensitivity: usize,
		gradient: bool
	) -> Result<(), error::MountError> {
		if !image.len().is_power_of_two() || image.len().trailing_zeros() % 2 == 1 {
			return Err(error::MountError::InvalidSize);
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
				*h.entry(n).or_insert(0) += 1isize;
				h
			});
		let mut abundance_sort = abundance_map.iter()
			.map(|e| (-e.1, e.0))
			.collect::<Vec<_>>();
		abundance_sort.sort();
		let abundance_res = abundance_sort[0];
		self.color = **abundance_res.1;
		// Validate color. This should be validated for every pixel, but
		// due to recursion that goes down through every pixel, it will be handled.
		if self.color > 1 << palette.width() {
			return Err(error::MountError::ColorOutOfRange);
		}
		// Recursion
		if size > 1 && (-abundance_res.0 as usize) < (sensitivity * size * size) / 16384 {
			self.sections = Some(Default::default());
			let abundance_four = abundance_sort.iter().chain(std::iter::repeat(&(0, &&0)).take(4)).take(4);
			if gradient && size > 2 && abundance_four.map(|x| if -x.0 as usize > (sensitivity * size * size) / 65536
					{ -x.0 as usize } else { 0 }).sum::<usize>() > (sensitivity * size * size) / 16384 {
				for sect_ind in 0..4 {
					let off = size / 4;
					let x_off = (sect_ind & 1) * 6 * off / 2;
					let y_off = (sect_ind & 2) * 3 * off / 2;
					let mut abundance_sort = ((start_pos.1 + y_off)..(start_pos.1 + y_off + off)).flat_map(|row| image[
						(row * row_len + start_pos.0 + x_off)..(row * row_len + start_pos.0 + x_off + off)
						].iter())
						.fold(std::collections::HashMap::new(), |mut h, n| {
							*h.entry(n).or_insert(0) += 1;
							h
						})
						.into_iter()
						.map(|e| (-e.1, e.0))
						.collect::<Vec<_>>();
					abundance_sort.sort();
					self.sections.as_mut().unwrap()[sect_ind].color = *abundance_sort[0].1;
				}
			} else {
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
							sensitivity,
							gradient
						)?;
				}
			}
		}
		Ok(())
	}
}

pub mod image;
pub mod qti;