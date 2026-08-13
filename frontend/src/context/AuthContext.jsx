import { createContext, useContext, useEffect, useRef, useState, useCallback } from 'react';
import { authApi, socialApi, setCurrentAccessToken } from '../services/api';

const AuthContext = createContext(null);

const ACCESS_TOKEN_TTL_MS = 5 * 60 * 1000;
const REFRESH_MARGIN_MS = 30 * 1000; // refresh 30s before expiry

export function AuthProvider({ children }) {
  const [accessToken, setAccessToken] = useState(null);
  const [refreshToken, setRefreshToken] = useState(() => localStorage.getItem('ck_refresh_token'));
  const [isAuthenticated, setIsAuthenticated] = useState(false);
  const [user, setUser] = useState(null);
  const refreshTimer = useRef(null);

  // Every page that shows the logged-in person's own username, avatar,
  // rating, or coin balance reads it from here - not from the tokens.
  // Call after any action that could change the backend's copy of the
  // user's own profile (e.g. a shop purchase changing coin_balance).
  const refreshUser = useCallback(async () => {
    try {
      const profile = await socialApi.getMyProfile();
      setUser(profile);
      return profile;
    } catch (e) {
      return null;
    }
  }, []);

  const setSession = useCallback((tokens) => {
    setAccessToken(tokens.access_token);
    setCurrentAccessToken(tokens.access_token);
    setRefreshToken(tokens.refresh_token);
    localStorage.setItem('ck_refresh_token', tokens.refresh_token);
    setIsAuthenticated(true);
    scheduleRefresh(tokens.refresh_token);
    refreshUser();
  }, [refreshUser]);

  const clearSession = useCallback(() => {
    setAccessToken(null);
    setCurrentAccessToken(null);
    setRefreshToken(null);
    localStorage.removeItem('ck_refresh_token');
    setIsAuthenticated(false);
    setUser(null);
    if (refreshTimer.current) clearTimeout(refreshTimer.current);
  }, []);

  // Doc 3 §7: "Silent refresh: frontend automatically requests a new
  // access token shortly before the 5-minute expiry, no page reload, no
  // user-visible interruption."
  const scheduleRefresh = useCallback((currentRefreshToken) => {
    if (refreshTimer.current) clearTimeout(refreshTimer.current);
    refreshTimer.current = setTimeout(async () => {
      try {
        const tokens = await authApi.refresh(currentRefreshToken);
        setSession(tokens);
      } catch (e) {
        // §7 reuse detection / expiry → force redirect to Login (§8)
        clearSession();
      }
    }, ACCESS_TOKEN_TTL_MS - REFRESH_MARGIN_MS);
  }, [clearSession, setSession]);

  // On mount, attempt silent refresh if a refresh token was persisted.
  useEffect(() => {
    if (refreshToken && !accessToken) {
      authApi.refresh(refreshToken).then(setSession).catch(clearSession);
    }
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
    <AuthContext.Provider value={{ accessToken, isAuthenticated, user, refreshUser, setSession, clearSession, logout }}>
      {children}
    </AuthContext.Provider>
  );
}

export function useAuth() {
  const ctx = useContext(AuthContext);
  if (!ctx) throw new Error('useAuth must be used within AuthProvider');
  return ctx;
}
