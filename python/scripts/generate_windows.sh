#!/usr/bin/env bash

set -euo pipefail
python3 --version
pip install --user -r requirements.txt

LIBNAME=payjoin_ffi.dll

echo "Generating payjoin_ffi.py..."
cd ../
cargo run --bin uniffi-bindgen generate src/payjoin_ffi.udl --language python --out-dir python/src/payjoin/


echo "Generating native binaries..."
rustup target add x86_64-pc-windows-gnu
cargo build --profile release-smaller --target x86_64-pc-windows-gnu --features uniffi

echo "Copying windows payjoin_ffi.dll"
cp target/$WINDOWS_TARGET/release-smaller/$LIBNAME python/src/payjoin/$LIBNAME

echo "All done!"