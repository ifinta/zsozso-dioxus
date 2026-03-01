// In-app log ring buffer — keeps the last 100 entries.
// Both app-level console.log/error and service worker messages feed into this.
(function() {
    const MAX_ENTRIES = 100;
    const buffer = [];

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

    // Listen for log messages forwarded from the service worker
    if ('serviceWorker' in navigator) {
        navigator.serviceWorker.addEventListener('message', function(event) {
            if (event.data && event.data.type === '__ZSOZSO_SW_LOG') {
                push('SW', [event.data.text]);
            }
        });
    }

    // Public API for Rust to read
    window.__zsozso_log = {
        // Returns all buffered lines as a single newline-separated string
        get: function() { return buffer.join('\n'); },
        // Returns the current count
        count: function() { return buffer.length; },
        // Clear the buffer
        clear: function() { buffer.length = 0; },
    };
})();
