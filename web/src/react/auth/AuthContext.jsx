import React, { createContext, useContext, useMemo, useState } from 'react';

const AuthContext = createContext(null);

export function AuthProvider({ children }) {
  const [token, setTokenState] = useState(() => localStorage.getItem('run-agent.access_token') || localStorage.getItem('run-agent.token'));

  const value = useMemo(
    () => ({
      token,
      setToken: (t) => {
        if (t) {
          localStorage.setItem('run-agent.access_token', t);
          localStorage.removeItem('run-agent.token');
        } else {
          localStorage.removeItem('run-agent.access_token');
          localStorage.removeItem('run-agent.token');
        }
        setTokenState(t);
      },
      logout: () => {
        localStorage.removeItem('run-agent.access_token');
        localStorage.removeItem('run-agent.token');
        setTokenState(null);
      },
    }),
    [token]
  );

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
}

export function useAuth() {
  const ctx = useContext(AuthContext);
  if (!ctx) throw new Error('useAuth must be used within AuthProvider');
  return ctx;
}
