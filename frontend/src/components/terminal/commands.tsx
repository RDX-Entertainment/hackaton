import { tr } from '../../i18n/text';
import { formatIp } from '../../utils/ip';
import { SystemFetch } from '../SystemFetch';
import { cowsay, fortune, HACK_THE_PLANET, matrixRain, TEAPOT } from './asciiArt';
import type { Command, CommandContext } from './types';

/*
 * The command table. Everything the terminal can do lives here, including the
 * jokes — Raul asked for them to be reachable from `help`, so they are a group
 * rather than a set of hidden strings.
 */

const FAKE_FILES: Record<string, string> = {
  'README.md': [
    '# Hacker Experience',
    '',
    'Un juego de hacking ficticio. Nada de lo que hay aquí toca una máquina real:',
    'todo vive en una base de datos PostgreSQL.',
    '',
    'Escribe `useradd` para crear una cuenta, o `login` si ya tienes una.',
  ].join('\n'),
  '.bashrc': [
    '# ~/.bashrc',
    'export PS1="\\u@\\h:\\w\\$ "',
    'alias ll="ls -la"',
    'alias please="sudo"',
    '',
    '# no borres esta línea',
    'echo "the quieter you become, the more you are able to hear"',
  ].join('\n'),
  'todo.txt': [
    '[x] portar el bucle jugable',
    '[x] repartir el backend en módulos',
    '[x] ordenar el frontend',
    '[ ] guerra de clanes',
    '[ ] storyline',
    '[ ] que alguien juegue de una vez',
  ].join('\n'),
  get '.secret'() { return tr("No hay nada aquí. Bueno, sí: prueba `fortune`."); },
};

function helpCommand(context: CommandContext, registry: Command[]): void {
  const groups: Command['group'][] = ['sesión', 'sistema', 'extras'];

  context.print(tr("Comandos disponibles. Escribe el nombre y pulsa Enter."), 'hint');

  for (const group of groups) {
    const inGroup = registry.filter((command) => command.group === group);
    if (!inGroup.length) {
      continue;
    }

    const lines = inGroup.map((command) => {
      const usage = command.usage ?? command.name;
      return `  ${usage.padEnd(16)} ${command.summary}`;
    });

    context.print(`\n${tr(group).toUpperCase()}\n${lines.join('\n')}`);
  }

  context.print(tr("↑ y ↓ recorren el historial · Ctrl+L limpia la pantalla"), 'hint');
}

const sessionCommands: Command[] = [
  {
    name: 'login',
    group: 'sesión',
    get summary() { return tr("Entrar con una cuenta existente."); },
    usage: 'login',
    run: (context) => {
      context.ask(
        [
          { key: 'identifier', label: 'email' },
          { key: 'password', label: 'password', mask: true },
        ],
        async (answers) => {
          await context.session.login(answers.identifier, answers.password);
        },
      );
    },
  },
  {
    name: 'useradd',
    group: 'sesión',
    aliases: ['register', 'adduser'],
    get summary() { return tr("Crear una cuenta nueva."); },
    usage: 'useradd',
    run: (context) => {
      context.ask(
        [
          { key: 'username', label: 'username' },
          { key: 'email', label: 'email' },
          { key: 'password', label: 'password', mask: true },
        ],
        async (answers) => {
          await context.session.register(answers.username, answers.email, answers.password);
        },
      );
    },
  },
  {
    name: 'systemctl',
    group: 'sesión',
    usage: 'systemctl start UI',
    get summary() { return tr("Arrancar la interfaz gráfica."); },
    run: (context) => {
      const [verb, unit] = context.args;

      // Sólo hay una unidad que arrancar, pero se comprueba igual: `systemctl`
      // a secas no debería sacarte del shell por accidente.
      if (verb?.toLowerCase() !== 'start' || unit?.toLowerCase() !== 'ui') {
        context.print(tr("Uso: systemctl start UI"), 'error');
        return;
      }

      context.session.startUi();
    },
  },
  {
    name: 'start-ui',
    group: 'sesión',
    aliases: ['startui'],
    get summary() { return tr("Atajo de `systemctl start UI`."); },
    run: (context) => {
      context.session.startUi();
    },
  },
  {
    name: 'whoami',
    group: 'sesión',
    get summary() { return tr("Quién eres ahora mismo."); },
    run: (context) => {
      const me = context.session.identity();

      if (!me) {
        context.print(tr("guest · sin autenticar. Prueba `login` o `useradd`."));
        return;
      }

      context.print(`${me.login} · ip ${formatIp(me.ip)}`);
    },
  },
];

const systemCommands: Command[] = [
  {
    name: 'neofetch',
    group: 'sistema',
    get summary() { return tr("Datos de tu equipo, del navegador."); },
    run: (context) => {
      context.printNode(<SystemFetch />);
    },
  },
  {
    name: 'clear',
    group: 'sistema',
    aliases: ['cls'],
    get summary() { return tr("Limpiar la pantalla."); },
    run: (context) => {
      context.clear();
    },
  },
  {
    name: 'ls',
    group: 'sistema',
    get summary() { return tr("Listar el directorio."); },
    run: (context) => {
      const names = Object.keys(FAKE_FILES);
      context.print(names.join('   '));
    },
  },
  {
    name: 'cat',
    group: 'sistema',
    get summary() { return tr("Leer un fichero."); },
    get usage() { return tr("cat <fichero>"); },
    run: (context) => {
      const name = context.args[0];

      if (!name) {
        context.print(tr("cat: falta el nombre del fichero"), 'error');
        return;
      }

      const contents = FAKE_FILES[name]?.split('\n').map((line) => tr(line)).join('\n');
      if (contents === undefined) {
        context.print(tr("cat: {0}: No existe el fichero", name), 'error');
        return;
      }

      context.print(contents);
    },
  },
  {
    name: 'echo',
    group: 'sistema',
    get summary() { return tr("Repetir lo que escribas."); },
    get usage() { return tr("echo <texto>"); },
    run: (context) => {
      context.print(context.rest);
    },
  },
  {
    name: 'date',
    group: 'sistema',
    get summary() { return tr("Fecha y hora locales."); },
    run: (context) => {
      context.print(new Date().toString());
    },
  },
  {
    name: 'uname',
    group: 'sistema',
    get summary() { return tr("Identificar el sistema."); },
    run: (context) => {
      context.print(tr("HackerOS 0.1.0 hackaton x86_64 GNU/Ficción"));
    },
  },
  {
    name: 'history',
    group: 'sistema',
    get summary() { return tr("Comandos que has escrito."); },
    run: (context) => {
      if (!context.history.length) {
        context.print(tr("Todavía nada."), 'hint');
        return;
      }

      const lines = context.history.map(
        (entry, index) => `  ${String(index + 1).padStart(3)}  ${entry}`,
      );
      context.print(lines.join('\n'));
    },
  },
];

const easterEggs: Command[] = [
  {
    name: 'sudo',
    group: 'extras',
    get summary() { return tr("Pedir permisos que no tienes."); },
    get usage() { return tr("sudo <lo que sea>"); },
    run: (context) => {
      context.print('[sudo] password for guest: ');
      context.print(
        'guest is not in the sudoers file. This incident will be reported.',
        'error',
      );
    },
  },
  {
    name: 'hack',
    group: 'extras',
    get summary() { return tr("Hackear el planeta."); },
    run: (context) => {
      context.print(HACK_THE_PLANET, 'success');
      context.print('Mess with the best, die like the rest.', 'hint');
    },
  },
  {
    name: 'matrix',
    group: 'extras',
    get summary() { return tr("Ver la lluvia."); },
    run: (context) => {
      context.print(matrixRain(), 'success');
      context.print('Wake up...', 'hint');
    },
  },
  {
    name: 'cowsay',
    group: 'extras',
    get summary() { return tr("Que lo diga una vaca."); },
    get usage() { return tr("cowsay <texto>"); },
    run: (context) => {
      context.print(cowsay(context.rest));
    },
  },
  {
    name: 'fortune',
    group: 'extras',
    get summary() { return tr("Una verdad incómoda al azar."); },
    run: (context) => {
      context.print(fortune(), 'hint');
    },
  },
  {
    name: 'coffee',
    group: 'extras',
    aliases: ['brew'],
    get summary() { return tr("Pedir un café."); },
    run: (context) => {
      context.print(TEAPOT);
    },
  },
  {
    name: 'rm',
    group: 'extras',
    get summary() { return tr("Borrar cosas. Con cuidado."); },
    usage: 'rm -rf /',
    run: (context) => {
      const flags = context.rest.trim();

      if (flags.includes('-rf') && flags.includes('/')) {
        context.print(tr("rm: se niega a borrar «/»"), 'error');
        context.print(tr("Buen intento. Aquí ya se borró todo una vez."), 'hint');
        return;
      }

      context.print(tr("rm: usa `rm -rf /` si de verdad tienes prisa"), 'hint');
    },
  },
  {
    name: 'exit',
    group: 'extras',
    aliases: ['logout', 'quit'],
    get summary() { return tr("Salir. En teoría."); },
    run: (context) => {
      context.print(tr("No hay salida. Solo `login`."), 'hint');
    },
  },
  {
    name: '42',
    group: 'extras',
    get summary() { return tr("La respuesta."); },
    run: (context) => {
      context.print(tr("42. La pregunta sigue siendo el problema."), 'hint');
    },
  },
];

/** Everything, with `help` appended last so it can describe the rest. */
export const COMMANDS: Command[] = (() => {
  const base = [...sessionCommands, ...systemCommands, ...easterEggs];

  const help: Command = {
    name: 'help',
    group: 'sistema',
    aliases: ['?', 'man'],
    get summary() { return tr("Esta lista."); },
    run: (context) => helpCommand(context, [...base, help]),
  };

  // help sits with the rest of the system commands rather than at the end.
  return [...sessionCommands, help, ...systemCommands, ...easterEggs];
})();

export function findCommand(name: string): Command | undefined {
  const wanted = name.toLowerCase();
  return COMMANDS.find(
    (command) => command.name === wanted || command.aliases?.includes(wanted),
  );
}
