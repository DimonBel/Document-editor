import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
export default defineConfig({
  plugins: [react()],
  server: {
    port: 3000,
    host: '0.0.0.0',
    proxy: {
      '/api':   { target: 'http://gateway:8080', changeOrigin: true },
      '/ws':    { target: 'ws://gateway:8080',  ws: true, changeOrigin: true },
      '/api/cms': { target: 'http://cms:1337',  changeOrigin: true, rewrite: p => p.replace(/^\/api\/cms/, '/api') },
    }
  }
});
