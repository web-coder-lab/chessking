import { createContext, useContext, useEffect, useRef, useState, useCallback } from 'react';
import { authApi, socialApi, setCurrentAccessToken } from '../services/api';

const AuthContext = createContext(null);

function ck_setCookie(name, value, days = 30) {
  try {
    const maxAge = days * 24 * 60 * 60;
    document.cookie = `${name}=${encodeURIComponent(value)}; path=/; max-age=${maxAge}; SameSite=Lax`;
  } catch (_) {}
}
function ck_getCookie(name) {
  try {
    const m = document.cookie.match(new RegExp('(?:^|; )' + name + '=([^;]*)'));
    return m ? decodeURIComponent(m[1]) : null;
  } catch (_) {
    return null;
  }
}
function ck_clearCookie(name) {
  try {
    document.cookie = `${name}=; path=/; max-age=0`;
  } catch (_) {}
}
function loadRefreshToken() {
  return localStorage.getItem('ck_refresh_token') || ck_getCookie('ck_refresh_token');
}

/** Server JWT access TTL is 15 min; refresh a bit before that. */
const ACCESS_TOKEN_TTL_MS = 12 * 60 * 1000;
const REFRESH_MARGIN_MS = 60 * 1000;

export function AuthProvider({ children }) {
  const [accessToken, setAccessToken] = useState(null);
  const [refreshToken, setRefreshToken] = useState(() => loadRefreshToken());
  const [isAuthenticated, setIsAuthenticated] = useState(false);
  const [user, setUser] = useState(null);
  /** True until first silent refresh attempt finishes (avoids login flash). */
  const [bootstrapping, setBootstrapping] = useState(() => !!loadRefreshToken());
  const refreshTimer = useRef(null);
  const refreshTokenRef = useRef(refreshToken);
  refreshTokenRef.current = refreshToken;

  const clearSession = useCallback(() => {
    setAccessToken(null);
    setCurrentAccessToken(null);
    setRefreshToken(null);
    refreshTokenRef.current = null;
    localStorage.removeItem('ck_refresh_token');
    ck_clearCookie('ck_refresh_token');
    setIsAuthenticated(false);
    setUser(null);
    if (refreshTimer.current) clearTimeout(refreshTimer.current);
  }, []);

  const refreshUser = useCallback(async () => {
    try {
      const profile = await socialApi.getMyProfile();
      setUser(profile);
      return profile;
    } catch {
      return null;
    }
  }, []);

  const scheduleRefresh = useCallback((currentRefreshToken) => {
    if (refreshTimer.current) clearTimeout(refreshTimer.current);
    if (!currentRefreshToken) return;
    refreshTimer.current = setTimeout(async () => {
      try {
        const tokens = await authApi.refresh(currentRefreshToken);
        setAccessToken(tokens.access_token);
        setCurrentAccessToken(tokens.access_token);
        setRefreshToken(tokens.refresh_token);
        refreshTokenRef.current = tokens.refresh_token;
        localStorage.setItem('ck_refresh_token', tokens.refresh_token);
        ck_setCookie('ck_refresh_token', tokens.refresh_token, 30);
        setIsAuthenticated(true);
        scheduleRefresh(tokens.refresh_token);
      } catch (e) {
        // Only hard-logout on auth failures — not network blips
        const fatal =
          e?.status === 401 ||
          e?.code === 'unauthorized' ||
          e?.code === 'refresh_token_reuse' ||
          e?.code === 'RefreshTokenReuseDetected';
        if (fatal) {
          clearSession();
        } else {
          // Retry once later (server waking / network)
          refreshTimer.current = setTimeout(() => {
            const rt = refreshTokenRef.current;
            if (rt) scheduleRefresh(rt);
          }, 30_000);
        }
      }
    }, ACCESS_TOKEN_TTL_MS - REFRESH_MARGIN_MS);
  }, [clearSession]);

  const setSession = useCallback((tokens) => {
    setAccessToken(tokens.access_token);
    setCurrentAccessToken(tokens.access_token);
    setRefreshToken(tokens.refresh_token);
    refreshTokenRef.current = tokens.refresh_token;
    localStorage.setItem('ck_refresh_token', tokens.refresh_token);
    ck_setCookie('ck_refresh_token', tokens.refresh_token, 30);
    setIsAuthenticated(true);
    scheduleRefresh(tokens.refresh_token);
    refreshUser();
  }, [refreshUser, scheduleRefresh]);

  // Mount: silent restore
  useEffect(() => {
    const rt = loadRefreshToken();
    if (!rt) {
      setBootstrapping(false);
      return;
    }
    let cancelled = false;
    const safety = setTimeout(() => { if (!cancelled) setBootstrapping(false); }, 10000);
    authApi
      .refresh(rt)
      .then((tokens) => {
        if (!cancelled) setSession(tokens);
      })
      .catch((e) => {
        if (cancelled) return;
        const fatal =
          e?.status === 401 ||
          e?.code === 'unauthorized' ||
          e?.code === 'refresh_token_reuse';
        if (fatal) clearSession();
        // network error: keep cookie, user can retry navigation
      })
      .finally(() => {
        if (!cancelled) { clearTimeout(safety); setBootstrapping(false); }
      });
    return () => {
      cancelled = true; clearTimeout(safety);
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const logout = useCallback(async () => {
    try {
      if (accessToken) await authApi.logout(accessToken);
    } finally {
      clearSession();
    }
  }, [accessToken, clearSession]);

  return (
    <AuthContext.Provider
      value={{
        accessToken,
        isAuthenticated,
        user,
        refreshUser,
        setSession,
        clearSession,
        logout,
        bootstrapping,
      }}
    >
      {children}
    </AuthContext.Provider>
  );
}

export function useAuth() {
  const ctx = useContext(AuthContext);
  if (!ctx) throw new Error('useAuth must be used within AuthProvider');
  return ctx;
}
