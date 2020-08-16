# `quadtree_img`

A quadtree-based image compression system and reference implementation.

Written in Rust; licensed with AGPL3.

## Files

`qti_spec.md` documents the new QTI file format used in this project.

`src/lib.rs` is the main library source module with the submodule dependency from `src/quantize.rs`.

`src/main.rs` is the source for a CLI tool using the `quadtree_img` library here for converting between PNG (or JFIF) and QTI.

`cargo run` in the project root will run this CLI tool in `src/main.rs`.

As of this writing, the code has no `unsafe`, no warnings, and no `cargo clippy` issues.

## Lossiness

**Quadtrees on their own are lossless,** however, most images have slight color noise or gradients, making quadtree compression highly inefficient, so to improve the efficiency
of compression, this software quantizes images (among other techniques), which make the compression lossy.

If the input images have a limited set of colors, and all areas of approximately the same color are truly *the same color*, the compression will be (in theory) lossless.

The amount of perceptible loss comes from similar aspects of images as that which causes images to be inefficiently compressed as QTI; that is, images that cannot be very much
compressed as QTI will often be quite lossy, while images that are efficiently compressed when stored as QTI will often be close to the original.

## Compression

`TODO`

## Performance

`TODO`
