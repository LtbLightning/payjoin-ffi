#!/usr/bin/env bash

set -euo pipefail

LIBNAME=libpayjoin_ffi.dylib

echo "Combining arm64 and x86_64 binaries into fat binary..."
lipo -create -output python/src/payjoin/$LIBNAME \
        python/src/payjoin/libpayjoin_ffi_arm64.dylib \
        python/src/payjoin/libpayjoin_ffi_x86_64.dylib

echo "Cleaning up architecture-specific binaries..."
rm python/src/payjoin/libpayjoin_ffi_arm64.dylib
rm python/src/payjoin/libpayjoin_ffi_x86_64.dylib

echo "Fat binary created successfully!" 