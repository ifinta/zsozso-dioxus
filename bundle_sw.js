#!/usr/bin/env node
/**
 * bundle_sw.js — Bundle all build assets into a service-worker deployment.
 *
 * Usage:  node bundle_sw.js [-z|-c] <source-folder> <deploy-folder> [base-path]
 *
 * Flags:
 *   -z             External compressed mode — assets stored in a separate
 *                   gzipped JSON file (assets.json.gz).  The SW fetches and
 *                   decompresses it during install.  Output: 3 files.
 *   -c             Inline compressed mode — each file is individually gzipped
 *                   then base64-encoded inside sw.js.  The SW decompresses
 *                   each asset on demand when serving.  Output: 2 files.
 *                   Smaller sw.js than plain inline, no extra fetch needed.
 *
 * Arguments:
 *   source-folder  Build output (e.g. dist/app/)
 *   deploy-folder  Root of deploy tree (e.g. deploy/)
 *   base-path      Optional sub-path the app is served under, e.g. "app"
 *                   or "profil".  When given, output goes to
 *                   <deploy-folder>/<base>/ and the SW strips the /<base>/
 *                   prefix from request URLs.  When omitted, output goes
 *                   directly to <deploy-folder>/ with no stripping.
 *
 * Examples:
 *   node bundle_sw.js dist/app deploy app       →  inline       (~5.4 MB sw.js)
 *   node bundle_sw.js -c dist/app deploy app    →  inline+gzip  (~2.7 MB sw.js)
 *   node bundle_sw.js -z dist/app deploy app    →  external gz  (~7 KB sw.js + ~2.1 MB assets.json.gz)
 *
 * Output (inline mode, default):
 *   sw.js       — original sw.js + ALL assets as raw base64 + fetch handler
 *   index.html  — bootloader that installs the SW then reloads
 *
 * Output (inline compressed mode, -c):
 *   sw.js       — original sw.js + ALL assets as gzip+base64 + fetch handler
 *   index.html  — bootloader that installs the SW then reloads
 *
 * Output (external compressed mode, -z):
 *   sw.js           — original sw.js logic + asset-loading fetch handler
 *   assets.json.gz  — gzipped JSON with all base64-encoded assets
 *   index.html      — bootloader that installs the SW then reloads
 */

const fs   = require('fs');
const path = require('path');
const zlib = require('zlib');

// ── CLI args ─────────────────────────────────────────────────────────────────
const rawArgs       = process.argv.slice(2);
const modeExternal  = rawArgs.includes('-z');
const modeCompact   = rawArgs.includes('-c');
const positional    = rawArgs.filter(a => a !== '-z' && a !== '-c');

const srcFolder    = positional[0];
const deployFolder = positional[1];
const basePath     = (positional[2] || '').replace(/^\/|\/$/g, ''); // strip slashes
const basePrefix   = basePath ? '/' + basePath + '/' : '/';
// mode label used in generated code comments and summary
const modeName     = modeExternal ? 'external-gz' : modeCompact ? 'inline-gz' : 'inline';

if (!srcFolder || !deployFolder) {
    console.error('Usage: node bundle_sw.js [-z|-c] <source-folder> <deploy-folder> [base-path]');
    process.exit(1);
}
if (modeExternal && modeCompact) {
    console.error('Error: -z and -c are mutually exclusive');
    process.exit(1);
}

if (!fs.existsSync(srcFolder)) {
    console.error(`Error: source folder not found: ${srcFolder}`);
    process.exit(1);
}

// ── Mime type map ────────────────────────────────────────────────────────────
const MIME_BY_EXT = {
    '.html':  'text/html',
    '.ico':   'image/x-icon',
    '.png':   'image/png',
    '.js':    'application/javascript',
    '.wasm':  'application/wasm',
    '.json':  'application/json',
};

// ── Collect files recursively ────────────────────────────────────────────────
function walkDir(dir, base) {
    let results = [];
    for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
        const full = path.join(dir, entry.name);
        const rel  = path.join(base, entry.name);
        if (entry.isDirectory()) {
            results = results.concat(walkDir(full, rel));
        } else if (entry.isFile()) {
            results.push(rel);
        }
    }
    return results;
}

// Only skip sw.js — index.html IS embedded so the SW can serve it
const SKIP = new Set(['sw.js']);
const allFiles = walkDir(srcFolder, '')
    .filter(f => !SKIP.has(f));

// ── Read sw.js ───────────────────────────────────────────────────────────────
const swPath = path.join(srcFolder, 'sw.js');
if (!fs.existsSync(swPath)) {
    console.error(`Error: sw.js not found in ${srcFolder}`);
    process.exit(1);
}
let swContent = fs.readFileSync(swPath, 'utf8');

// ── Remove the existing fetch event listener ─────────────────────────────────
function removeFetchListener(code) {
    const marker = "self.addEventListener('fetch'";
    const idx = code.indexOf(marker);
    if (idx === -1) return code;

    let depth = 0;
    let started = false;
    let end = idx;
    for (let i = idx; i < code.length; i++) {
        if (code[i] === '(') { depth++; started = true; }
        else if (code[i] === ')') {
            depth--;
            if (started && depth === 0) {
                end = i + 1;
                if (code[end] === ';') end++;
                while (end < code.length && (code[end] === '\n' || code[end] === '\r')) end++;
                break;
            }
        }
    }
    return code.substring(0, idx).trimEnd() + '\n';
}

swContent = removeFetchListener(swContent);

// ── Build the ASSETS and MIME objects ────────────────────────────────────────
const assets = {};
const compactAssets = {};  // gzip+base64 per file (used by -c mode)
const mimeTypes = {};
let totalRaw = 0;
let totalCompact = 0;

for (const relPath of allFiles) {
    const absPath  = path.join(srcFolder, relPath);
    const buf      = fs.readFileSync(absPath);
    const ext      = path.extname(relPath).toLowerCase();
    const mime     = MIME_BY_EXT[ext] || 'application/octet-stream';
    const key      = relPath.split(path.sep).join('/');

    assets[key]    = buf.toString('base64');
    mimeTypes[key] = mime;
    totalRaw      += buf.length;

    if (modeCompact) {
        const gzBuf = zlib.gzipSync(buf, { level: 9 });
        compactAssets[key] = gzBuf.toString('base64');
        totalCompact += gzBuf.length;
    }
}

// ── Shared helper functions (used in both modes) ─────────────────────────────

function generateServeHelpers() {
    return `
function _b64ToArrayBuffer(b64) {
    var bin = atob(b64);
    var len = bin.length;
    var bytes = new Uint8Array(len);
    for (var i = 0; i < len; i++) bytes[i] = bin.charCodeAt(i);
    return bytes.buffer;
}

function _serve404(pathname) {
    var html = '<!DOCTYPE html><html><head><meta charset="UTF-8">'
        + '<meta name="viewport" content="width=device-width,initial-scale=1.0">'
        + '<title>404 — Not Found</title>'
        + '<style>body{display:flex;align-items:center;justify-content:center;height:100vh;margin:0;'
        + 'font-family:sans-serif;background:#f5f5f5;color:#333;text-align:center}'
        + 'h1{font-size:4em;margin:0;color:#dc3545}p{color:#666;margin:8px 0}'
        + 'a{color:#17a2b8;text-decoration:none;font-weight:bold}'
        + '</style></head><body><div>'
        + '<h1>404</h1>'
        + '<p>The requested resource was not found.</p>'
        + '<p style="font-size:0.85em;font-family:monospace;word-break:break-all">' + pathname + '</p>'
        + '<p style="margin-top:24px"><a href="./">← Back to app</a></p>'
        + '</div></body></html>';
    return new Response(html, {
        status: 404,
        headers: { 'Content-Type': 'text/html; charset=utf-8' }
    });
}
`;
}

function generateFetchHandler(prefix) {
    // Navigation and sub-resource serving — sync for inline/compact, async for external
    const isAsync = modeExternal;
    return `
// Baked-in base path prefix for stripping (set by bundle_sw.js).
var __BASE_PREFIX = '${prefix}';

self.addEventListener('fetch', function(event) {
    var url = new URL(event.request.url);
${modeExternal ? `
    // Let the asset bundle pass through to network
    if (url.origin === self.location.origin && url.pathname.endsWith('assets.json.gz')) return;
` : ''}
    // Navigation requests → serve embedded index.html
    if (event.request.mode === 'navigate') {
        ${isAsync
            ? "event.respondWith(_loadAssets().then(function() { return _serveEmbedded('index.html') || _serve404(url.pathname); }));"
            : "var resp = _serveEmbedded('index.html');\n        if (resp) { event.respondWith(resp); return; }\n        event.respondWith(_serve404(url.pathname));"}
        return;
    }

    // Cross-origin — fall through to normal network fetch
    if (url.origin !== self.location.origin) return;

    // Strip the base prefix to get the embedded-asset key.
    // Example: base="/app/", pathname="/app/assets/foo.js" → "assets/foo.js"
    var relative = url.pathname;
    if (__BASE_PREFIX !== '/' && relative.startsWith(__BASE_PREFIX)) {
        relative = relative.substring(__BASE_PREFIX.length);
    } else if (relative.startsWith('/')) {
        relative = relative.substring(1);
    }

    ${isAsync
        ? `event.respondWith(
        _loadAssets().then(function() {
            return _serveEmbedded(relative) || _serve404(url.pathname);
        })
    );`
        : `var resp = _serveEmbedded(relative);
    if (resp) { event.respondWith(resp); return; }

    // Not embedded and same-origin — return 404
    event.respondWith(_serve404(url.pathname));`}
});
`;
}

function generateBootloader(prefix) {
    return `<!DOCTYPE html>
<html><head><meta charset="UTF-8"><title>Loading…</title>
<meta name="viewport" content="width=device-width,initial-scale=1.0">
<style>body{display:flex;align-items:center;justify-content:center;height:100vh;margin:0;font-family:sans-serif;background:#f5f5f5;color:#333}
.spinner{width:40px;height:40px;border:4px solid #ddd;border-top-color:#17a2b8;border-radius:50%;animation:spin .8s linear infinite}
@keyframes spin{to{transform:rotate(360deg)}}</style></head>
<body><div style="text-align:center"><div class="spinner" style="margin:0 auto 16px"></div><p>Loading app…</p></div>
<script>
if ('serviceWorker' in navigator) {
  // Ensure trailing slash so the URL is within the SW scope
  if (window.location.pathname.slice(-1) !== '/') {
    window.location.replace(window.location.pathname + '/' + window.location.search + window.location.hash);
  } else if (navigator.serviceWorker.controller) {
    window.location.reload();
  } else {
    var reloading = false;
    function doReload() {
      if (reloading) return;
      reloading = true;
      window.location.reload();
    }

    // Primary: listen for controllerchange event
    navigator.serviceWorker.addEventListener('controllerchange', doReload);

    // Fallback: poll every 100ms in case controllerchange was missed
    setInterval(function() {
      if (navigator.serviceWorker.controller) doReload();
    }, 100);

    navigator.serviceWorker.register('${prefix}sw.js', { scope: '${prefix}' });
  }
} else {
  document.body.innerHTML = '<p>Service Workers are not supported in this browser.</p>';
}
</script></body></html>`;
}

// ── Output ───────────────────────────────────────────────────────────────────
const outFolder = basePath ? path.join(deployFolder, basePath) : deployFolder;
fs.mkdirSync(outFolder, { recursive: true });

let outputSw;

if (modeExternal) {
    // ── External compressed mode (-z): assets in separate gzipped JSON file ──

    const assetsJson = JSON.stringify({ assets, mime: mimeTypes });
    const gzipped = zlib.gzipSync(Buffer.from(assetsJson, 'utf8'), { level: 9 });
    fs.writeFileSync(path.join(outFolder, 'assets.json.gz'), gzipped);

    const externalBlock = `
// ── Asset loader (generated by bundle_sw.js -z) ─────────────────────────────

var __ASSETS = null;

async function _loadAssets() {
    if (__ASSETS) return;
    LOG('Loading compressed assets from assets.json.gz …');
    var resp = await fetch('${basePrefix}assets.json.gz');
    if (!resp.ok) throw new Error('Failed to load assets: ' + resp.status);
    var ds = new DecompressionStream('gzip');
    var decompressed = resp.body.pipeThrough(ds);
    var text = await new Response(decompressed).text();
    __ASSETS = JSON.parse(text);
    LOG('Assets loaded:', Object.keys(__ASSETS.assets).length, 'files');
}

// Eagerly load assets during install (before activation)
self.addEventListener('install', function(event) {
    event.waitUntil(_loadAssets());
});

function _serveEmbedded(key) {
    if (!__ASSETS) return null;
    var data = __ASSETS.assets[key];
    if (!data) return null;
    var mime = __ASSETS.mime[key] || 'application/octet-stream';
    return new Response(_b64ToArrayBuffer(data), {
        status: 200,
        headers: { 'Content-Type': mime }
    });
}
${generateServeHelpers()}${generateFetchHandler(basePrefix)}`;

    outputSw = swContent + externalBlock;

} else if (modeCompact) {
    // ── Inline compressed mode (-c): per-file gzip+base64 inside sw.js ───────

    const compactBlock = `
// ── Embedded assets — per-file gzip+base64 (generated by bundle_sw.js -c) ───

const __EMBEDDED_ASSETS = ${JSON.stringify(compactAssets)};

const __EMBEDDED_MIME = ${JSON.stringify(mimeTypes)};

function _serveEmbedded(key) {
    var data = __EMBEDDED_ASSETS[key];
    if (!data) return null;
    var mime = __EMBEDDED_MIME[key] || 'application/octet-stream';
    // Decompress: base64 → gzip bytes → DecompressionStream → raw bytes
    var gzBuf = _b64ToArrayBuffer(data);
    var ds = new DecompressionStream('gzip');
    var writer = ds.writable.getWriter();
    writer.write(new Uint8Array(gzBuf));
    writer.close();
    return new Response(ds.readable, {
        status: 200,
        headers: { 'Content-Type': mime }
    });
}
${generateServeHelpers()}${generateFetchHandler(basePrefix)}`;

    outputSw = swContent + compactBlock;

} else {
    // ── Inline mode (default): raw base64 assets inside sw.js ────────────────

    const inlineBlock = `
// ── Embedded assets (generated by bundle_sw.js) ──────────────────────────────

const __EMBEDDED_ASSETS = ${JSON.stringify(assets)};

const __EMBEDDED_MIME = ${JSON.stringify(mimeTypes)};

function _serveEmbedded(key) {
    var data = __EMBEDDED_ASSETS[key];
    if (!data) return null;
    var mime = __EMBEDDED_MIME[key] || 'application/octet-stream';
    return new Response(_b64ToArrayBuffer(data), {
        status: 200,
        headers: { 'Content-Type': mime }
    });
}
${generateServeHelpers()}${generateFetchHandler(basePrefix)}`;

    outputSw = swContent + inlineBlock;
}

fs.writeFileSync(path.join(outFolder, 'sw.js'), outputSw, 'utf8');

// ── Write bootloader ─────────────────────────────────────────────────────────
fs.writeFileSync(path.join(outFolder, 'index.html'), generateBootloader(basePrefix), 'utf8');

// ── Summary ──────────────────────────────────────────────────────────────────
const swSize = Buffer.byteLength(outputSw, 'utf8');
const modeLabel = modeExternal ? 'external-gz (-z)' : modeCompact ? 'inline-gz (-c)' : 'inline';
console.log(`Bundled ${allFiles.length} files — ${modeLabel} mode`);
console.log(`  Base path: ${basePath ? '/' + basePath + '/' : '/ (root)'}`);
console.log(`  Raw assets: ${(totalRaw / 1024).toFixed(1)} KB`);
console.log(`  Output sw.js: ${(swSize / 1024).toFixed(1)} KB`);
if (modeExternal) {
    const gzSize = fs.statSync(path.join(outFolder, 'assets.json.gz')).size;
    console.log(`  Output assets.json.gz: ${(gzSize / 1024).toFixed(1)} KB`);
}
if (modeCompact) {
    console.log(`  Compressed assets: ${(totalCompact / 1024).toFixed(1)} KB (${(100 * totalCompact / totalRaw).toFixed(0)}% of raw)`);
}
console.log(`  Deploy folder: ${outFolder}/`);
console.log('');
for (const relPath of allFiles) {
    const size = fs.statSync(path.join(srcFolder, relPath)).size;
    const mime = mimeTypes[relPath.split(path.sep).join('/')];
    console.log(`  ${relPath} (${(size / 1024).toFixed(1)} KB) → ${mime}`);
}
