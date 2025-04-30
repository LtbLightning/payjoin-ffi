#!/usr/bin/env bash

set -euo pipefail
python3 --version
pip install -r requirements.txt -r requirements-dev.txt
LIBNAME=libpayjoin_ffi.dylib

cd ../

rustup target add x86_64-apple-darwin

echo "Generating x86_64 native binary..."
# This is a test script the actual release should not include the test utils feature
cargo build --profile release-smaller --target x86_64-apple-darwin --features uniffi,_test-utils

echo "Generating payjoin_ffi.py..."
cargo run --profile release --features uniffi,_test-utils --bin uniffi-bindgen generate --library target/x86_64-apple-darwin/release-smaller/$LIBNAME --language python --out-dir python/src/payjoin/

echo "Copying x86_64 binary to final location"
cp target/x86_64-apple-darwin/release-smaller/$LIBNAME python/src/payjoin/libpayjoin_ffi_x86_64.dylib

echo "All done!"
