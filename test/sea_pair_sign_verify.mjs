/**
 * test/sea_pair_sign_verify.mjs
 *
 * Verifies that key pairs derived by `pairFromSeed()` are fully functional
 * for both ECDSA signing/verification and ECDH shared secret derivation.
 *
 * Background:
 *   It is not enough to check that the same seed produces the same keys — the
 *   keys must actually *work* with the WebCrypto API.  This test exercises:
 *
 *     1. ECDSA sign / verify — import the derived pub/priv as JWK (the same
 *        format GUN SEA uses internally via `keyForPair`), sign data, and
 *        verify the signature with the public key only.
 *
 *     2. ECDH shared secret — derive two independent key pairs (Alice & Bob),
 *        compute the shared secret from both sides, and confirm they match.
 *        This is the primitive behind `SEA.secret()`.
 *
 * Run:
 *   node test/sea_pair_sign_verify.mjs
 */

const crypto = globalThis.crypto || (await import('crypto')).webcrypto;

// ---------------------------------------------------------------------------
// pairFromSeed — identical logic to sea_bridge.js (standalone for testing)
// ---------------------------------------------------------------------------
async function pairFromSeed(seed) {
    const subtle = crypto.subtle;
    const seedBytes = new TextEncoder().encode(seed);
    const baseKey = await subtle.importKey(
        'raw', seedBytes, { name: 'PBKDF2' }, false, ['deriveBits']
    );

    const ecdsaBits = new Uint8Array(await subtle.deriveBits({
        name: 'PBKDF2',
        salt: new TextEncoder().encode('zsozso-sea-ecdsa'),
        iterations: 100000,
        hash: 'SHA-256'
    }, baseKey, 256));

    const ecdhBits = new Uint8Array(await subtle.deriveBits({
        name: 'PBKDF2',
        salt: new TextEncoder().encode('zsozso-sea-ecdh'),
        iterations: 100000,
        hash: 'SHA-256'
    }, baseKey, 256));

    const curveN = BigInt(
        '0xFFFFFFFF00000000FFFFFFFFFFFFFFFFBCE6FAADA7179E84F3B9CAC2FC632551'
    );

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

    function buildPkcs8(d) {
        const prefix = new Uint8Array([
            0x30, 0x41, 0x02, 0x01, 0x00, 0x30, 0x13,
            0x06, 0x07, 0x2a, 0x86, 0x48, 0xce, 0x3d, 0x02, 0x01,
            0x06, 0x08, 0x2a, 0x86, 0x48, 0xce, 0x3d, 0x03, 0x01, 0x07,
            0x04, 0x27, 0x30, 0x25, 0x02, 0x01, 0x01, 0x04, 0x20
        ]);
        const buf = new Uint8Array(prefix.length + 32);
        buf.set(prefix);
        buf.set(d, prefix.length);
        return buf;
    }

    const ecdsaD = toValidScalar(ecdsaBits);
    const ecdhD  = toValidScalar(ecdhBits);

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

const subtle = crypto.subtle;

// ---- Test 1: ECDSA sign + verify -----------------------------------------
console.log("=== Test: ECDSA sign / verify with derived keys ===\n");

const pair = await pairFromSeed("test-passphrase");

// Reconstruct the ECDSA private key as JWK — this mirrors how GUN SEA's
// internal `keyForPair(pair)` function rebuilds the CryptoKey for signing.
const [x, y] = pair.pub.split('.');
const signJwk = { kty: 'EC', crv: 'P-256', x, y, d: pair.priv };
const signKey = await subtle.importKey(
    'jwk', signJwk,
    { name: 'ECDSA', namedCurve: 'P-256' }, false, ['sign']
);

// Sign some data
const message = "Hello Iceberg Protocol!";
const data = new TextEncoder().encode(message);
const signature = await subtle.sign(
    { name: 'ECDSA', hash: 'SHA-256' }, signKey, data
);

// Verify with public key only (no private key needed)
const verifyJwk = { kty: 'EC', crv: 'P-256', x, y };
const verifyKey = await subtle.importKey(
    'jwk', verifyJwk,
    { name: 'ECDSA', namedCurve: 'P-256' }, false, ['verify']
);
const isValid = await subtle.verify(
    { name: 'ECDSA', hash: 'SHA-256' }, verifyKey, signature, data
);

assert(isValid, "Signature verified with derived ECDSA keys");

// Verify that tampering breaks the signature
const tampered = new TextEncoder().encode("Tampered message!");
const isInvalid = await subtle.verify(
    { name: 'ECDSA', hash: 'SHA-256' }, verifyKey, signature, tampered
);
assert(!isInvalid, "Tampered data correctly fails verification");

// ---- Test 2: ECDH shared secret ------------------------------------------
console.log("\n=== Test: ECDH shared secret with derived keys ===\n");

// Derive two independent key pairs (simulating two users)
const alice = await pairFromSeed("alice-seed");
const bob   = await pairFromSeed("bob-seed");

// Alice computes the shared secret using her private key + Bob's public key
const [ax, ay] = alice.epub.split('.');
const [bx, by] = bob.epub.split('.');

const alicePrivKey = await subtle.importKey(
    'jwk',
    { kty: 'EC', crv: 'P-256', x: ax, y: ay, d: alice.epriv },
    { name: 'ECDH', namedCurve: 'P-256' }, false, ['deriveBits']
);
const bobPubKey = await subtle.importKey(
    'jwk',
    { kty: 'EC', crv: 'P-256', x: bx, y: by },
    { name: 'ECDH', namedCurve: 'P-256' }, false, []
);
const secretAlice = new Uint8Array(
    await subtle.deriveBits({ name: 'ECDH', public: bobPubKey }, alicePrivKey, 256)
);

// Bob computes the shared secret using his private key + Alice's public key
// (ECDH guarantees both sides derive the same value)
const bobPrivKey = await subtle.importKey(
    'jwk',
    { kty: 'EC', crv: 'P-256', x: bx, y: by, d: bob.epriv },
    { name: 'ECDH', namedCurve: 'P-256' }, false, ['deriveBits']
);
const alicePubKey = await subtle.importKey(
    'jwk',
    { kty: 'EC', crv: 'P-256', x: ax, y: ay },
    { name: 'ECDH', namedCurve: 'P-256' }, false, []
);
const secretBob = new Uint8Array(
    await subtle.deriveBits({ name: 'ECDH', public: alicePubKey }, bobPrivKey, 256)
);

// Both sides must arrive at the same shared secret
const secretsMatch = secretAlice.length === secretBob.length
    && secretAlice.every((b, i) => b === secretBob[i]);

assert(secretsMatch, "ECDH shared secret matches (Alice ↔ Bob)");

// Sanity: shared secret should not be all zeros
const allZero = secretAlice.every(b => b === 0);
assert(!allZero, "Shared secret is not all zeros");

// ---- Summary --------------------------------------------------------------
console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed > 0 ? 1 : 0);
