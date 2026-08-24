/**
 * Phase 1–2 Android shell — Capacitor when inside APK.
 */
export async function initNativeShell() {
  try {
    const { Capacitor } = await import('@capacitor/core');
    if (!Capacitor.isNativePlatform()) return;

    document.documentElement.classList.add('ck-native');
    document.body.classList.add('ck-native');

    try {
      const { StatusBar, Style } = await import('@capacitor/status-bar');
      await StatusBar.setStyle({ style: Style.Dark });
      await StatusBar.setBackgroundColor({ color: '#0F1115' });
      try {
        await StatusBar.setOverlaysWebView({ overlay: false });
      } catch (_) {}
    } catch (_) {}

    try {
      const { SplashScreen } = await import('@capacitor/splash-screen');
      // Keep brand splash briefly, then hide when React is up
      setTimeout(() => {
        SplashScreen.hide().catch(() => {});
      }, 400);
    } catch (_) {}

    try {
      const { App } = await import('@capacitor/app');
      App.addListener('backButton', ({ canGoBack }) => {
        if (canGoBack) {
          window.history.back();
        } else {
          App.exitApp();
        }
      });
      // Deep link / app URL open
      App.addListener('appUrlOpen', ({ url }) => {
        try {
          const u = new URL(url);
          if (u.pathname && u.pathname !== '/') {
            window.location.hash = '';
            window.history.replaceState(null, '', u.pathname + u.search);
            window.dispatchEvent(new PopStateEvent('popstate'));
          }
        } catch (_) {}
      });
    } catch (_) {}
  } catch (_) {
    // Web
  }
}
