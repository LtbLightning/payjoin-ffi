#!/usr/bin/env bash

set -euo pipefail

# Make all scripts executable
chmod +x ./scripts/*.sh

# Install Python dependencies
python3 --version
pip install -r requirements.txt -r requirements-dev.txt

# Build native binaries in parallel
echo "Building native binaries..."
./scripts/generate_linux.sh &
./scripts/generate_macos_arm64.sh &
./scripts/generate_macos_x86_64.sh &
wait

# Combine macOS binaries
echo "Combining macOS binaries..."
./scripts/combine_macos_binaries.sh

echo "All done!" 