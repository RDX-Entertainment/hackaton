# Módulos de capacidades

Lab → Mejorar contiene el inventario, las ranuras y la fusión. Los módulos se instalan después de compilar. Los perfiles y diseños anteriores conservan sus mejoras integradas; no se vuelven a sumar ni se convierten en copias físicas. La edición libre de perfiles queda reservada a administradores.

- Nivel del programa = versión, aumentada con Lab → Investigar.
- Ranuras desbloqueadas en niveles 1, 3, 6, 10, 15 y 21; seis como máximo.
- Estándar / Optimizado / Experimental / Clasificado / Zero-day: multiplicadores 1 / 1,35 / 1,8 / 2,4 / 3,2. Redondeo al entero más cercano; también se escalan los costes.
- Fusionar consume 3 copias libres del mismo catálogo y grado; entrega 1 del grado siguiente. No admite reversión ni fusión de Zero-day.
- Cada Encargo completado entrega una copia aleatoria recabable. Los niveles 1, 2 y 3
  entregan respectivamente grados Estándar, Optimizado y Experimental.
- Instalar consume una copia; retirar o borrar el archivo devuelve la copia. Los módulos instalados no están disponibles para fusión.
- Para cambiar módulos el programa debe estar fuera de memoria y sin procesos pendientes. Se valida el espacio de su disco. La RAM y el tamaño del archivo reflejan su configuración efectiva.

El Sensor heurístico concede `antivirus_scan`; el Motor de purga concede `antivirus_clean`. Son funciones independientes: detectar no concede limpieza. Un sensor Zero-day aporta 96 de detección, permitiendo superar el antivirus base de 50. El resultado del combate sigue siendo probabilístico y se compara con el sigilo del virus.

Admin → Software → Módulos permite crear/editar nombre, deltas, funciones concedidas, capacidades base compatibles, nivel mínimo, posiciones admitidas, máximo de copias e incompatibilidades. Destinos vacíos significa cualquier capacidad. Las incompatibilidades se comprueban en ambas direcciones. Desactivar impide nuevas instalaciones y reparto, sin destruir las existentes. El código no se puede renombrar.

Cada instalación conserva una instantánea del catálogo: las ediciones administrativas afectan futuras instalaciones, no las que ya están equipadas. Los procesos conservan sus estadísticas y tiradas al encolarse. Las operaciones y defensas consultan el perfil efectivo (base + módulos); las capacidades concedidas participan en la selección de herramientas. Los módulos físicos nunca se clonan con archivos infectados.

API autenticada:

- `GET /api/software-system/workshop`: catálogo, existencias por grado, módulos instalados, niveles y tiempo de espera.
- El botín entra al inventario al completar el objetivo de un Encargo.
- `POST /api/software-system/workshop/equip`: `{software_id, slot, module_id, tier}`; posiciones 1–6, grados 0–4.
- `POST /api/software-system/workshop/unequip`: `{software_id, slot}`.
- `POST /api/software-system/workshop/fuse`: `{module_id, tier}`.
- `GET /api/admin/software-modules` y `POST` a la misma ruta: catálogo y alta/edición, protegido por AdminOnly; `id` opcional identifica una edición.

Validación local aislada: `HACKATON_TEST_BASE=http://127.0.0.1:18091 python3 scripts/test-module-workshop.py`. Nunca ejecuta contra producción.
