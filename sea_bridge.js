/**
 * sea_bridge.js — Thin JS bridge between GUN SEA and Rust/WASM.
 *
 * Exposes window.__sea_bridge with async functions that Rust calls
 * via js_sys::eval → Promise → JsFuture.
 *
 * Both gun.js AND sea.js must be loaded before this script.
 * SEA is available as Gun.SEA after sea.js loads.
 */
(function () {
    "use strict";

    var SEA = null;

    function _sea() {
        if (!SEA) {
            SEA = Gun.SEA;
            console.log("[sea_bridge._sea] SEA initialized:", SEA ? "ok" : "null");
        }
        return SEA;
    }

    /**
     * SEA.pair() — generate a new cryptographic key pair.
     * Returns JSON: { pub, priv, epub, epriv }
     * @returns {Promise<string>}
     */
    async function pair() {
        console.log("[sea_bridge.pair] Generating random key pair");
        var p = await _sea().pair();
        console.log("[sea_bridge.pair] Result:", p ? "got pair, pub=" + p.pub : "null");
        return JSON.stringify(p);
    }

    /**
     * Derive a deterministic P-256 key pair from a seed/passphrase.
     *
     * Uses PBKDF2 (100 000 iterations, SHA-256) with two fixed,
     * application-specific salts to derive independent ECDSA and ECDH
     * private key scalars.  The scalars are reduced modulo (n-1)+1 so
     * they are always valid P-256 private keys in [1, n-1].
     *
     * Identical seed → identical key pair (deterministic).
     *
     * @param {string} seed - passphrase / seed to derive the key pair from
     * @returns {Promise<string>} JSON: { pub, priv, epub, epriv }
     */
    async function pairFromSeed(seed) {
        console.log("[sea_bridge.pairFromSeed] Deriving deterministic key pair, seed length=", seed.length);

        var subtle = crypto.subtle;
        var seedBytes = new TextEncoder().encode(seed);

        // Import the seed as PBKDF2 base key material
        var baseKey = await subtle.importKey(
            'raw', seedBytes, { name: 'PBKDF2' }, false, ['deriveBits']
        );

        // Derive 32 bytes for ECDSA private key
        var ecdsaBits = new Uint8Array(await subtle.deriveBits({
            name: 'PBKDF2',
            salt: new TextEncoder().encode('zsozso-sea-ecdsa'),
            iterations: 100000,
            hash: 'SHA-256'
        }, baseKey, 256));

        // Derive 32 bytes for ECDH private key (independent salt)
        var ecdhBits = new Uint8Array(await subtle.deriveBits({
            name: 'PBKDF2',
            salt: new TextEncoder().encode('zsozso-sea-ecdh'),
            iterations: 100000,
            hash: 'SHA-256'
        }, baseKey, 256));

        // P-256 curve order
        var curveN = BigInt('0xFFFFFFFF00000000FFFFFFFFFFFFFFFFBCE6FAADA7179E84F3B9CAC2FC632551');

        // Convert raw bytes to a valid P-256 private key scalar in [1, n-1]
        function toValidScalar(bytes) {
            var hex = '';
            for (var i = 0; i < bytes.length; i++) hex += bytes[i].toString(16).padStart(2, '0');
            var val = (BigInt('0x' + hex) % (curveN - 1n)) + 1n;
            var out = val.toString(16).padStart(64, '0');
            var result = new Uint8Array(32);
            for (var i = 0; i < 32; i++) result[i] = parseInt(out.substr(i * 2, 2), 16);
            return result;
        }

        // Build a minimal PKCS8 DER encoding for a P-256 private key.
        // This lets WebCrypto compute the public point (x, y) for us.
        // Structure: SEQUENCE { version=0, AlgId{ecPublicKey, P-256}, OCTET{ECPrivateKey{v=1, d}} }
        function buildPkcs8(d) {
            var prefix = new Uint8Array([
                0x30, 0x41,                                         // SEQUENCE 65 bytes
                0x02, 0x01, 0x00,                                   // INTEGER 0 (version)
                0x30, 0x13,                                         // SEQUENCE 19 bytes (AlgorithmIdentifier)
                0x06, 0x07, 0x2a, 0x86, 0x48, 0xce, 0x3d, 0x02, 0x01, // OID ecPublicKey
                0x06, 0x08, 0x2a, 0x86, 0x48, 0xce, 0x3d, 0x03, 0x01, 0x07, // OID P-256
                0x04, 0x27,                                         // OCTET STRING 39 bytes
                0x30, 0x25,                                         // SEQUENCE 37 bytes (ECPrivateKey)
                0x02, 0x01, 0x01,                                   // INTEGER 1 (version)
                0x04, 0x20                                          // OCTET STRING 32 bytes (private key)
            ]);
            var buf = new Uint8Array(prefix.length + 32);
            buf.set(prefix);
            buf.set(d, prefix.length);
            return buf;
        }

        var ecdsaD = toValidScalar(ecdsaBits);
        var ecdhD  = toValidScalar(ecdhBits);

        // Import ECDSA key via PKCS8, export as JWK to obtain public point
        var ecdsaKey = await subtle.importKey(
            'pkcs8', buildPkcs8(ecdsaD),
            { name: 'ECDSA', namedCurve: 'P-256' }, true, ['sign']
        );
        var ecdsaJwk = await subtle.exportKey('jwk', ecdsaKey);

        // Import ECDH key via PKCS8, export as JWK to obtain public point
        var ecdhKey = await subtle.importKey(
            'pkcs8', buildPkcs8(ecdhD),
            { name: 'ECDH', namedCurve: 'P-256' }, true, ['deriveBits']
        );
        var ecdhJwk = await subtle.exportKey('jwk', ecdhKey);

        // Format in GUN SEA style: pub/epub = "x.y"  priv/epriv = "d"
        var p = {
            pub:   ecdsaJwk.x + '.' + ecdsaJwk.y,
            priv:  ecdsaJwk.d,
            epub:  ecdhJwk.x  + '.' + ecdhJwk.y,
            epriv: ecdhJwk.d,
        };

        console.log("[sea_bridge.pairFromSeed] Result:", p ? "got pair, pub=" + p.pub : "null");
        return JSON.stringify(p);
    }

    /**
     * SEA.sign(data, pair) — sign data with a key pair.
     * @param {string} data - the data to sign (plain string)
     * @param {string} pairJson - JSON key pair { pub, priv, epub, epriv }
     * @returns {Promise<string>} - the signed message string, or "" on error
     */
    async function sign(data, pairJson) {
        console.log("[sea_bridge.sign] data length=", data.length);
        var kp = JSON.parse(pairJson);
        console.log("[sea_bridge.sign] Signing with pub=", kp.pub);
        var msg = await _sea().sign(data, kp);
        console.log("[sea_bridge.sign] Result:", msg === undefined ? "undefined" : "signed");
        return (msg === undefined) ? "" : msg;
    }

    /**
     * SEA.verify(message, pub) — verify a signed message.
     * @param {string} message - the signed message from sign()
     * @param {string} pub - the public key string (pair.pub)
     * @returns {Promise<string>} - the original data if valid, or "" if invalid
     */
    async function verify(message, pub_key) {
        console.log("[sea_bridge.verify] message length=", message.length, "pub_key=", pub_key);
        var data = await _sea().verify(message, pub_key);
        console.log("[sea_bridge.verify] Result:", data === undefined ? "invalid" : "valid");
        if (data === undefined) return "";
        return (typeof data === "string") ? data : JSON.stringify(data);
    }

    /**
     * SEA.encrypt(data, pair) — encrypt data.
     * @param {string} data - plaintext to encrypt
     * @param {string} pairOrPassphrase - JSON key pair or a passphrase string
     * @returns {Promise<string>} - the encrypted message string, or "" on error
     */
    async function encrypt(data, pairOrPassphrase) {
        console.log("[sea_bridge.encrypt] data length=", data.length);
        var key;
        try { key = JSON.parse(pairOrPassphrase); } catch (e) { key = pairOrPassphrase; }
        var enc = await _sea().encrypt(data, key);
        console.log("[sea_bridge.encrypt] Result:", enc === undefined ? "undefined" : "encrypted");
        return (enc === undefined) ? "" : enc;
    }

    /**
     * SEA.decrypt(message, pair) — decrypt data.
     * @param {string} message - the encrypted message from encrypt()
     * @param {string} pairOrPassphrase - JSON key pair or a passphrase string
     * @returns {Promise<string>} - the decrypted plaintext, or "" on error
     */
    async function decrypt(message, pairOrPassphrase) {
        console.log("[sea_bridge.decrypt] message length=", message.length);
        var key;
        try { key = JSON.parse(pairOrPassphrase); } catch (e) { key = pairOrPassphrase; }
        var dec = await _sea().decrypt(message, key);
        console.log("[sea_bridge.decrypt] Result:", dec === undefined ? "undefined" : "decrypted");
        if (dec === undefined) return "";
        return (typeof dec === "string") ? dec : JSON.stringify(dec);
    }

    /**
     * SEA.work(data, salt) — proof of work / key derivation (PBKDF2).
     * @param {string} data - the data to hash
     * @param {string} saltJson - JSON key pair to use as salt, or a passphrase string
     * @returns {Promise<string>} - the derived hash, or "" on error
     */
    async function work(data, saltJson) {
        console.log("[sea_bridge.work] data length=", data.length, "salt length=", saltJson.length);
        var salt;
        try { salt = JSON.parse(saltJson); } catch (e) { salt = saltJson; }
        var hash = await _sea().work(data, salt);
        console.log("[sea_bridge.work] Result:", hash === undefined ? "undefined" : "hash computed");
        return (hash === undefined) ? "" : hash;
    }

    /**
     * SEA.secret(otherEpub, myPair) — ECDH shared secret derivation.
     * @param {string} otherEpub - the other party's epub (public encryption key)
     * @param {string} myPairJson - JSON of my full key pair
     * @returns {Promise<string>} - the shared secret string, or "" on error
     */
    async function secret(otherEpub, myPairJson) {
        console.log("[sea_bridge.secret] otherEpub length=", otherEpub.length);
        var myPair = JSON.parse(myPairJson);
        var sec = await _sea().secret(otherEpub, myPair);
        console.log("[sea_bridge.secret] Result:", sec === undefined ? "undefined" : "secret derived");
        return (sec === undefined) ? "" : sec;
    }

    // Expose on window
    window.__sea_bridge = {
        pair: pair,
        pairFromSeed: pairFromSeed,
        sign: sign,
        verify: verify,
        encrypt: encrypt,
        decrypt: decrypt,
        work: work,
        secret: secret,
    };
})();
