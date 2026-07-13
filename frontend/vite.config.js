import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

// In the Vite *dev* proxy, both `/api` and `/ws` target the in-network
// `gateway:8080` over plain HTTP / WS. There is no transport
// encryption at this layer; TLS terminates either at a fronting
// reverse proxy or, in production, at the gateway's HTTPS listener.
// We opt in to `wss` automatically when the upstream is HTTPS.
const gatewayBase = process.env.VITE_GATEWAY_BASE_URL || 'http://gateway:8080';
const gatewayWs   = gatewayBase.startsWith('https')
    ? gatewayBase.replace(/^https/, 'wss')
    : gatewayBase.replace(/^http/,  'ws');

export default defineConfig({
  plugins: [react()],
  server: {
    port: 3000,
    host: '0.0.0.0',
    proxy: {
      '/api':   { target: gatewayBase, changeOrigin: true },
      // nosemgrep: javascript.lang.security.detect-insecure-websocket
      '/ws':    { target: gatewayWs, ws: true, changeOrigin: true },
      '/api/cms': { target: 'http://cms:1337',  changeOrigin: true, rewrite: p => p.replace(/^\/api\/cms/, '/api') },
    }
  }
});
