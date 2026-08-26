# Auditoria tecnica - Hackaton

Fecha: 2026-08-25

## Resumen ejecutivo

El proyecto tiene una base solida: Rust con SQL parametrizado, bcrypt con coste 13, autorizacion de administrador en servidor, iframe sandbox para las paginas y build de frontend correcto. No esta listo para exponerse fuera de desarrollo sin corregir secretos versionados, CORS permisivo, JWT persistente en navegador y proteccion anti-fuerza-bruta. El login es visualmente consistente en escritorio, pero presenta una colision visible en movil.

Prioridad: SEC-01, SEC-02, SEC-03, SEC-04 y SEC-05; despues PERF-01 y UX-01.

## Alcance y metodo

Revision estatica de Rust/Actix, React/TypeScript, CSS, Docker y configuracion; ejecucion local con PostgreSQL Docker; build, pruebas y auditoria npm; inspeccion del login con Chromium a 929 px y 390 px. No es una prueba de penetracion ni una revision de infraestructura externa.

| Comprobacion | Resultado |
| --- | --- |
| npm run build | Pasa. JS 441.02 kB (135.78 kB gzip); CSS 121.91 kB (18.25 kB gzip). |
| npm audit --omit=dev | 0 vulnerabilidades conocidas en produccion. |
| cargo test -q | Pasa. |
| cargo fmt --check | Falla: hay diferencias de formato en multiples modulos. |
| cargo clippy --all-targets -- -D warnings | Falla: seis avisos tratados como error. |
| cargo audit | No disponible en esta maquina; advisories Rust sin verificar. |

## Seguridad

### SEC-01 - Alta - Secretos de desarrollo versionados

**Ubicacion:** .gitignore:6-7; backend/.env esta rastreado por Git.

**Evidencia:** el comentario declara que el archivo con credenciales se versiona. El archivo contiene configuracion de JWT, base de datos y bootstrap de administrador.

**Impacto:** cualquier persona con acceso al repositorio, clon o historial puede obtener secretos de desarrollo. Si se reutilizan fuera de local, puede facilitar acceso a base de datos, suplantacion de sesiones o administracion.

**Correccion:** sacar backend/.env del indice, ignorarlo, conservar solo .env.example sin valores reales y rotar todos los secretos publicados. Inyectar secretos por el entorno de despliegue.

### SEC-02 - Media - CORS acepta cualquier origen y credenciales

**Ubicacion:** backend/src/main.rs:89.

**Evidencia:** Cors::permissive(). Una preflight local desde https://example.invalid obtuvo access-control-allow-origin para ese origen y access-control-allow-credentials: true.

**Impacto:** amplifica el dano de futuros endpoints autenticados por cookie o errores de cliente. No debe ser el valor por defecto en produccion.

**Correccion:** allowlist por entorno de origenes, metodos y cabeceras; desactivar credenciales si no se usan cookies.

### SEC-03 - Media - JWT de siete dias en localStorage

**Ubicacion:** frontend/src/utils/session.ts:8-31; backend/src/auth/services.rs:25-44; frontend/src/utils/api/client.ts:25-28.

**Evidencia:** la sesion completa, incluido bearer token, se guarda en localStorage; el token dura siete dias y se envia en cada solicitud.

**Impacto:** XSS, una extension maliciosa o un equipo compartido pueden extraer una sesion reutilizable.

**Correccion:** en produccion, cookie HttpOnly/Secure/SameSite con proteccion CSRF y rotacion. Como transicion, token corto en memoria y mecanismo de revocacion.

### SEC-04 - Media - Sin limite visible de intentos de login y registro

**Ubicacion:** backend/src/auth/handlers.rs:20-25; backend/src/auth/services.rs:53-84; backend/src/main.rs:84-118.

**Evidencia:** login y registro llegan al servicio sin middleware de rate limit; solo se montan tracing, logging y CORS.

**Impacto:** facilita fuerza bruta, enumeracion operativa y abuso de registro. bcrypt ayuda, pero no sustituye rate limiting.

**Correccion:** rate limit por IP e identidad, backoff, limites de registro y registro de intentos fallidos. Mantener error generico de autenticacion.

### SEC-05 - Media - Sanitizador HTML por subcadenas para contenido renderizado

**Ubicacion:** backend/src/internet/sanitize.rs:67-74; frontend/src/components/internet/SiteFrame.tsx:113-121; frontend/src/components/internet/siteRuntime.ts:121-129.

**Evidencia:** bloquea src= y url( por texto literal, mientras HTML/CSS admiten variantes como src = o url (...). El HTML se entrega por srcDoc y el runtime usa innerHTML al repetir plantillas.

**Impacto:** se puede romper la regla de no cargar recursos externos y convertir una pagina del juego en seguimiento. El sandbox reduce alcance, pero un filtro textual no sustituye un parser.

**Correccion:** sanitizador HTML basado en parser y allowlist, politica CSS separada y pruebas de bypass. Mantener allow-scripts sin allow-same-origin.

### SEC-06 - Baja - Cabeceras defensivas no visibles

**Ubicacion:** frontend/index.html:1-11 y respuesta local de Vite.

**Evidencia:** no hay CSP, frame-ancestors, nosniff, Referrer-Policy o Permissions-Policy en el proyecto; la respuesta local solo incluye cabeceras de Vite.

**Impacto:** falta defensa en profundidad. El proxy de produccion podria resolverlo, pero no es verificable aqui.

**Correccion:** definir y comprobar cabeceras en proxy/CDN. Empezar CSP report-only; no introducir unsafe-inline ni unsafe-eval.

## Rendimiento y escalabilidad

### PERF-01 - Media - Bundle inicial unico

**Evidencia:** build actual: 441.02 kB JS y 121.91 kB CSS. frontend/src/main.tsx:6-23 importa App y todas las hojas de estilo de forma estatica.

**Recomendacion:** dividir rutas/panel admin con React.lazy, cargar estilos por area cuando sea viable y fijar presupuesto de bundle. Mantener login fuera de modulos pesados.

### PERF-02 - Media a futuro - El panel descarga todos los sitios

**Ubicacion:** backend/src/internet/services.rs:699-716.

**Evidencia:** devuelve todas las paginas con contenido y el propio codigo reconoce que necesitara paginacion al crecer.

**Recomendacion:** listado de metadatos y contenido bajo demanda; paginacion, filtro y cache. Aceptable para decenas actuales, no para miles.

### PERF-03 - Baja - Consulta de rol por solicitud autenticada

**Ubicacion:** backend/src/auth/middleware.rs:79-93.

**Evidencia:** el rol de administrador se consulta en PostgreSQL para cada solicitud. Es correcto para revocacion inmediata, pero suma latencia y presion de pool.

**Recomendacion:** medir p95 antes de cambiarlo. Si hace falta, cache corto y acotado o auth_version invalidable.

## Calidad de codigo y buenas practicas

### CODE-01 - Media - Puerta de calidad Rust no limpia

**Evidencia:** rustfmt detecta diferencias en multiples ficheros. Clippy estricto falla por import sin uso, variante/estructuras/campos sin uso y una funcion sin uso.

**Recomendacion:** aplicar rustfmt, eliminar o justificar codigo muerto y exigir fmt, Clippy y tests en CI.

### CODE-02 - Media - Sin lint ni pruebas frontend automatizadas

**Ubicacion:** frontend/package.json.

**Evidencia:** define dev, build y preview; no scripts de lint, pruebas o accesibilidad.

**Recomendacion:** ESLint TypeScript/React, tests de cliente API y flujos Playwright de login/rutas criticas en CI.

### CODE-03 - Baja - Modulos extensos

**Evidencia:** internet.css tiene 1570 lineas, admin.css 1107, hardware.css 980 y utils/api/types.ts 577.

**Recomendacion:** partir por componente o dominio, co-localizar estilos/tipos y conservar barrels por area.

### Aspectos positivos

- Consultas revisadas usan .bind, reduciendo SQL injection.
- Contrasenas de cuenta usan bcrypt con coste 13.
- Autorizacion admin se aplica en servidor, no solo en UI.
- Iframe de sitios usa sandbox sin allow-same-origin y valida event.source.
- Hay lockfile y npm audit no reporta vulnerabilidades de produccion.

## Estetica, UX y accesibilidad

### UX-01 - Alta - Colision de controles a 390 px

**Evidencia:** output/playwright/login-normal-mobile.png muestra que titulo de ventana y selector Normal/Console se superponen; se reproduce tambien en modo consola.

**Correccion:** bajo 480 px, permitir wrap, ocultar/abreviar titulo o llevar selector a fila propia. Anadir captura movil en CI.

### UX-02 - Media - Consola movil con scroll interno dificil de descubrir

**Evidencia:** output/playwright/login-mobile.png muestra contenido truncado dentro de la ventana, scrollbar fino y datos comprimidos.

**Correccion:** neofetch a una columna en movil, menos campos o ventana que crezca con contenido.

### UX-03 - Baja - Idioma y acabado de entrada

**Ubicacion:** frontend/index.html:2 y respuesta local a /favicon.ico.

**Evidencia:** lang=en no corresponde al texto mayoritariamente en espanol; navegador registra 404 de favicon.

**Correccion:** usar lang=es o localizacion coherente y anadir favicon/manifest basico.

### Valoracion visual

En escritorio, la pantalla de consola tiene identidad fuerte, jerarquia clara y paleta coherente. Los formularios conservan etiquetas asociadas y foco visible. El problema principal es la falta de composicion responsive en la barra de titulo.

## Plan de remediacion

1. Rotar secretos y sacar .env del historial operativo; configurar variables por entorno.
2. Reemplazar CORS permisivo por allowlist y anadir cabeceras de produccion.
3. Disenar sesion segura, rate limit y telemetria de autenticacion.
4. Sustituir filtro HTML por sanitizacion estructurada y pruebas de bypass.
5. Corregir movil, favicon e idioma; automatizar capturas a 390 px.
6. Limpiar rustfmt/Clippy, introducir lint/tests frontend y presupuesto de bundle.
7. Paginar sitios y medir autorizacion antes de optimizar.

