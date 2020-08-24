# `quadtree_img`

A quadtree-based image compression system and reference implementation.

Written in Rust; licensed with AGPL3.

Issues and PRs very much welcome. Note that versions before `738b48f` do not have a correct `Cargo.lock` and will not compile without modification; unless you're interested in
the history of this project, there's no reason to be using any versions before that anyway.

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

Quadtree-based compression is dependent on the idea that most images have significant areas of the same color (or very similar color, which is quantized to the same color).
Areas that are filled with the same color are represented by large squares, and edges between areas or within more chaotic areas are divided into smaller squares to preserve
detail.

The compression of an image in QTI compared to a moderate-efficiency PNG is typically 1.5x to 4x (but can be more extreme), depending on the input image and compression
parameters. Usually, the compression factor derived from the parameters is roughly inversely proportional to the quality of the resulting compressed image.

Applying [`oxipng`](https://github.com/shssoichiro/oxipng) or a comparable PNG optimizer to the relevant images will reduce the relevant efficiency of QTI (by increasing the
efficiency of PNG). QTI, however, unlike PNG, can be made more efficient by running it through a generic lossless compression algorithm such as `xz`.

Images with large areas of the same color (such as cartoons) will be compressed efficiently as QTI, while noisier and more complex images (such as photos) would not get
significant efficiency gains from QTI, except with certain combinations of parameters that would result in a major decrease in quality.

## Performance

Applying `quadtree_img` to compress a 2048x2048 6.6 MB PNG photo into a 2.3 MB QTI takes 7.5 to 11.5 seconds, and decompressing the resultant QTI back to a slightly lossy
2048x2048 4.5 MB PNG takes 1.6 to 2.2 seconds. The tests giving these results were performed on a 3.2 GHz x86_64 processor with 4GB RAM.
