/**
 * test/sea_pair_deterministic.mjs
 *
 * Verifies that the custom `pairFromSeed()` implementation in sea_bridge.js
 * produces **deterministic** P-256 key pairs from a passphrase/seed string.
 *
 * Background:
 *   GUN SEA's built-in `SEA.pair()` always generates random keys — it does NOT
 *   accept a seed parameter (the first argument is a callback). Our bridge
 *   replaces `pairFromSeed()` with a custom implementation that uses:
 *
 *     1. PBKDF2 (100 000 iterations, SHA-256) with two fixed application-specific
 *        salts ("zsozso-sea-ecdsa" and "zsozso-sea-ecdh") to derive 32 bytes of
 *        private key material for each key type (ECDSA for signing, ECDH for
 *        encryption / shared secrets).
 *
 *     2. Modular reduction of the raw bytes modulo the P-256 curve order to
 *        ensure the scalar is in the valid range [1, n-1].
 *
 *     3. WebCrypto `importKey('pkcs8')` + `exportKey('jwk')` to let the browser
 *        compute the public point (x, y) from the private scalar.
 *
 *   The output format matches GUN SEA conventions:
 *     - pub  / epub  = "base64url(x).base64url(y)"
 *     - priv / epriv = "base64url(d)"
 *
 * What this test checks:
 *   ✓ Same seed produces identical {pub, priv, epub, epriv} every time
 *   ✓ Different seeds produce different key pairs
 *
 * Run:
 *   node test/sea_pair_deterministic.mjs
 */

const crypto = globalThis.crypto || (await import('crypto')).webcrypto;

// ---------------------------------------------------------------------------
// pairFromSeed — identical logic to sea_bridge.js (standalone for testing)
// ---------------------------------------------------------------------------
async function pairFromSeed(seed) {
    const subtle = crypto.subtle;
    const seedBytes = new TextEncoder().encode(seed);

    // Import the passphrase as raw key material for PBKDF2
    const baseKey = await subtle.importKey(
        'raw', seedBytes, { name: 'PBKDF2' }, false, ['deriveBits']
    );

    // Derive 32 bytes for the ECDSA (signing) private key
    const ecdsaBits = new Uint8Array(await subtle.deriveBits({
        name: 'PBKDF2',
        salt: new TextEncoder().encode('zsozso-sea-ecdsa'),
        iterations: 100000,
        hash: 'SHA-256'
    }, baseKey, 256));

    // Derive 32 bytes for the ECDH (encryption) private key — independent salt
    const ecdhBits = new Uint8Array(await subtle.deriveBits({
        name: 'PBKDF2',
        salt: new TextEncoder().encode('zsozso-sea-ecdh'),
        iterations: 100000,
        hash: 'SHA-256'
    }, baseKey, 256));

    // P-256 curve order (n)
    const curveN = BigInt(
        '0xFFFFFFFF00000000FFFFFFFFFFFFFFFFBCE6FAADA7179E84F3B9CAC2FC632551'
    );

    /**
     * Reduce raw 32-byte value to a valid P-256 private key scalar in [1, n-1].
     *
     * We compute:  scalar = (rawValue mod (n - 1)) + 1
     *
     * This guarantees the result is never 0 (invalid) and is always < n.
     */
    function toValidScalar(bytes) {
        let hex = '';
        for (let i = 0; i < bytes.length; i++) {
            hex += bytes[i].toString(16).padStart(2, '0');
        }
        const val = (BigInt('0x' + hex) % (curveN - 1n)) + 1n;
        const out = val.toString(16).padStart(64, '0');
        const result = new Uint8Array(32);
        for (let i = 0; i < 32; i++) {
            result[i] = parseInt(out.substr(i * 2, 2), 16);
        }
        return result;
    }

    /**
     * Build a minimal PKCS#8 DER-encoded wrapper around a raw 32-byte
     * P-256 private key scalar.
     *
     * This is the simplest valid PKCS#8 structure that WebCrypto will accept
     * for `importKey('pkcs8', ...)`.  It encodes:
     *
     *   SEQUENCE {
     *     INTEGER 0                          -- version
     *     SEQUENCE {                         -- AlgorithmIdentifier
     *       OID 1.2.840.10045.2.1           -- ecPublicKey
     *       OID 1.2.840.10045.3.1.7         -- P-256 (secp256r1)
     *     }
     *     OCTET STRING {                    -- privateKey
     *       SEQUENCE {                      -- ECPrivateKey (RFC 5915)
     *         INTEGER 1                     -- version
     *         OCTET STRING (32 bytes)       -- private key scalar d
     *       }
     *     }
     *   }
     *
     * Note: the public key is intentionally omitted — WebCrypto will derive
     * the public point from d when we call exportKey('jwk').
     */
    function buildPkcs8(d) {
        const prefix = new Uint8Array([
            0x30, 0x41,                                                     // SEQUENCE (65 bytes)
            0x02, 0x01, 0x00,                                               // INTEGER 0 (version)
            0x30, 0x13,                                                     // SEQUENCE (19 bytes) — AlgorithmIdentifier
            0x06, 0x07, 0x2a, 0x86, 0x48, 0xce, 0x3d, 0x02, 0x01,         // OID ecPublicKey
            0x06, 0x08, 0x2a, 0x86, 0x48, 0xce, 0x3d, 0x03, 0x01, 0x07,   // OID P-256
            0x04, 0x27,                                                     // OCTET STRING (39 bytes)
            0x30, 0x25,                                                     // SEQUENCE (37 bytes) — ECPrivateKey
            0x02, 0x01, 0x01,                                               // INTEGER 1 (version)
            0x04, 0x20                                                      // OCTET STRING (32 bytes) — d
        ]);
        const buf = new Uint8Array(prefix.length + 32);
        buf.set(prefix);
        buf.set(d, prefix.length);
        return buf;
    }

    const ecdsaD = toValidScalar(ecdsaBits);
    const ecdhD  = toValidScalar(ecdhBits);

    // Import as PKCS#8, then export as JWK to get the public point (x, y)
    const ecdsaKey = await subtle.importKey(
        'pkcs8', buildPkcs8(ecdsaD),
        { name: 'ECDSA', namedCurve: 'P-256' }, true, ['sign']
    );
    const ecdsaJwk = await subtle.exportKey('jwk', ecdsaKey);

    const ecdhKey = await subtle.importKey(
        'pkcs8', buildPkcs8(ecdhD),
        { name: 'ECDH', namedCurve: 'P-256' }, true, ['deriveBits']
    );
    const ecdhJwk = await subtle.exportKey('jwk', ecdhKey);

    // Format in GUN SEA style
    return {
        pub:   ecdsaJwk.x + '.' + ecdsaJwk.y,
        priv:  ecdsaJwk.d,
        epub:  ecdhJwk.x  + '.' + ecdhJwk.y,
        epriv: ecdhJwk.d,
    };
}

// ---------------------------------------------------------------------------
// Test runner
// ---------------------------------------------------------------------------
let passed = 0;
let failed = 0;

function assert(condition, label) {
    if (condition) {
        console.log(`  ✓ ${label}`);
        passed++;
    } else {
        console.error(`  ✗ ${label}`);
        failed++;
    }
}

console.log("=== Test: Deterministic key pair derivation ===\n");

const seed = "my-test-passphrase-12345";

console.log("Deriving pair #1 from seed...");
const p1 = await pairFromSeed(seed);

console.log("Deriving pair #2 from same seed...");
const p2 = await pairFromSeed(seed);

console.log("Deriving pair #3 from different seed...\n");
const p3 = await pairFromSeed("different-seed");

assert(p1.pub   === p2.pub,   "pub   matches for same seed");
assert(p1.priv  === p2.priv,  "priv  matches for same seed");
assert(p1.epub  === p2.epub,  "epub  matches for same seed");
assert(p1.epriv === p2.epriv, "epriv matches for same seed");

assert(p1.pub  !== p3.pub,  "pub  differs for different seed");
assert(p1.epub !== p3.epub, "epub differs for different seed");

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed > 0 ? 1 : 0);
