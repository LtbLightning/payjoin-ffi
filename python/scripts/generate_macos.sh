#!/usr/bin/env bash

set -euo pipefail
python3 --version
pip install -r requirements.txt -r requirements-dev.txt
LIBNAME=libpayjoin_ffi.dylib

echo "Generating payjoin_ffi.py..."
cd ../
cargo build --features uniffi --profile release 
cargo run --features uniffi --profile release --bin uniffi-bindgen generate --library target/release/$LIBNAME --language python --out-dir python/src/payjoin/

echo "Generating native binaries..."
rustup target add aarch64-apple-darwin x86_64-apple-darwin

cargo build --profile release-smaller --target aarch64-apple-darwin --features uniffi
echo "Done building aarch64-apple-darwin"

cargo build --profile release-smaller --target x86_64-apple-darwin --features uniffi
echo "Done building x86_64-apple-darwin"

echo "Building macos fat library"

lipo -create -output python/src/payjoin/$LIBNAME \
        target/aarch64-apple-darwin/release-smaller/$LIBNAME \
        target/x86_64-apple-darwin/release-smaller/$LIBNAME


echo "All done!"