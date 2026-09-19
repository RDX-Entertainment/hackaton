/**
 * Castellano, y el respaldo de los otros dos: si a `pt` o a `en` les falta una clave, sale
 * la de aqui antes que el codigo pelado.
 *
 * Las frases dicen que ha pasado **y por donde se sale**. Un error que solo dice que no es
 * la mitad de un error.
 */
export const es: Record<string, string> = {
  'language.label': 'Idioma',
  'shell.funds': 'fondos',
  'shell.running': 'en marcha',
  'shell.mail': 'correo',
  'shell.admin': 'panel',
  'shell.logout': 'salir',
  'shell.showIp': 'Mostrar tu dirección',
  'shell.hideIp': 'Ocultar tu dirección',

  'nav.destinations': 'Destinos del juego',
  'nav.screens': 'Pantallas',
  'nav.destination.equipo': 'Equipo',
  'nav.destination.red': 'Red',
  'nav.destination.trabajo': 'Trabajo',
  'nav.destination.gente': 'Gente',
  'nav.destination.cuenta': 'Cuenta',
  'nav.screen.hardware': 'Piezas',
  'nav.screen.software': 'Discos',
  'nav.screen.lab': 'Lab',
  'nav.screen.processes': 'Tareas',
  'nav.screen.internet': 'Internet',
  'nav.screen.accesses': 'Accesos',
  'nav.screen.missions': 'Encargos',
  'nav.screen.university': 'Academia',
  'nav.screen.finances': 'Banco',
  'nav.screen.bitcoin': 'Bitcoin',
  'nav.screen.mail': 'Correo',
  'nav.screen.clan': 'Clan',
  'nav.screen.forum': 'Foro',
  'nav.screen.wiki': 'Wiki',
  'nav.screen.ranking': 'Ranking',
  'nav.screen.profile': 'Perfil',
  'nav.screen.settings': 'Ajustes',
  'nav.screen.log': 'Registro',
  'nav.screen.list': 'Listas',
  'nav.screen.stats': 'Números',

  'admin.sections': 'Secciones del panel',
  'admin.back': 'volver al juego',
  'admin.catalog': 'Catálogo',
  'admin.kits': 'Kits',
  'admin.organizaciones': 'Organizaciones',
  'admin.sitios': 'Sitios',
  'admin.acciones': 'Acciones',
  'admin.usuarios': 'Usuarios',
  'admin.software': 'Software',
  'admin.tutorial': 'Tutorial',

  // Los cuatro que trae cualquier respuesta fallida.
  'error.UNAUTHORIZED': 'Tu sesión ha caducado. Vuelve a entrar.',
  'error.NOT_FOUND': 'Ahí no hay nada.',
  'error.INTERNAL': 'Algo se ha roto por dentro. No es culpa tuya.',
  'error.BAD_REQUEST': 'Esa petición no se puede atender.',

  // El primer bucle: fabricar, salir y entrar.
  'error.NO_CERTIFICATION': 'Te falta la certificación. La sacas gratis en Academia y tardas un minuto.',
  'error.NO_SOFT': 'No tienes ningún programa capaz de hacer esto. Diseña uno en Lab y compílalo.',
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

  // El circuito: los cinco pasos que llevan de una cuenta recien hecha a estar dentro de una
  // maquina ajena. La clave `.step` es el rotulo corto del riel; `.title` y `.hint` son lo
  // que la portada dice cuando ese es el paso que toca.
  'onboarding.modules.step': 'Investigar',
  'onboarding.modules.title': 'Consigue módulos con los que construir',
  'onboarding.modules.hint': 'Un programa es una capacidad más los módulos que le pongas. Sin módulos solo salen programas pelados.',
  'onboarding.modules.action': 'Investigar en Lab',

  'onboarding.blueprint.step': 'Diseñar',
  'onboarding.blueprint.title': 'Diseña tu primer programa',
  'onboarding.blueprint.hint': 'Elige qué quiere hacer —entrar en una máquina, dejar algo dentro— y móntale hasta tres módulos. Verás lo que va a salir antes de gastar un segundo en compilarlo.',
  'onboarding.blueprint.action': 'Diseñar en Lab',

  'onboarding.compile.step': 'Compilar',
  'onboarding.compile.title': 'Compila el diseño',
  'onboarding.compile.hint': 'Compilar tarda, y hasta que no termina no hay fichero. Solo puedes tener una compilación en marcha.',
  'onboarding.compile.action': 'Compilar en Lab',
  'onboarding.compile.waiting': 'Compilando. Cuando termine, el fichero aparece en el disco de la máquina donde quepa.',

  'onboarding.install.step': 'Cargar',
  'onboarding.install.title': 'Cárgalo en memoria',
  'onboarding.install.hint': 'Lo que está en disco no dispara. Un programa solo funciona en la máquina donde está cargado, y ocupa su RAM mientras lo esté.',
  'onboarding.install.action': 'Cargar desde Discos',

  'onboarding.intrusion.step': 'Entrar',
  'onboarding.intrusion.title': 'Entra en tu primera máquina',
  'onboarding.intrusion.hint': 'Ya tienes con qué. Estas dos direcciones son tuyas y están para esto: nadie más las ve, y no se enfadan.',
  'onboarding.intrusion.action': 'Abrir Internet',

  'onboarding.done.title': 'El circuito está cerrado',
  'onboarding.done.hint': 'Diseñas, compilas, cargas y entras. A partir de aquí el juego es tuyo: misiones, bancos y lo que se te ocurra.',

  'onboarding.practice.title': 'Tu campo de tiro',
  'onboarding.practice.hint': 'Dos máquinas de práctica que solo ves tú.',
  'onboarding.rail.title': 'El circuito de un programa',

  //
  // El tutorial: lo que dice cuando algo esta cerrado, y por que.
  //
  'error.TUTORIAL_LOCKED':
    'Ahora no toca eso. Termina el paso del tutorial que tienes en marcha y se abre solo.',
  'error.CERTIFICATION_BY_DOING':
    'Esta certificación no se estudia: se aprueba jugando. Haz lo que pide el tutorial y la tienes.',
  'tutorial.locked':
    'Cerrado hasta que termines: {paso}',
  'tutorial.go':
    'Ir al paso',
  'tutorial.earned':
    'Se aprueba haciendo',
};
