use image::error::ImageError;

use quadtree_img::QuadtreeNode;

use std::fs::File;

use std::io::{Read, Write};

/// Helper function for `main`.
fn error_exit(msg: &str, code: i32) -> ! {
	eprintln!("{}", msg);
	std::process::exit(code)
}

/// `clap`-based CLI for working with QTI files.
///
/// May exit process with status code if there are errors:
///
/// 1: `clap` error
///
/// 2: invalid arguments
///
/// 3: file I/O issues
///
/// 4: invalid image data
///
/// 5: computation limits exceeded
///
/// 10: other, potentially unknown error
fn main() {
	let clap_matches = clap::App::new("quadtree_img")
		.version("0.1.0")
		.author("|x| Some(x[0]) <undisclosed@example.com>")
		.about("Converts to and from a quadtree-based image compression format (QTI).")
		.arg_from_usage("-i, --into 'Convert the input file from PNG or JFIF to QTI'")
		.arg_from_usage("-f, --from 'Convert the input file from QTI to PNG'")
		.arg_from_usage("<INPUT> 'Path to input file`")
		.arg_from_usage("[OUTPUT] 'Path to output file; defaults to INPUT with a modified file extension`")
		.get_matches();

	let (into, from) = (clap_matches.is_present("into"), clap_matches.is_present("from"));
	match (into, from) {
		(true, true) => error_exit("Only one of -i/--into and -f/--from must be present", 2),
		(true, false) => {
			let input_path = clap_matches.value_of("INPUT").unwrap();
			let source = match image::open(input_path) {
				Ok(i) => i,
				Err(e) => {
					let (msg, code) = match e {
						ImageError::Decoding(_) => ("Invalid image data", 4),
						ImageError::Limits(_) => ("Computation limits exceeded", 5),
						ImageError::IoError(_) => ("File not found or could not be read", 3),
						_ => ("An error occurred", 10)
					};
					error_exit(msg, code)
				}
			}.into_rgba();
			// TODO: Allow configuration of palette generation
			let palette = quadtree_img::quantize::generate_palette::
				<quadtree_img::quantize::PaletteView5>(&source, 512);
			let mut tree: QuadtreeNode<_> = Default::default();
			// TODO: Allow configuration of sensitivity
			match tree.from_image(&source, &palette, 15872) {
				Ok(()) => (),
				// TODO: Add support for non-square/non-power-of-two images
				Err(_) => error_exit("Input image has invalid dimensions", 4)
			}
			// `.expect()` is valid here, because the only error that can occur here
			// is a color in the quadtree out of range of the palette, but since the
			// quadtree is generated programmatically from an image, that should not
			// happen. If it does happen, there is a bug in the program to be fixed.
			let qti_data = tree.to_qti(&palette).expect("failure to serialize to QTI");
			let mut out_fh = match File::create(clap_matches.value_of("OUTPUT")
				.unwrap_or(&(input_path.rsplitn(2, '.').last().unwrap().to_string() + ".qti"))) {
				Ok(f) => f,
				Err(_) => error_exit("Could not open output file", 3)
			};
			match out_fh.write_all(&qti_data) {
				Ok(_) => (),
				Err(_) => error_exit("Could not write to output file", 3)
			}
		},
		(false, true) => {
			let input_path = clap_matches.value_of("INPUT").unwrap();
			let mut source_data = Vec::new();
			let mut source_fh = match File::open(input_path) {
				Ok(f) => f,
				Err(_) => error_exit("File not found or could not be read", 3)
			};
			match source_fh.read_to_end(&mut source_data) {
				Ok(_) => (),
				Err(_) => error_exit("Could not read from input file", 3)
			}
			let (tree, palette): (_, quadtree_img::quantize::PaletteView5) =
				match QuadtreeNode::from_qti(&source_data) {
				Ok((t, p)) => (t, p),
				Err(_) => error_exit("Invalid image data", 4)
			};
			// TODO: Allow configuration of output size
			let mut output = image::RgbaImage::new(512, 512);
			match tree.to_image(&mut output, &palette, None, None) {
				Ok(_) => (),
				Err(e) => {
					let (msg, code) = match e {
						quadtree_img::DrawError::NonSquare |
						quadtree_img::DrawError::NonPowerOfTwo => ("Invalid output dimensions", 2),
						quadtree_img::DrawError::ColorOutOfRange => ("Invalid image data", 4)
					};
					error_exit(msg, code)
				}
			}
			match output.save(clap_matches.value_of("OUTPUT")
				.unwrap_or(&(input_path.rsplitn(2, '.').last().unwrap().to_string() + ".png"))) {
				Ok(_) => (),
				Err(_) => error_exit("Could not save output", 3)
			}
		},
		(false, false) => error_exit("One of -i/--into and -f/--from must be present", 2)
	}
}
