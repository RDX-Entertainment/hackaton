# Hacker Experience — port Rust + React

El proyecto está separado deliberadamente en dos aplicaciones:

- `backend/`: API JSON de Rust con Actix Web y acceso directo al esquema MySQL heredado.
- `frontend/`: SPA React con Redux Toolkit y rutas que sustituyen las páginas del juego.
- `backend/schema/legacy.sql`: esquema MySQL original, preservado para importar datos existentes.

## Ejecutar con Distrobox

Importa el esquema en una instancia MySQL/MariaDB compatible y copia `backend/.env.example` a `backend/.env`, configurando `DATABASE_URL` y `JWT_SECRET`.

```sh
distrobox enter rust-dev -- bash -lc 'cd /home/raul/Projects/hackaton && cargo run -p legacy-api'
distrobox enter web-dev -- bash -lc 'cd /home/raul/Projects/hackaton/frontend && npm install && npm run dev'
```

La API escucha en `127.0.0.1:8080` y Vite en `127.0.0.1:5173`. El proxy de Vite dirige `/api` a Actix Web.
