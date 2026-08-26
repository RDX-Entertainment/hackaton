import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

const API_PORT = 8080;
const DEV_PORT = 5173;
// En el host los dos procesos comparten loopback; dentro del contenedor, loopback es el
// propio Vite. Docker aporta API_PROXY_TARGET=host.docker.internal para alcanzar la API.
const API_PROXY_TARGET = process.env.API_PROXY_TARGET ?? `http://127.0.0.1:${API_PORT}`;

export default defineConfig({
  plugins: [react()],
  server: {
    port: DEV_PORT,
    // The API is a separate process; the proxy keeps the browser on one origin.
    proxy: {
      '/api': API_PROXY_TARGET,
    },
  },
});
