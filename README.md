# Hacker Experience — port Rust + React

El proyecto está separado deliberadamente en dos aplicaciones:

- `backend/`: API JSON de Rust con Actix Web y PostgreSQL.
- `frontend/`: SPA React con Redux Toolkit y rutas que sustituyen las páginas del juego.
- `backend/schema/legacy.sql`: esquema MySQL original, preservado como referencia de migración.
- `backend/migrations/0002_wiki_forum.sql`: wiki y foro propios que sustituyen las instalaciones heredadas de DokuWiki y phpBB.

## Ejecutar con Distrobox

Copia `backend/.env.example` a `backend/.env`, configura `DATABASE_URL` y `JWT_SECRET`; el backend aplica las migraciones PostgreSQL al arrancar. La base local usada por este port es `hacker_experience`.

`backend/.env` **no se versiona**: durante un tiempo sí lo estuvo, para que un clon arrancara solo, y la auditoría del 2026-08-25 lo anotó como SEC-01. Copiar el ejemplo es el paso que sustituye a aquello, y el `JWT_SECRET` conviene generarlo (`openssl rand -base64 32`) en vez de dejar el de muestra. Los secretos del despliegue son otros y viven en `deploy/.env`, fuera del repositorio.

```sh
distrobox enter rust-dev -- bash -lc 'cd /home/raul/Projects/hackaton && cargo run -p legacy-api'
distrobox enter web-dev -- bash -lc 'cd /home/raul/Projects/hackaton/frontend && npm install && npm run dev'
```

La API escucha en `127.0.0.1:8080` y Vite en `127.0.0.1:5173`. El proxy de Vite dirige `/api` a Actix Web.

## Worker de juego

El reemplazo Rust de los cronjobs se ejecuta separado del API. Procesa expiración de premium, ranking y la instantánea de estadísticas; en producción se deja en ejecución, y `--once` permite ejecutarlo desde un scheduler externo.

```sh
distrobox enter rust-dev -- bash -lc 'cd /home/raul/Projects/hackaton && cargo run -p legacy-api --bin worker'
distrobox enter rust-dev -- bash -lc 'cd /home/raul/Projects/hackaton && cargo run -p legacy-api --bin worker -- --once'
```
