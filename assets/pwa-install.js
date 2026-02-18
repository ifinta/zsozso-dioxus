let deferredPrompt = null;

window.addEventListener('beforeinstallprompt', (e) => {
    e.preventDefault();
    deferredPrompt = e;
    const btn = document.getElementById('pwa-install-btn');
    if (btn) {
        btn.style.display = 'inline-block';
    }
});

window.triggerPwaInstall = async function() {
    if (!deferredPrompt) {
        return 'not-available';
    }
    deferredPrompt.prompt();
    const { outcome } = await deferredPrompt.userChoice;
    deferredPrompt = null;
    const btn = document.getElementById('pwa-install-btn');
    if (btn) {
        btn.style.display = 'none';
    }
    return outcome;
};

window.isPwaInstalled = function() {
    return window.matchMedia('(display-mode: standalone)').matches
        || window.navigator.standalone === true;
};

window.isIosSafari = function() {
    const ua = window.navigator.userAgent;
    const isIos = /iPad|iPhone|iPod/.test(ua);
    const isWebkit = /WebKit/.test(ua);
    const isChrome = /CriOS/.test(ua);
    return isIos && isWebkit && !isChrome;
};

window.addEventListener('appinstalled', () => {
    deferredPrompt = null;
    const btn = document.getElementById('pwa-install-btn');
    if (btn) {
        btn.style.display = 'none';
    }
});

// iOS: show hint if not installed
document.addEventListener('DOMContentLoaded', () => {
    if (window.isIosSafari() && !window.isPwaInstalled()) {
        const hint = document.getElementById('ios-install-hint');
        if (hint) hint.style.display = 'block';
    }
});
