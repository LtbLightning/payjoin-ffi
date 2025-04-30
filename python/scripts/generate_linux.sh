#!/usr/bin/env bash
set -euo pipefail
${PYBIN}/python --version
${PYBIN}/pip install -r requirements.txt -r requirements-dev.txt
LIBNAME=libpayjoin_ffi.so
LINUX_TARGET=x86_64-unknown-linux-gnu

cd ../

rustup target add $LINUX_TARGET

echo "Generating native binaries..."
# This is a test script the actual release should not include the test utils feature
cargo build --profile release-smaller --target $LINUX_TARGET --features uniffi,_test-utils

echo "Generating payjoin_ffi.py..."
cargo run --profile release --features uniffi,_test-utils --bin uniffi-bindgen generate --library target/$LINUX_TARGET/release-smaller/$LIBNAME --language python --out-dir python/src/payjoin/

echo "Copying linux payjoin_ffi.so"
cp target/release-smaller/$LIBNAME python/src/payjoin/$LIBNAME

echo "All done!"
