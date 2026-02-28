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
        if (!SEA) { SEA = Gun.SEA; }
        return SEA;
    }

    /**
     * SEA.pair() — generate a new cryptographic key pair.
     * Returns JSON: { pub, priv, epub, epriv }
     * @returns {Promise<string>}
     */
    async function pair() {
        var p = await _sea().pair();
        return JSON.stringify(p);
    }

    /**
     * SEA.pair(seed) — derive a deterministic key pair from a seed string.
     * @param {string} seed - passphrase / seed to derive the key pair from
     * @returns {Promise<string>} JSON: { pub, priv, epub, epriv }
     */
    async function pairFromSeed(seed) {
        var p = await _sea().pair(seed);
        return JSON.stringify(p);
    }

    /**
     * SEA.sign(data, pair) — sign data with a key pair.
     * @param {string} data - the data to sign (plain string)
     * @param {string} pairJson - JSON key pair { pub, priv, epub, epriv }
     * @returns {Promise<string>} - the signed message string, or "" on error
     */
    async function sign(data, pairJson) {
        var kp = JSON.parse(pairJson);
        var msg = await _sea().sign(data, kp);
        return (msg === undefined) ? "" : msg;
    }

    /**
     * SEA.verify(message, pub) — verify a signed message.
     * @param {string} message - the signed message from sign()
     * @param {string} pub - the public key string (pair.pub)
     * @returns {Promise<string>} - the original data if valid, or "" if invalid
     */
    async function verify(message, pub_key) {
        var data = await _sea().verify(message, pub_key);
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
        var key;
        try { key = JSON.parse(pairOrPassphrase); } catch (e) { key = pairOrPassphrase; }
        var enc = await _sea().encrypt(data, key);
        return (enc === undefined) ? "" : enc;
    }

    /**
     * SEA.decrypt(message, pair) — decrypt data.
     * @param {string} message - the encrypted message from encrypt()
     * @param {string} pairOrPassphrase - JSON key pair or a passphrase string
     * @returns {Promise<string>} - the decrypted plaintext, or "" on error
     */
    async function decrypt(message, pairOrPassphrase) {
        var key;
        try { key = JSON.parse(pairOrPassphrase); } catch (e) { key = pairOrPassphrase; }
        var dec = await _sea().decrypt(message, key);
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
        var salt;
        try { salt = JSON.parse(saltJson); } catch (e) { salt = saltJson; }
        var hash = await _sea().work(data, salt);
        return (hash === undefined) ? "" : hash;
    }

    /**
     * SEA.secret(otherEpub, myPair) — ECDH shared secret derivation.
     * @param {string} otherEpub - the other party's epub (public encryption key)
     * @param {string} myPairJson - JSON of my full key pair
     * @returns {Promise<string>} - the shared secret string, or "" on error
     */
    async function secret(otherEpub, myPairJson) {
        var myPair = JSON.parse(myPairJson);
        var sec = await _sea().secret(otherEpub, myPair);
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
