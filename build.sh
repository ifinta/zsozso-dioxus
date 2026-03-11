#!/usr/bin/env bash
# build.sh — build zsozso and create deployment bundle
#
# Pipeline:
#   1. Run `dx build --release`
#   2. Stage output into dist/app/ (preserves manually-added files like gun.js)
#   3. Stamp a fresh CACHE_NAME into dist/app/sw.js and dist/app/index.html
#   4. Run bundle_sw.js to create the deployment in deploy/<prefix>/
#
# Usage:
#   ./build.sh          — build + bundle for GitHub Pages (/zsozso-dioxus/)
#   ./build.sh -live    — build + bundle for self-hosted (/app/)
#   ./build.sh -z       — build + bundle (compressed mode, smaller deploy)
#   ./build.sh --dry    — print the new CACHE_NAME without building

set -euo pipefail

DRY=false
BUNDLE_FLAG=""
LIVE=false
for arg in "$@"; do
  case "$arg" in
    --dry) DRY=true ;;
    -z|-c|-j|-r) BUNDLE_FLAG="$arg" ;;
    -live) LIVE=true ;;
  esac
done

# Deployment prefix: /app/ for live server, /zsozso-dioxus/ for GitHub Pages
if $LIVE; then
  PREFIX="app"
else
  PREFIX="zsozso-dioxus"
fi

# ── 1. Generate CACHE_NAME ────────────────────────────────────────────────────
BUILD_TS="$(date +%Y%m%d.%H%M)"
GIT_HASH="$(git rev-parse --short=8 HEAD)"
CACHE_NAME="zsozso-v0.${BUILD_TS}-${GIT_HASH}"

echo "CACHE_NAME → ${CACHE_NAME}"
$DRY && exit 0

# ── 2. Build ──────────────────────────────────────────────────────────────────
echo "Running: dx build --release --platform web --features web"
dx build --release --platform web --features web

# ── 3. Stage to dist/app/ ────────────────────────────────────────────────────
DX_OUT="target/dx/zsozso/release/web/public"
DIST_DIR="dist/app"

echo "Staging ${DX_OUT}/ → ${DIST_DIR}/"
rm -rf "${DIST_DIR}/assets"
mkdir -p "${DIST_DIR}/assets"
cp -r "${DX_OUT}/." "${DIST_DIR}/"
rm -rf "${DX_OUT}"

# Copy root static assets into dist (in CI there is no persistent dist/)
cp sw.js manifest.json favicon.ico icon-192.png icon-512.png \
   gun.js gun_bridge.js sea.js sea_bridge.js log_bridge.js \
   passkey_bridge.js qr_scanner_bridge.js wascan_bg.wasm wascan.js \
   "${DIST_DIR}/"

# ── 4. Stamp CACHE_NAME ──────────────────────────────────────────────────────
sed -i "s|^const CACHE_NAME = '.*';|const CACHE_NAME = '${CACHE_NAME}';|" "${DIST_DIR}/sw.js"
echo "Stamped ${DIST_DIR}/sw.js"

sed -i "s|window.__APP_VERSION = '.*'|window.__APP_VERSION = '${CACHE_NAME}'|" "${DIST_DIR}/index.html"
echo "Stamped ${DIST_DIR}/index.html"

# ── 5. Bundle for deployment ─────────────────────────────────────────────────
# For -live builds, patch manifest.json paths to match the /app/ prefix
if $LIVE; then
  sed -i 's|"/zsozso-dioxus/"|"/app/"|g' "${DIST_DIR}/manifest.json"
  echo "Patched manifest.json for live deployment (/app/)"
fi

echo "Running: node bundle_sw.js ${BUNDLE_FLAG} -dioxus -logging ${DIST_DIR} deploy ${PREFIX}"
node bundle_sw.js ${BUNDLE_FLAG} -dioxus -logging "${DIST_DIR}" deploy "${PREFIX}"

echo ""
echo "✓ Build complete — CACHE_NAME: ${CACHE_NAME}"
echo "  Deploy from: deploy/${PREFIX}/"
echo "  Test:        npx serve deploy/ -l 8080  →  http://localhost:8080/${PREFIX}/"
