/**
 * A qué pantalla lleva cada paso.
 *
 * Vive aquí y no en el servidor a propósito: las rutas son cosa del cliente, y el día que
 * una pantalla cambie de sitio no hay que desplegar la API para arreglarlo.
 */
export const RUTA_DEL_PASO: Record<string, string> = {
  modules: '/research',
  blueprint: '/arsenal',
  compile: '/arsenal',
  install: '/software',
  intrusion: '/internet',
};
