#!/usr/bin/env bash
set -euo pipefail

# App Pro build environment setup
# Requires: /tmp/sysroot and /tmp/sysroot-lib already prepared (see PREPARE.md)

export PKG_CONFIG_PATH=/tmp/sysroot/usr/lib/x86_64-linux-gnu/pkgconfig:/tmp/sysroot/usr/share/pkgconfig:/tmp/gtk4-dev/usr/lib/x86_64-linux-gnu/pkgconfig
export PKG_CONFIG_ALLOW_SYSTEM_CFLAGS=1
export CFLAGS="-I/tmp/sysroot/usr/include -I/tmp/sysroot/usr/include/glib-2.0 -I/tmp/sysroot/usr/lib/x86_64-linux-gnu/glib-2.0/include -I/tmp/sysroot/usr/include/gdk-pixbuf-2.0 -I/tmp/sysroot/usr/include/pango-1.0 -I/tmp/sysroot/usr/include/cairo -I/tmp/sysroot/usr/include/graphene-1.0 -I/tmp/sysroot/usr/lib/x86_64-linux-gnu/graphene-1.0/include -I/tmp/sysroot/usr/include/gtk-4.0 -I/tmp/sysroot/usr/include/freetype2 -I/tmp/sysroot/usr/include/harfbuzz -I/tmp/sysroot/usr/include/libpng16 -I/tmp/sysroot/usr/include/pixman-1"
export BINDGEN_EXTRA_CLANG_ARGS="$CFLAGS"
export RUSTFLAGS="-L /tmp/sysroot-lib"

exec /home/kali/.cargo/bin/cargo "$@"
