/**
 * test/sea_pair_sign_verify.mjs
 *
 * Verifies that key pairs derived by `pairFromSeed()` are fully functional
 * for both ECDSA signing/verification and ECDH shared secret derivation.
 *
 * Run:
 *   node test/sea_pair_sign_verify.mjs
 */

import { pairFromSeed } from './p256.mjs';

const crypto = globalThis.crypto || (await import('crypto')).webcrypto;

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
