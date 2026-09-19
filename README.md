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

## El primer paso

`GET /api/onboarding` dice en qué punto del circuito está el jugador —módulos, diseño,
compilación, carga en memoria, primera intrusión— deducido del mundo mientras aprende.
La graduación queda guardada permanentemente: borrar programas no vuelve a activar el tutorial.
Llegar por otro camino también cuenta. Devuelve códigos y
no frases, como los errores, porque la lengua la pone el cliente. Incluye las direcciones del
campo de tiro, que el alta crea desde la 0056 y hasta ahora no enseñaba ninguna pantalla.

La portada, Arsenal, Software e Investigación lo consumen: la primera para decir qué toca, y
las otras tres para pintar el mismo riel de cinco pasos.

La guía de misión acompaña al jugador en todas las pantallas: se fija junto al control
señalado por el panel (`focus_view` y `focus_anchor`), reconoce los campos y pestañas
previos, y se coloca al pie en móvil. Puede plegarse sin saltarse las reglas del servidor.
El botón, campo o pestaña que toca usar lleva un borde verde con glow suave, sin oscurecer
la pantalla. Si hace falta cambiar de sección, el brillo señala primero el menú y después
la pestaña; los controles deshabilitados no brillan y se respeta el movimiento reducido.
Los objetivos avanzan por hechos; compilar muestra una espera y el primer acceso confirma
la misión. Sus textos están en los tres catálogos de idioma, con `hint` del panel como
texto personalizado. El armazón comparte una sola lectura de onboarding, que se refresca
al completar una petición de escritura y cada cuatro segundos para los trabajos del worker.

Para cerrar el primer acceso, un jugador con certificación 1 puede usar fuerza bruta
contra **su propio campo de tiro**. La certificación 2 sigue siendo necesaria en el mundo
abierto: exigirla también en la práctica impedía ganar la certificación que da ese acceso.

## Bucle de encargos

Encargos ya cierra el circuito jugable después del tutorial. El jugador acepta una oferta
de su nivel de certificación, abre el objetivo desde el propio encargo y obtiene acceso con
fuerza bruta o una intrusión táctica. El worker completa el contrato una sola vez, deposita
la recompensa en la cuenta principal, actualiza las estadísticas y entrega un módulo de
software aleatorio. Ese módulo puede fusionarse o equiparse en el Taller para preparar el
siguiente ataque. No existe ya la entrega gratuita provisional de módulos.

La regresión integral se ejecuta únicamente contra una API y PostgreSQL aislados:
`HACKATON_TEST_BASE=http://127.0.0.1:18091 HACKATON_TEST_DB=hackaton-gameplay-test-20260919 python3 scripts/test-mission-loop.py`.
Comprueba aceptación, acceso, pago, estadísticas y loot en una sola vuelta.

## Hardware y cuenta principal

El tutorial se completa una sola vez: `users.tutorial_completed_at` conserva la graduación
aunque se borren programas, se descarguen de RAM o cambien los pasos del administrador.
Los eventos del juego registran el final sin depender del siguiente sondeo del navegador;
tras completarlo, el cliente deja de sondear el tutorial. Las pantallas administrativas
muestran los errores de carga reales y permiten reintentar sin confundirlos con permisos.
Prueba de regresión: `HACKATON_TEST_BASE=http://127.0.0.1:18091 node scripts/test-tutorial-once.mjs`
(solo contra una API aislada con el administrador de pruebas).

La migración 0059 divide los DIMM en grupos iguales, uno por socket de CPU: una placa
de cuatro sockets tiene al menos cuatro ranuras RAM. Los módulos se reparten entre las
CPU instaladas; con dos CPU deben ir en múltiplos de dos y con cuatro, de cuatro.
No se exige que los módulos tengan la misma capacidad. Las ranuras de sockets sin CPU
no aumentan la memoria utilizable del equipo.

Sin placa, CPU, RAM o disco, o con RAM desequilibrada, el equipo queda **Desactivado**.
Se conservan las piezas y los ficheros; no puede iniciar tareas y el worker espera a que
el origen vuelva a estar operativo. La tarjeta de red sigue siendo opcional para un
equipo interno. «Agregar compatibles» muestra piezas del cajón y de la tienda; comprar
e instalar es una única transacción que se revierte si no encaja o no hay saldo.
Los NPC antiguos conservan su hardware virtual y sus capacidades. Al instalarles una pieza
real pasan definitivamente a las mismas reglas de montaje; el campo de práctica sigue activo.

Las gamas son doméstica, gamer, workstation y server. Las piezas `gaming` pasan a
`gamer` y las `industrial` a `workstation`, sin reemplazar las instancias existentes.
El switch es un nodo LAN de ocho puertos, cuesta $1,000 y no consume plaza de cómputo.
Se compra en Hardware → Tienda → Extras y aparece directamente en el mapa de Red.
Hardware permite nombrar otra máquina principal: debe ser un ordenador activo con NIC
y dirección pública. La cuenta pasa a usar la IP del nuevo principal; las IP y los archivos
de cada equipo se conservan. Se protege la última NIC del principal tanto al retirarla
como al moverla, y los cambios concurrentes mantienen una única máquina principal.
No admite programas ni componentes de ordenador. Los cables se pueden tender a través
de switches y el límite de puertos también se comprueba ante solicitudes concurrentes.

La primera cuenta bancaria propia es la principal. Se cambia desde Banco y queda
seleccionada al pagar. Su titular puede consultar y transferir desde ella sin reintroducir
el PIN; las demás cuentas mantienen sus credenciales. El acceso automático nunca autoriza
a otro jugador ni expone el PIN. Cambiar la principal invalida las órdenes nuevas que
todavía indiquen la anterior como principal.

Regresión de API (solo base aislada, con administrador de prueba `hardwaretest`):
`HACKATON_TEST_BASE=http://127.0.0.1:18091 node scripts/test-hardware-bank.mjs`.
El script crea jugadores, piezas, equipos y cuentas de prueba; no ejecutarlo en producción.

## Fuerza bruta y antivirus

Fuerza bruta en Internet usa el perfil del crackeador instalado: eficacia contra
resistencia, penetración contra blindaje y sigilo contra detección. La versión ya no
decide si entra. El hardware y los atributos determinan el tiempo; las tiradas y los
perfiles se congelan al iniciar el proceso. Ganar una sola fase no concede acceso.
Las tareas antiguas de fuerza bruta ya encoladas conservan su resolución anterior.

Lab muestra únicamente los atributos relevantes/secundarios declarados por cada
capacidad, además de estabilidad y recursos. Los módulos sin beneficios para esa
capacidad no se ofrecen en el selector. Esto se aplica a creación, diseños guardados,
ajustes y lanzamiento.

La migración 0063 reúne detector y eliminador en **Antivirus**, conservando los diseños,
módulos y estadísticas existentes. Desde **Lab → Lanzar**, un antivirus instalado puede:

- Buscar virus: detección contra sigilo; los hallazgos se hacen visibles en Discos.
- Eliminar detectados: limpieza contra persistencia; solo afecta a virus visibles en
  el equipo del antivirus. No elimina los originales de malware creados en Lab.

Son acciones independientes del mismo programa; detección alta no implica limpieza
alta. La limpieza elimina el fichero y su entrada en memoria, dentro de una transacción.
No se actúa sobre otras máquinas, aunque sean del mismo dueño o cluster.
`POST /api/software-system/antivirus` recibe `software_id` y `mode` (`scan` o `clean`)
y devuelve `process_id` y `seconds`; el resultado se consulta en Procesos.

Pruebas: `cargo test -p legacy-api`, `npm run build` en frontend y
`HACKATON_TEST_BASE=http://127.0.0.1:18091 python3 scripts/test-antivirus-bruteforce.py`.
Esta última usa exclusivamente el contenedor PostgreSQL aislado de hardware y los
fixtures `lmmtxiy3mn` / `lnmtxiy3mn`, con dos antivirus creados antes de la migración;
nunca debe ejecutarse contra producción.

## Worker de juego

El reemplazo Rust de los cronjobs se ejecuta separado del API. Procesa expiración de premium, ranking y la instantánea de estadísticas; en producción se deja en ejecución, y `--once` permite ejecutarlo desde un scheduler externo.

```sh
distrobox enter rust-dev -- bash -lc 'cd /home/raul/Projects/hackaton && cargo run -p legacy-api --bin worker'
distrobox enter rust-dev -- bash -lc 'cd /home/raul/Projects/hackaton && cargo run -p legacy-api --bin worker -- --once'
```
