import type { InternalAxiosRequestConfig } from "axios";
import {
  PropsWithChildren,
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useRef,
  useState
} from "react";

import {
  AuthUser,
  LoginPayload,
  LoginResponse,
  login as loginRequest,
  logout as logoutRequest,
  refresh as refreshRequest
} from "../lib/api/auth";
import api from "../lib/api/client";

const STORAGE_KEYS = {
  accessToken: "erp.accessToken",
  refreshToken: "erp.refreshToken",
  user: "erp.user",
  expiresAt: "erp.tokenExpiresAt"
} as const;

type AuthContextValue = {
  user: AuthUser | null;
  accessToken: string | null;
  refreshToken: string | null;
  isAuthenticated: boolean;
  isLoading: boolean;
  signIn: (payload: LoginPayload) => Promise<void>;
  signOut: () => Promise<void>;
};

const AuthContext = createContext<AuthContextValue | undefined>(undefined);

type RetryableRequestConfig = InternalAxiosRequestConfig & { _retry?: boolean };

function persistAuth(response: LoginResponse) {
  localStorage.setItem(STORAGE_KEYS.accessToken, response.access_token);
  localStorage.setItem(STORAGE_KEYS.refreshToken, response.refresh_token);
  localStorage.setItem(STORAGE_KEYS.expiresAt, String(Date.now() + response.expires_in * 1000));
  localStorage.setItem(STORAGE_KEYS.user, JSON.stringify(response.user));
}

function clearPersistedAuth() {
  localStorage.removeItem(STORAGE_KEYS.accessToken);
  localStorage.removeItem(STORAGE_KEYS.refreshToken);
  localStorage.removeItem(STORAGE_KEYS.expiresAt);
  localStorage.removeItem(STORAGE_KEYS.user);
}

function loadPersistedUser(): AuthUser | null {
  const raw = localStorage.getItem(STORAGE_KEYS.user);
  if (!raw) return null;
  try {
    return JSON.parse(raw) as AuthUser;
  } catch (error) {
    console.warn("Failed to parse stored user", error);
    return null;
  }
}

export function AuthProvider({ children }: PropsWithChildren) {
  const [user, setUser] = useState<AuthUser | null>(null);
  const [accessToken, setAccessToken] = useState<string | null>(null);
  const [refreshToken, setRefreshToken] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const refreshPromiseRef = useRef<Promise<LoginResponse> | null>(null);

  const applyAuthResponse = useCallback((response: LoginResponse) => {
    setUser(response.user);
    setAccessToken(response.access_token);
    setRefreshToken(response.refresh_token);
    persistAuth(response);
  }, []);

  useEffect(() => {
    const storedAccess = localStorage.getItem(STORAGE_KEYS.accessToken);
    const storedRefresh = localStorage.getItem(STORAGE_KEYS.refreshToken);
    const storedExpiresAt = Number(localStorage.getItem(STORAGE_KEYS.expiresAt) ?? 0);
    const storedUser = loadPersistedUser();

    if (storedAccess && storedRefresh && storedUser) {
      if (storedExpiresAt && storedExpiresAt < Date.now()) {
        clearPersistedAuth();
      } else {
        setAccessToken(storedAccess);
        setRefreshToken(storedRefresh);
        setUser(storedUser);
      }
    }

    setIsLoading(false);
  }, []);

  const signIn = useCallback(async (payload: LoginPayload) => {
    setIsLoading(true);
    try {
      const response = await loginRequest(payload);
      applyAuthResponse(response);
    } finally {
      setIsLoading(false);
    }
  }, [applyAuthResponse]);

  const signOut = useCallback(async () => {
    try {
      await logoutRequest();
    } catch (error) {
      console.warn("Logout request failed", error);
    } finally {
      clearPersistedAuth();
      setUser(null);
      setAccessToken(null);
      setRefreshToken(null);
    }
  }, []);

  useEffect(() => {
    const interceptor = api.interceptors.response.use(
      (response) => response,
      async (error) => {
        const status = error?.response?.status;
        const originalRequest = error?.config as RetryableRequestConfig | undefined;

        if (status !== 401 || !originalRequest || originalRequest._retry) {
          if (status === 401 && !refreshToken) {
            await signOut();
          }
          return Promise.reject(error);
        }

        if (!refreshToken) {
          await signOut();
          return Promise.reject(error);
        }

        originalRequest._retry = true;

        if (!refreshPromiseRef.current) {
          refreshPromiseRef.current = refreshRequest(refreshToken)
            .then((response) => {
              applyAuthResponse(response);
              return response;
            })
            .catch(async (refreshError) => {
              await signOut();
              throw refreshError;
            })
            .finally(() => {
              refreshPromiseRef.current = null;
            });
        }

        const refreshed = await refreshPromiseRef.current;
        if (!refreshed) {
          return Promise.reject(error);
        }

        originalRequest.headers = originalRequest.headers ?? {};
        originalRequest.headers.Authorization = `Bearer ${refreshed.access_token}`;
        return api(originalRequest);
      }
    );

    return () => {
      api.interceptors.response.eject(interceptor);
    };
  }, [applyAuthResponse, refreshToken, signOut]);

  const value = useMemo<AuthContextValue>(() => ({
    user,
    accessToken,
    refreshToken,
    isAuthenticated: Boolean(user && accessToken),
    isLoading,
    signIn,
    signOut
  }), [accessToken, isLoading, refreshToken, signIn, signOut, user]);

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
}

export function useAuth() {
  const context = useContext(AuthContext);
  if (!context) {
    throw new Error("useAuth must be used within an AuthProvider");
  }
  return context;
}
