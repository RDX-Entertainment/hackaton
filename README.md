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

## Software modular

Arsenal permite previsualizar un diseño con hasta tres módulos, guardarlo y compilarlo.
La vista previa y la compilación usan el mismo cálculo del servidor. Los módulos deben estar
desbloqueados y ser compatibles; no se admiten duplicados. Diseño y módulos se guardan en una
transacción. Solo puede haber una compilación activa por jugador, también ante peticiones simultáneas.

El worker materializa el fichero con los atributos congelados al encolar, en un disco con espacio.
Software permite cargarlo en memoria y moverlo entre equipos. Arsenal muestra su máquina y sus
recursos efectivos —incluido el cluster—, y permite elegir la copia exacta que inicia una operación.
Las operaciones de intrusión e infección apuntan a la máquina principal del rival; la defensa se
busca en ese equipo o en su cluster. Las acciones tácticas se usan desde Accesos.

El diseño de research sigue usando los módulos iniciales desbloqueados, como en el modelo actual.
El laboratorio conserva sus ajustes manuales para experimentar con los atributos.

Validación: `cargo test -p legacy-api` y `npm run build` en `frontend/`. Para probar contra una
instancia **aislada**, con API, worker y `PROCESS_TEST_SECONDS=3`, ejecutar
`HACKATON_TEST_BASE=http://127.0.0.1:18089 node scripts/test-software.mjs`.
La prueba crea dos jugadores y comprueba diseño, concurrencia, compilación, instalación,
propiedad y resolución. `HACKATON_TEST_SSH` añade la regresión de defensa por máquina y requiere
el contenedor de pruebas `hackaton-software-test-postgres-1`; no debe apuntar a producción.

## Worker de juego

El reemplazo Rust de los cronjobs se ejecuta separado del API. Procesa expiración de premium, ranking y la instantánea de estadísticas; en producción se deja en ejecución, y `--once` permite ejecutarlo desde un scheduler externo.

```sh
distrobox enter rust-dev -- bash -lc 'cd /home/raul/Projects/hackaton && cargo run -p legacy-api --bin worker'
distrobox enter rust-dev -- bash -lc 'cd /home/raul/Projects/hackaton && cargo run -p legacy-api --bin worker -- --once'
```
