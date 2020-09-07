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