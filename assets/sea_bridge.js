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
     * SEA.pair(seed) — derive a deterministic key pair from a seed string.
     * @param {string} seed - passphrase / seed to derive the key pair from
     * @returns {Promise<string>} JSON: { pub, priv, epub, epriv }
     */
    async function pairFromSeed(seed) {
        console.log("[sea_bridge.pairFromSeed] seed length=", seed.length);
        var p = await _sea().pair(seed);
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
