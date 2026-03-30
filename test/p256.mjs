/**
 * test/p256.mjs — Shared P-256 deterministic key derivation module.
 *
 * Extracted from sea_bridge.js so both test files can reuse the same logic
 * without duplicating ~100 lines of curve arithmetic.
 *
 * Uses:
 *   1. PBKDF2 (100 000 iterations, SHA-256) with two fixed salts
 *      ("zsozso-sea-ecdsa" / "zsozso-sea-ecdh") to derive 32 bytes each.
 *   2. Modular reduction to produce valid P-256 private key scalars.
 *   3. Pure-JS BigInt scalar multiplication on the P-256 generator to
 *      compute the public key (x, y) — no PKCS#8 import needed, works
 *      identically across Node.js, Chrome, Firefox, and Safari/iOS.
 *   4. JWK import for validation (WebCrypto rejects invalid points).
 *
 * Run:
 *   node test/sea_pair_deterministic.mjs
 *   node test/sea_pair_sign_verify.mjs
 */

// btoa polyfill for Node.js
if (typeof btoa === 'undefined') {
    globalThis.btoa = (s) => Buffer.from(s, 'binary').toString('base64');
}

const crypto = globalThis.crypto || (await import('crypto')).webcrypto;

// ── P-256 (secp256r1) curve constants ────────────────────────────────────────
const curveP = BigInt('0xFFFFFFFF00000001000000000000000000000000FFFFFFFFFFFFFFFFFFFFFFFF');
const curveA = curveP - 3n; // a = -3
const curveN = BigInt('0xFFFFFFFF00000000FFFFFFFFFFFFFFFFBCE6FAADA7179E84F3B9CAC2FC632551');
const Gx = BigInt('0x6B17D1F2E12C4247F8BCE6E563A440F277037D812DEB33A0F4A13945D898C296');
const Gy = BigInt('0x4FE342E2FE1A7F9B8EE7EB4A7C0F9E162BCE33576B315ECECBB6406837BF51F5');

// ── Modular arithmetic ───────────────────────────────────────────────────────
function mod(a, m) { return ((a % m) + m) % m; }

function modInverse(a, m) {
    let old_r = mod(a, m), r = m;
    let old_s = 1n, s = 0n;
    while (r !== 0n) {
        const q = old_r / r;
        [old_r, r] = [r, old_r - q * r];
        [old_s, s] = [s, old_s - q * s];
    }
    return mod(old_s, m);
}

// ── Elliptic curve point operations ──────────────────────────────────────────
function pointDouble(px, py) {
    if (py === 0n) return [0n, 0n];
    const s = mod((3n * px * px + curveA) * modInverse(2n * py, curveP), curveP);
    const xr = mod(s * s - 2n * px, curveP);
    return [xr, mod(s * (px - xr) - py, curveP)];
}

function pointAdd(x1, y1, x2, y2) {
    if (x1 === 0n && y1 === 0n) return [x2, y2];
    if (x2 === 0n && y2 === 0n) return [x1, y1];
    if (x1 === x2 && y1 === y2) return pointDouble(x1, y1);
    if (x1 === x2) return [0n, 0n]; // point at infinity (inverse)
    const s = mod((y2 - y1) * modInverse(x2 - x1, curveP), curveP);
    const xr = mod(s * s - x1 - x2, curveP);
    return [xr, mod(s * (x1 - xr) - y1, curveP)];
}

/** Double-and-add scalar multiplication: k × P */
function scalarMult(k, px, py) {
    let rx = 0n, ry = 0n;
    let qx = px, qy = py;
    while (k > 0n) {
        if (k & 1n) [rx, ry] = pointAdd(rx, ry, qx, qy);
        [qx, qy] = pointDouble(qx, qy);
        k >>= 1n;
    }
    return [rx, ry];
}

// ── Helpers ──────────────────────────────────────────────────────────────────

/** 32-byte Uint8Array → BigInt scalar in [1, n-1] */
function toValidScalar(bytes) {
    let hex = '';
    for (let i = 0; i < bytes.length; i++) hex += bytes[i].toString(16).padStart(2, '0');
    return (BigInt('0x' + hex) % (curveN - 1n)) + 1n;
}

/** BigInt → base64url string (32 bytes, zero-padded) */
function bigintToBase64url(n) {
    const hex = n.toString(16).padStart(64, '0');
    const bytes = new Uint8Array(32);
    for (let i = 0; i < 32; i++) bytes[i] = parseInt(hex.substr(i * 2, 2), 16);
    const b64 = btoa(String.fromCharCode.apply(null, bytes));
    return b64.replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');
}

// ── Main function ────────────────────────────────────────────────────────────

/**
 * Derive a deterministic GUN SEA key pair from a passphrase.
 * Identical to the logic in sea_bridge.js pairFromSeed().
 *
 * @param {string} seed
 * @returns {Promise<{pub:string, priv:string, epub:string, epriv:string}>}
 */
export async function pairFromSeed(seed) {
    const subtle = crypto.subtle;
    const seedBytes = new TextEncoder().encode(seed);

    const baseKey = await subtle.importKey(
        'raw', seedBytes, { name: 'PBKDF2' }, false, ['deriveBits']
    );

    const ecdsaBits = new Uint8Array(await subtle.deriveBits({
        name: 'PBKDF2', salt: new TextEncoder().encode('zsozso-sea-ecdsa'),
        iterations: 100000, hash: 'SHA-256'
    }, baseKey, 256));

    const ecdhBits = new Uint8Array(await subtle.deriveBits({
        name: 'PBKDF2', salt: new TextEncoder().encode('zsozso-sea-ecdh'),
        iterations: 100000, hash: 'SHA-256'
    }, baseKey, 256));

    // Derive scalars and compute public points via curve arithmetic
    const ecdsaScalar = toValidScalar(ecdsaBits);
    const ecdsaPub    = scalarMult(ecdsaScalar, Gx, Gy);
    const ecdhScalar  = toValidScalar(ecdhBits);
    const ecdhPub     = scalarMult(ecdhScalar, Gx, Gy);

    const ecdsaJwk = {
        kty: 'EC', crv: 'P-256',
        x: bigintToBase64url(ecdsaPub[0]),
        y: bigintToBase64url(ecdsaPub[1]),
        d: bigintToBase64url(ecdsaScalar)
    };
    // Validate by importing — WebCrypto rejects invalid points
    await subtle.importKey('jwk', ecdsaJwk,
        { name: 'ECDSA', namedCurve: 'P-256' }, false, ['sign']);

    const ecdhJwk = {
        kty: 'EC', crv: 'P-256',
        x: bigintToBase64url(ecdhPub[0]),
        y: bigintToBase64url(ecdhPub[1]),
        d: bigintToBase64url(ecdhScalar)
    };
    await subtle.importKey('jwk', ecdhJwk,
        { name: 'ECDH', namedCurve: 'P-256' }, false, ['deriveBits']);

    return {
        pub:   ecdsaJwk.x + '.' + ecdsaJwk.y,
        priv:  ecdsaJwk.d,
        epub:  ecdhJwk.x  + '.' + ecdhJwk.y,
        epriv: ecdhJwk.d,
    };
}
