/**
 * gun_bridge.js — Thin JS bridge between GUN.js and Rust/WASM.
 *
 * Exposes window.__gun_bridge with simple functions that Rust calls
 * via js_sys::eval / js_sys::Function / wasm_bindgen.
 *
 * GUN must be loaded before this script (via <script src="https://cdn.jsdelivr.net/npm/gun/gun.js">).
 */
(function () {
    "use strict";

    // The single GUN instance (created by gun_bridge_init)
    let _gun = null;

    // Active .on() subscriptions keyed by numeric ID
    const _subscriptions = {};
    let _nextSubId = 1;

    /**
     * Initialise the GUN instance with the given peer URLs.
     * @param {string} peersJson - JSON array of peer URLs, e.g. '["https://gun-manhattan.herokuapp.com/gun"]'
     */
    function init(peersJson) {
        if (_gun) return; // already initialised
        const peers = JSON.parse(peersJson || "[]");
        _gun = Gun({ peers: peers });
    }

    /**
     * Navigate a GUN chain along a path.
     * @param {string[]} path - Array of keys
     * @returns {GunInstance} - The chained reference
     */
    function _ref(path) {
        let ref = _gun;
        for (const key of path) {
            ref = ref.get(key);
        }
        return ref;
    }

    /**
     * gun.get(path).once(cb) — read once.
     * Returns a Promise that resolves to JSON string of the value (or "null").
     * @param {string} pathJson - JSON array of keys, e.g. '["users","alice","name"]'
     * @returns {Promise<string>} - JSON-encoded value
     */
    function get(pathJson) {
        return new Promise(function (resolve) {
            const path = JSON.parse(pathJson);
            _ref(path).once(function (data) {
                resolve(JSON.stringify(data === undefined ? null : data));
            });
            // Safety timeout — if GUN finds nothing it may never call back
            setTimeout(function () { resolve("null"); }, 3000);
        });
    }

    /**
     * gun.get(path).put(value) — write/update.
     * @param {string} pathJson - JSON array of keys
     * @param {string} valueJson - JSON-encoded value to write
     * @returns {Promise<string>} - "ok" or error string
     */
    function put(pathJson, valueJson) {
        return new Promise(function (resolve) {
            const path = JSON.parse(pathJson);
            const value = JSON.parse(valueJson);
            _ref(path).put(value, function (ack) {
                if (ack.err) {
                    resolve("err:" + ack.err);
                } else {
                    resolve("ok");
                }
            });
            // Safety timeout for the ack
            setTimeout(function () { resolve("ok"); }, 5000);
        });
    }

    /**
     * gun.get(path).on(cb) — subscribe to real-time changes.
     * The Rust side passes a callback ID; JS stores the subscription
     * and writes incoming data into a queue that Rust can poll.
     * @param {string} pathJson - JSON array of keys
     * @returns {number} subscription ID
     */
    function on(pathJson) {
        const path = JSON.parse(pathJson);
        const subId = _nextSubId++;
        const queue = [];
        _subscriptions[subId] = { queue: queue, ref: null };

        const gunRef = _ref(path);
        _subscriptions[subId].ref = gunRef;

        gunRef.on(function (data, key) {
            queue.push(JSON.stringify({ data: data === undefined ? null : data, key: key }));
        });

        return subId;
    }

    /**
     * Poll queued updates for a given subscription.
     * Returns a JSON array of {data, key} objects, or "[]" if nothing new.
     * @param {number} subId
     * @returns {string} JSON array
     */
    function poll(subId) {
        const sub = _subscriptions[subId];
        if (!sub) return "[]";
        const items = sub.queue.splice(0);
        return "[" + items.join(",") + "]";
    }

    /**
     * gun.get(path).off() — unsubscribe.
     * @param {number} subId
     */
    function off(subId) {
        const sub = _subscriptions[subId];
        if (sub && sub.ref) {
            sub.ref.off();
        }
        delete _subscriptions[subId];
    }

    // Expose on window
    window.__gun_bridge = {
        init: init,
        get: get,
        put: put,
        on: on,
        poll: poll,
        off: off,
    };
})();
