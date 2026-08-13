import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

// Dev-server config tuned for running behind a tunnel/proxy/cloud IDE
// (ngrok, Replit, Codespaces, Gitpod, etc.), not just plain localhost:
// - host 0.0.0.0 binds every network interface, not just 127.0.0.1, so
//   the dev server is reachable from other devices / the outside tunnel
//   at all.
// - allowedHosts: true disables Vite 5's Host-header allowlist, which
//   otherwise rejects requests whose Host doesn't match localhost/the
//   configured host - the most common cause of a blank page or
//   "Blocked request" error when opening the app through any forwarded
//   URL that isn't literally localhost.
// - hmr.clientPort lets the browser's hot-reload websocket connect back
//   through a proxy that terminates on a different external port than
//   the one Vite listens on internally (common on tunnels/cloud IDEs).
//   Falls back to the same port when nothing overrides it.
export default defineConfig({
  plugins: [react()],
  server: {
    host: '0.0.0.0',
    port: 5173,
    strictPort: false,
    allowedHosts: true,
    cors: true,
    hmr: {
      clientPort: process.env.VITE_HMR_CLIENT_PORT ? Number(process.env.VITE_HMR_CLIENT_PORT) : undefined,
    },
  },
  preview: {
    host: '0.0.0.0',
    port: 4173,
    strictPort: false,
    allowedHosts: true,
  },
});
