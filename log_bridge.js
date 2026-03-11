// In-app log ring buffer — keeps the last 100 entries.
// Both app-level console.log/error and service worker messages feed into this.
(function() {
    const MAX_ENTRIES = 100;
    const buffer = [];
    const seenSwLines = new Set();  // dedup SW lines (push + pull)

    function ts() {
        const d = new Date();
        return d.toLocaleTimeString('en-GB', { hour12: false }) + '.' +
            String(d.getMilliseconds()).padStart(3, '0');
    }

    function push(level, args) {
        const text = Array.from(args).map(a => {
            if (typeof a === 'string') return a;
            try { return JSON.stringify(a); } catch(_) { return String(a); }
        }).join(' ');
        const entry = ts() + ' [' + level + '] ' + text;
        buffer.push(entry);
        if (buffer.length > MAX_ENTRIES) buffer.shift();
    }

    function pushSwLine(line) {
        // Deduplicate: the same SW line might arrive via push (postMessage)
        // and again via pull (GET_LOGS). Use the line text as key.
        if (seenSwLines.has(line)) return;
        seenSwLines.add(line);
        // Keep the set bounded
        if (seenSwLines.size > MAX_ENTRIES * 2) {
            const it = seenSwLines.values();
            for (let i = 0; i < MAX_ENTRIES; i++) it.next();
            // rebuild
            const keep = new Set();
            for (const v of seenSwLines) { if (keep.size < MAX_ENTRIES) keep.add(v); }
            seenSwLines.clear();
            for (const v of keep) seenSwLines.add(v);
        }
        buffer.push(line);
        if (buffer.length > MAX_ENTRIES) buffer.shift();
    }

    // Intercept console.log and console.error
    const origLog = console.log.bind(console);
    const origErr = console.error.bind(console);

    console.log = function() {
        push('LOG', arguments);
        origLog.apply(console, arguments);
    };

    console.error = function() {
        push('ERR', arguments);
        origErr.apply(console, arguments);
    };

    // Listen for log messages pushed from the service worker
    if ('serviceWorker' in navigator) {
        navigator.serviceWorker.addEventListener('message', function(event) {
            if (event.data && event.data.type === '__ZSOZSO_SW_LOG') {
                pushSwLine(event.data.text);
            }
        });
    }

    // Pull: actively request buffered logs from the SW.
    // This is essential for standalone PWA (Home Screen) on iOS where
    // the push mechanism (SW→client postMessage) can be unreliable.
    function pullSwLogs() {
        if (!navigator.serviceWorker || !navigator.serviceWorker.controller) return;
        try {
            var ch = new MessageChannel();
            ch.port1.onmessage = function(e) {
                if (e.data && Array.isArray(e.data.logs)) {
                    e.data.logs.forEach(function(line) { pushSwLine(line); });
                }
            };
            navigator.serviceWorker.controller.postMessage(
                { type: 'GET_LOGS' }, [ch.port2]
            );
        } catch(_) {}
    }

    // Pull once the SW is ready. Don't poll on an interval —
    // iOS Safari kills the SW after ~30s of inactivity, and constant
    // polling would either keep it alive unnecessarily or fail silently.
    // Instead, pull on demand when the Log tab reads the buffer.
    if ('serviceWorker' in navigator) {
        navigator.serviceWorker.ready.then(function() {
            pullSwLogs();
        });
    }

    // Upload logs to the server (POST /app/upload_log)
    function uploadLogs() {
        if (buffer.length === 0) return Promise.resolve('EMPTY');
        var body = buffer.join('\n');
        return fetch('/app/upload_log', {
            method: 'POST',
            headers: { 'Content-Type': 'text/plain; charset=utf-8' },
            body: body
        }).then(function(resp) {
            if (resp.ok) return 'OK';
            return 'HTTP_' + resp.status;
        }).catch(function(err) {
            return 'ERR:' + (err.message || err);
        });
    }

    // Public API for Rust to read
    window.__zsozso_log = {
        // Returns all buffered lines as a single newline-separated string.
        // Also triggers a pull from the SW so next read has fresh data.
        get: function() {
            pullSwLogs();
            return buffer.join('\n');
        },
        // Returns the current count
        count: function() { return buffer.length; },
        // Clear both local buffer and the SW-side buffer
        clear: function() {
            buffer.length = 0;
            seenSwLines.clear();
            // Tell the service worker to clear its buffer too
            if (navigator.serviceWorker && navigator.serviceWorker.controller) {
                try {
                    navigator.serviceWorker.controller.postMessage({ type: 'CLEAR_LOGS' });
                } catch(_) {}
            }
        },
        // Upload the current log buffer to the server.
        // Returns a Promise that resolves to 'OK', 'EMPTY', or an error string.
        upload: function() {
            return uploadLogs();
        },
        // Extract the version (CACHE_NAME) from SW log lines.
        // SW log entries contain the CACHE_NAME string, e.g. "12:34:56.789 zsozso-v2 [SW] ..."
        version: function() {
            // Fallback: check the logs as you do now
            for (var i = buffer.length - 1; i >= 0; i--) { // Check newest first
                var m = buffer[i].match(/\b(zsozso-v[\w.]+)\b/);
                if (m) return m[1];
            }
            return 'detecting...'; 
        }
    };
})();
