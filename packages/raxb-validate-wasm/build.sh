#!/bin/bash

source $EMSDK/emsdk_env.sh
mkdir -p dist
cp package.tmp.json dist/package.json
emcc -Oz -s WASM=1 -s EMULATE_FUNCTION_POINTER_CASTS=1 \
	-s MODULARIZE=1 \
	-s IMPORTED_MEMORY \
	-s ALLOW_MEMORY_GROWTH \
	-s MAXIMUM_MEMORY=4GB \
	-s NO_EXIT_RUNTIME=0 \
	-s EXPORT_ES6=1 \
	-s EXPORT_NAME=Module \
	-Ithird_party/lz4 \
	-I../../crates/raxb-libxml2-sys/third_party/libxml2/include \
	-I../../crates/raxb-libxml2-sys/third_party/libxml2-build/include \
	--closure 1 \
	--no-entry \
	-lembind \
	--emit-tsd raxb-validate-wasm.d.ts \
	./src/lib.cpp ./third_party/lz4/lz4.cpp ../../crates/raxb-libxml2-sys/third_party/libxml2-build/.libs/libxml2.a \
	-o dist/raxb-validate-wasm.js \
	-s ENVIRONMENT=web