#!/bin/bash
set -euo pipefail
cd "$(dirname "$0")/.."

PROFILE="${1:-debug}"
TARGET="x86_64-pc-windows-gnu"

if [ "$PROFILE" = "release" ]; then
    CARGO_FLAGS="--release"
    PROFILE_DIR="release"
else
    CARGO_FLAGS=""
    PROFILE_DIR="debug"
fi

BUILD_DIR="target/${TARGET}/${PROFILE_DIR}"
OUTPUT_DIR="dist/skyme-ime-${PROFILE}"

echo "=== Building Skyme for Windows x86_64 ==="
cargo build --target "${TARGET}" ${CARGO_FLAGS} --workspace

echo "=== Creating package at ${OUTPUT_DIR} ==="
mkdir -p "${OUTPUT_DIR}"
cp "${BUILD_DIR}/skyme_ime_service.dll" "${OUTPUT_DIR}/"
cp "${BUILD_DIR}/skyme_deploy.exe" "${OUTPUT_DIR}/"
cp "${BUILD_DIR}/skyme_settings_ui.exe" "${OUTPUT_DIR}/"
cp dist/install.bat "${OUTPUT_DIR}/"
cp dist/uninstall.bat "${OUTPUT_DIR}/"

echo ""
echo "Package created: ${OUTPUT_DIR}"
echo "Copy rime.dll into ${OUTPUT_DIR}/ before distribution."
