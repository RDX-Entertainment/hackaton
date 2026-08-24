import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

const API_PORT = 8080;
const DEV_PORT = 5173;

export default defineConfig({
  plugins: [react()],
  server: {
    port: DEV_PORT,
    // The API is a separate process; the proxy keeps the browser on one origin.
    proxy: {
      '/api': `http://127.0.0.1:${API_PORT}`,
    },
  },
});
