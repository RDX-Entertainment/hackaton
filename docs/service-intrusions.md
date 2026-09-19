# Hackeo visual por servicios

En **Internet**, abre una IP del directorio, los marcadores o el historial. Desde **Accesos** también puedes abrir una máquina directamente.

1. **Servicios → Escanear puertos**. Los exploits requieren certificación 3 y un escaneo completado. La primera práctica sigue permitiendo fuerza bruta en nivel 1.
   Para conseguir la herramienta: **Lab → Crear → Escáner de puertos**, guardar diseño, compilar y activar en **Software**. El enlace **Crear escáner de puertos** de Servicios preselecciona la capacidad. Puedes elegir qué escáner activo usar.
2. Compila la capacidad necesaria en **Lab** y activa el programa en **Software**. Los módulos que concedan esa función también son compatibles.
3. Elige el programa y ataca **Web**, **Almacenamiento** o **SSH**. Los atributos efectivos y los recursos de su nodo determinan el combate y su duración; no la versión.
4. Espera el resultado. Si triunfas aparecen las acciones permitidas. Puedes conquistar otra puerta sin perder los permisos anteriores.

| Entrada | Permisos |
| --- | --- |
| Exploit web | Leer y modificar el sitio |
| Exploit FTP | Leer, descargar, subir y borrar archivos |
| Exploit SSH | Archivos, ejecución y registros |
| Fuerza bruta | Acceso completo a las tres puertas |

Los tres servicios son puertas de la **simulación**, no conexiones HTTP/FTP/SSH reales ni un catálogo configurable de daemons. La máquina debe estar operativa para iniciar la intrusión. Ejecutar comandos arbitrarios por terminal no forma parte de esta entrega.

El escáner compara eficacia contra blindaje y sigilo contra detección. Sus atributos y recursos determinan el tiempo y la probabilidad; también acepta módulos que concedan `port_scan`. Si falla, no descubre puertas. Si triunfa, guarda el reconocimiento del nodo y permite intentar sus exploits, **sin conceder permisos de acceso**. Los PortScan antiguos conservan compatibilidad.

## Progreso en el lugar de trabajo

La compilación muestra una barra animada junto al botón del diseño; investigar la muestra en la tarjeta del programa. Software también incluye la barra en su aviso de compilación. El porcentaje no llega al 100 % hasta la confirmación del worker. Segundos enteros, pausa visible y sondeo detenido al finalizar o esconder la pestaña. El movimiento solo afecta a la barra, sin volver a animar toda la pantalla.

## Modificar HTML

Con permiso web, abre **editar HTML**. Se muestra el contenido público actual del nodo y su revisión. Escribe HTML/CSS o el documento Webscript de varias páginas y pulsa **Actualizar vista previa**. La vista previa no ejecuta actions ni JavaScript y bloquea recursos externos.

**Publicar modificación** guarda inmediatamente el HTML cuando el acceso activo incluye `write_web`. La intrusión ya concedió ese permiso: publicar no exige otro programa ni una segunda tirada de combate. Al confirmar el guardado, la interfaz recarga **sitio**, incluido su documento Webscript. Si la petición falla, el borrador permanece y el error aparece junto al editor. No modifica otros nodos del dueño. El guardado conserva la versión anterior, un recibo de proceso ya completado y evidencia visible, en una sola transacción. El worker no vuelve a ejecutar ese recibo.

Las actions conservan las concesiones del sitio: cambiar HTML no concede permisos bancarios ni administrativos. Límite de publicación: 16.384 bytes, sin scripts ni recursos externos.

## Verificación

- `scripts/test-service-intrusions.py`: API/worker en localhost aislado; permisos separados, suma sin degradación, revocación, aislamiento por nodo/NPC, publicación y sanitización.
- `scripts/test-service-intrusions.mjs`: navegador en 1440 y 390 px, puertas, editor, vista previa aislada y navegación a almacenamiento/SSH. Puede usar el bundle de producción con toda llamada API desviada al entorno aislado.
- `scripts/test-port-scanner.py`: creación, compilación real tipo 3, activación, escaneo ganado/bloqueado, compatibilidad y evidencia NPC.
- `scripts/test-inline-progress.mjs`: barra dentro de la tarjeta, interpolación, pausa, pestaña oculta y parada de sondeos al terminar, en escritorio y móvil.

Migración: `0065_service_intrusions.sql`. Las páginas atacadas nuevas se guardan en `node_web_content`, con revisiones por proceso en `node_web_revisions`. Las tablas históricas por dueño se conservan, sin copiar sus snapshots antiguos por encima de sitios más recientes.

## Despliegue verificado · 2026-09-13

- API/worker: `hackaton-api:service-intrusions-20260912`, imagen `ab6650591b279c27e8b12631bcec6b654fb289d63a99f8604b3a0a060e590f9b`.
- Web: `hackaton-web:service-intrusions-20260912`, imagen `01344061d8aa83adda980b22e850276820ed8fbb2011d436db5c25174c0230d0`.
- Fuentes compiladas en `/home/raul/hackaton-service-intrusions-20260912.tyR6a6` del servidor.
- Respaldo privado previo: `/home/raul/hackaton-prod/deploy/pre-services-20260912.Vxrpaf/database.dump` (permiso 0600).
- Migración 65 aplicada con éxito. API, worker y web activos, sin reinicios al verificar.

### Actualización: escáner y barras · 2026-09-13

- Migración 66 aplicada. Capacidad `port_scan` disponible como utilidad compilable.
- API/worker: `hackaton-api:scanner-progress-20260913`, imagen `134bb5a8990a0e266f9ab34677965c9cebe48b8017b94a60c6fd490d2c9ee4bd`.
- Web: `hackaton-web:scanner-progress-20260913`, imagen `b3c424d649b0121af2216b19cb6053b4c8943aee7ecdc8ea2755b1634228a2a2`.
- Fuentes: `/home/raul/hackaton-scanner-progress-20260913.VTGoyx` del servidor.
- Respaldo privado previo: `/home/raul/hackaton-prod/deploy/pre-scanner-20260913.5ib4Oe/database.dump` (0600).
- API, worker y web activos, sin reinicios al verificar.
