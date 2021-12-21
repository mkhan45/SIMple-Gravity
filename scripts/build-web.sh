#!/bin/sh

./scripts/wasm-bindgen-macroquad.sh simple_gravity

# https://github.com/WebAssembly/wabt
# wasm-strip docs/wbindgen/simple_gravity.wasm
mv docs/wbindgen/simple_gravity_bg.wasm docs/
mv docs/wbindgen/simple_gravity.js docs/

if [ "$1" = "serve" ]
then
    # cargo install basic-http-server
    basic-http-server docs
fi
