#!/usr/bin/env bash
# build.sh — build zsozso and stage output in dist/app/
#
# What it does:
#   1. Stamps a fresh CACHE_NAME (date+time) into assets/sw.js
#   2. Runs `dx build --release`
#   3. Copies the result into dist/app/
#
# Usage:
#   ./build.sh          — build + stage
#   ./build.sh --dry    — print the new CACHE_NAME without building

set -euo pipefail

DRY=false
[[ "${1:-}" == "--dry" ]] && DRY=true

# ── 1. Generate CACHE_NAME ────────────────────────────────────────────────────
BUILD_TS="$(date +%Y%m%d.%H%M)"
GIT_HASH="$(git rev-parse --short=8 HEAD)"
CACHE_NAME="zsozso-v0.${BUILD_TS}-${GIT_HASH}"
SW_FILE="assets/sw.js"

echo "CACHE_NAME → ${CACHE_NAME}"
$DRY && exit 0

# ── 2. Stamp CACHE_NAME into sw.js and index.html ────────────────────────────
# Replace the existing CACHE_NAME line regardless of its current value.
sed -i "s|^const CACHE_NAME = '.*';|const CACHE_NAME = '${CACHE_NAME}';|" "${SW_FILE}"
echo "Stamped ${SW_FILE}"

# Stamp the page-side version so the client can compare with the SW version.
sed -i "s|window.__APP_VERSION = '.*'|window.__APP_VERSION = '${CACHE_NAME}'|" "index.html"
echo "Stamped index.html"

# ── 3. Build ──────────────────────────────────────────────────────────────────
echo "Running: dx build --release --platform web --features web"
dx build --release --platform web --features web

# ── 4. Stage to dist/app/ ────────────────────────────────────────────────────
DX_OUT="target/dx/zsozso/release/web/public"
DIST_DIR="dist/app"

echo "Staging ${DX_OUT}/ → ${DIST_DIR}/"
rm -rf "${DIST_DIR}"
mkdir -p "${DIST_DIR}"
cp -r "${DX_OUT}/." "${DIST_DIR}/"

echo ""
echo "✓ Build complete — CACHE_NAME: ${CACHE_NAME}"
echo "  Staged in: ${DIST_DIR}/"
echo "  Serve:     npx serve dist/ -l 8080  →  http://localhost:8080/app/"
