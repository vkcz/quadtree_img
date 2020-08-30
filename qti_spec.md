# QTI File Structure

## Introduction

QTI files are a new and unique file type invented for storing quadtree-based
images. They are not documented in any popular standards, and at the time of
this writing, they have only been used by one person (me, the author of this
library). Hopefully this will change a bit in the future.

QTI files have no registered MIME type and have a file extension of `.qti`.

## File header (magic bytes and such)

At the start of a QTI file, there is the "magic byte" sequence starting with the
ASCII characters `QuTrIm` (for "QuadTree Image"), followed by a byte to
represent the format version (`0x02` for this version of the document), another
byte to describe the size of the color space, and four bytes to describe whether
gradients are to be used and the size of the image.

The color-space-size byte is split into two numbers; one from the upper three
bits and one from the lower five bits. The number of bits to represent each
color in the image from the palette is equal to the five-bit lower number plus
one, `b` (1 to 32 inclusive); the number of colors actually specified in the
palette is equal to the three-bit upper number plus nine, `n`, times `2^(b - 4)`
(where `^` represents exponentiation, not XOR). `c = n * 2 ^ (b - 4)`

The last four bytes of the header start with the most-significant-bit of the
first byte representing a boolean value to indicate whether or not the image
should be displayed with gradients; these four bytes (with the gradient bit
"removed") specify, as two big-endian two-byte unsigned integers, the dimensions
(height, then width) of the image. (The current implementation only supports
images with the same width and height, which must both be powers of two.)

## Color palette segment

After these first twelve bytes of header content, there is a color palette
specified as 32-bit RGBA (8 bits per channel). There are four bytes for each of
`c` colors, to match the palette size specified in the last byte of the header.

## Quadtree content

After the header and palette, a quadtree will be serialized in a bitwise manner
independent of byte boundaries; each quadtree node will be represented as one
bit to indicate whether or not it contains subnodes, followed by `b` bits to
indicate the color of that node.

All nodes at the same "level" of recursive depth will be encoded in a sequence
uninterrupted; the appropriate subnodes for deeper levels of recursion will come
in a sequence immediately following each immediately "shallower" level. The file
can then be rendered to an image by initializing a square with power-of-two
dimensions in the color specified from the initial node, followed by replacing
squares of half the dimension of the containing squares with the colors of
subnodes, when there are subnodes, recursively through the tree.