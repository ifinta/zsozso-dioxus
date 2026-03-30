/**
 * test/sea_pair_deterministic.mjs
 *
 * Verifies that the custom `pairFromSeed()` implementation produces
 * **deterministic** P-256 key pairs from a passphrase/seed string.
 *
 * What this test checks:
 *   ✓ Same seed produces identical {pub, priv, epub, epriv} every time
 *   ✓ Different seeds produce different key pairs
 *
 * Run:
 *   node test/sea_pair_deterministic.mjs
 */

import { pairFromSeed } from './p256.mjs';

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
