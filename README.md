# How to build

    git submodule init
    git submodule update
    (cd vendor/libvterm; make)
    cargo build

The make part in libvterm runs a perl script to transform some files in
libvterm/src/encoding.

# How to turn on libvterm debugging

Use CFLAGS env var to define DEBUG with is used in `vterm_internal.h` to
control logging to stderr:

    CFLAGS=-DDEBUG cargo test
