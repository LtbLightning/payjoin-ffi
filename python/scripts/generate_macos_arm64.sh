#!/usr/bin/env bash

set -euo pipefail
python3 --version
pip install -r requirements.txt -r requirements-dev.txt
LIBNAME=libpayjoin_ffi.dylib

cd ../

rustup target add aarch64-apple-darwin

echo "Generating arm64 native binary..."
# This is a test script the actual release should not include the test utils feature
cargo build --profile release-smaller --target aarch64-apple-darwin --features uniffi,_test-utils

echo "Generating payjoin_ffi.py..."
cargo run --profile release --features uniffi,_test-utils --bin uniffi-bindgen generate --library target/aarch64-apple-darwin/release-smaller/$LIBNAME --language python --out-dir python/src/payjoin/

echo "Copying arm64 binary to final location"
cp target/aarch64-apple-darwin/release-smaller/$LIBNAME python/src/payjoin/libpayjoin_ffi_arm64.dylib

echo "All done!"
