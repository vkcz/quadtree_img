pub mod node;

pub use node::*;

use quantize::palette::{Palette};

use std::collections::HashMap;

impl<P: Palette + Default> node::QuadtreeNode<P> {

	/// "Trims" the tree by removing leaf nodes.
	///
	/// Only leaf nodes past a depth of `depth` and with color repetition
	/// will be removed.
	pub fn trim(&mut self, depth: isize) {
		if let Some(sections) = &mut self.sections {
			if depth <= 0 && sections.iter().all(|s| s.sections.is_none()) {
				// Count unique colors
				let col_f = sections.iter().fold(HashMap::new(),
					|mut m, e| { *m.entry(e.color).or_insert(0) += 1; m });
				let freq = col_f.values().collect::<Vec<_>>();
				if freq.len() == 3 || (freq.len() == 2 && **freq.iter().max().unwrap() == 3) {
					self.sections = None;
				}
			} else {
				sections.iter_mut().for_each(|s| s.trim(depth - 1));
			}
		}
	}
}