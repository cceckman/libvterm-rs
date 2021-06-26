# TODO

* [x] validate Pos and Rect arguments are withing the right range, or
      segfaults happen! Like in `get_text_as_bytes`
* [ ] benchmark and performance improvements
* [x] take references to things when its more idiomatic
* [x] implement Write trait
* [x] replace u16 and i16 with usize
* [x] remove positions on screen cells
* [x] -try out the bitflags crate-
* [ ] use libvterm palette api instead of what I rolled on my own
* [ ] upgrade libvterm?
      * `libvterm-0.1.4` changes the color API, from RGB to tagged-union
        (RGB-or-palette).
      * Consider using `bindgen` instead of hand-rolled FFI.
* [x] rethink representing cell data as char vs Vec<u8> or [u8] or whatever.
* [ ] add methods to ffi datatypes to convert from that and rust
* [x] use geometry library from crates.io
