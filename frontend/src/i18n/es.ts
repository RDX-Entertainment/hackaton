/**
 * Castellano, y el respaldo de los otros dos: si a `pt` o a `en` les falta una clave, sale
 * la de aqui antes que el codigo pelado.
 *
 * Las frases dicen que ha pasado **y por donde se sale**. Un error que solo dice que no es
 * la mitad de un error.
 */
export const es: Record<string, string> = {
  // Los cuatro que trae cualquier respuesta fallida.
  'error.UNAUTHORIZED': 'Tu sesión ha caducado. Vuelve a entrar.',
  'error.NOT_FOUND': 'Ahí no hay nada.',
  'error.INTERNAL': 'Algo se ha roto por dentro. No es culpa tuya.',
  'error.BAD_REQUEST': 'Esa petición no se puede atender.',

  // El primer bucle: fabricar, salir y entrar.
  'error.NO_CERTIFICATION': 'Te falta la certificación. La sacas gratis en Academia y tardas un minuto.',
  'error.NO_SOFT': 'No tienes ningún programa capaz de hacer esto. Diseña uno en Arsenal y compílalo.',
  'error.BAD_CRACKER': 'Tu programa es más flojo que la defensa que tiene delante. Súbele la versión o busca un objetivo más blando.',
  'error.SERVER_DOWN': 'Esa máquina está caída. Prueba dentro de un rato.',
  'error.NOT_LOGGED_IN': 'Todavía no has entrado en esa máquina. En «force» están las formas de conseguirlo.',
  'error.NO_ACCESS': 'No tienes acceso a esa máquina.',
  'error.TARGET_GONE': 'Esa máquina ya no existe.',
  'error.PAYLOAD_GONE': 'El programa que llevaba la operación ha desaparecido de tu disco.',
  'error.EXPLOIT_BLOCKED': 'El firewall rechazó tus exploits.',

  // Disco, memoria y compilación.
  'error.DISK_FULL': 'No cabe en ningún disco tuyo. Libera espacio o compra otro.',
  'error.NO_DISK_SPACE': 'No hay sitio en ese disco.',
  'error.COMPILATION_ACTIVE': 'Ya tienes una compilación en marcha. Termina esa antes de empezar otra.',
  'error.CANNOT_TRANSFER': 'Ese fichero no se puede mover.',
  'error.SOFTWARE_RUNNING': 'Está cargado en memoria. Descárgalo antes.',
  'error.REQUIRED_SOFTWARE_NOT_RUNNING': 'Necesitas ese programa cargado en memoria, no solo guardado en disco.',
  'error.INSUFFICIENT_RESOURCES': 'A tu equipo no le dan los recursos para esto.',
  'error.SKILL_LOCKED': 'Tu programa no sabe hacer eso todavía. Necesita más de un atributo concreto.',
  'error.INSUFFICIENT_PERMISSION': 'El acceso que tienes no llega para eso.',
  'error.SAME_MACHINE': 'Origen y destino son la misma máquina.',
  'error.NO_MACHINE': 'Ese jugador no tiene ninguna máquina donde poner esto.',

  // Dinero.
  'error.INSUFFICIENT_FUNDS': 'No tienes saldo suficiente.',
  'error.BAD_PASSWORD': 'Esa contraseña no es.',
  'error.PIN_ALREADY_SET': 'Esa cuenta ya tiene PIN.',

  // Equipos y piezas.
  'error.TOO_MANY_MACHINES': 'No te quedan plazas. Compra una antes de montar otro equipo.',
  'error.LAST_MACHINE': 'Es tu última máquina. Sin ninguna no podrías jugar.',
  'error.MACHINE_NOT_EMPTY': 'Vacía la máquina antes de darla de baja.',
  'error.NO_FREE_SLOT': 'No queda ranura libre de ese tipo.',
  'error.NO_SLOTS': 'La placa no admite piezas de ese tipo.',
  'error.CHIPSET_MISMATCH': 'Esa pieza no encaja en el chipset de la placa.',
  'error.DDR_MISMATCH': 'Esa memoria no es del tipo que admite la placa.',
  'error.OVER_CAPACITY': 'La placa no llega a tanto.',
  'error.PART_IN_USE': 'Esa pieza está instalada. Quítala antes.',
  'error.RAM_IN_USE': 'Hay programas cargados que necesitan esa memoria.',
  'error.DISK_IN_USE': 'Hay ficheros guardados en ese disco.',
  'error.NOT_CABLED': 'Esas dos máquinas no están conectadas.',
  'error.NO_ROUTE': 'No hay camino hasta ahí.',

  // Y el que sale cuando un proceso revienta por dentro.
  'error.WORKER_ERROR': 'El proceso no se pudo terminar.',
  'error.UNKNOWN_LANGUAGE': 'Ese idioma no está entre los que habla el juego.',
};
