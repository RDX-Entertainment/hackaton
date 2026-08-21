# Hacker Experience — port Rust + React

El proyecto está separado deliberadamente en dos aplicaciones:

- `backend/`: API JSON de Rust con Actix Web y PostgreSQL.
- `frontend/`: SPA React con Redux Toolkit y rutas que sustituyen las páginas del juego.
- `backend/schema/legacy.sql`: esquema MySQL original, preservado como referencia de migración.
- `backend/migrations/0002_wiki_forum.sql`: wiki y foro propios que sustituyen las instalaciones heredadas de DokuWiki y phpBB.

## Ejecutar con Distrobox

Copia `backend/.env.example` a `backend/.env`, configura `DATABASE_URL` y `JWT_SECRET`; el backend aplica las migraciones PostgreSQL al arrancar. La base local usada por este port es `hacker_experience`.

```sh
distrobox enter rust-dev -- bash -lc 'cd /home/raul/Projects/hackaton && cargo run -p legacy-api'
distrobox enter web-dev -- bash -lc 'cd /home/raul/Projects/hackaton/frontend && npm install && npm run dev'
```

La API escucha en `127.0.0.1:8080` y Vite en `127.0.0.1:5173`. El proxy de Vite dirige `/api` a Actix Web.
