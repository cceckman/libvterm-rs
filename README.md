This library provides rust bindings for libvterm:
http://www.leonerd.org.uk/code/libvterm/

libvterm is "An abstract library implementation of a VT220/xterm/ECMA-48
terminal emulator"

# How to build

Just run

    cargo build

# How to turn on libvterm debugging

Use CFLAGS env var to define DEBUG with is used in `vterm_internal.h` to
control logging to stderr:

    CFLAGS=-DDEBUG cargo test
