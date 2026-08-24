/* Text art for the terminal. Kept apart so the command table stays readable. */

export const HACK_THE_PLANET = String.raw`
  ╦ ╦╔═╗╔═╗╦╔═  ╔╦╗╦ ╦╔═╗  ╔═╗╦  ╔═╗╔╗╔╔═╗╔╦╗
  ╠═╣╠═╣║  ╠╩╗   ║ ╠═╣║╣   ╠═╝║  ╠═╣║║║║╣  ║
  ╩ ╩╩ ╩╚═╝╩ ╩   ╩ ╩ ╩╚═╝  ╩  ╩═╝╩ ╩╝╚╝╚═╝ ╩
`.trim();

export const TEAPOT = String.raw`
       (  )   (   )  )
        ) (   )  (  (
        ( )  (    ) )
        _____________
       <_____________> ___
       |             |/ _ \
       |               | | |
       |               |_| |
    ___|             |\___/
   /    \___________/    \
   \_____________________/

   HTTP 418 · I'm a teapot
`.trimEnd();

/** Wraps text in a speech bubble the way cowsay(1) does. */
export function cowsay(text: string): string {
  const message = text || 'moo';
  const width = message.length;
  const top = ` _${'_'.repeat(width)}_ `;
  const bottom = ` -${'-'.repeat(width)}- `;

  // Plain strings with escaped backslashes rather than String.raw: a raw
  // template cannot end in a backslash, and this cow ends in three.
  return [
    top,
    `< ${message} >`,
    bottom,
    '        \\   ^__^',
    '         \\  (oo)\\_______',
    '            (__)\\       )\\/\\',
    '                ||----w |',
    '                ||     ||',
  ].join('\n');
}

const MATRIX_GLYPHS = 'ｦｱｳｴｵｶｷｹｺｻｼｽｾｿﾀﾂﾃﾅﾆﾇﾈﾊﾋﾎﾏﾐﾑﾒﾓﾔﾕﾗﾘﾜ0123456789';

/**
 * A few rows of falling nonsense. Not an animation — just a splash. Sized to
 * the centred content column, so it never forces a horizontal wrap.
 */
export function matrixRain(rows = 8, columns = 40): string {
  const line = () =>
    Array.from(
      { length: columns },
      () => MATRIX_GLYPHS[Math.floor(Math.random() * MATRIX_GLYPHS.length)],
    ).join('');

  return Array.from({ length: rows }, line).join('\n');
}

const FORTUNES = [
  'Un programa nunca está terminado, solo abandonado.',
  'El mejor código es el que no hay que escribir.',
  'Hay dos problemas difíciles: invalidar cachés, nombrar cosas y los off-by-one.',
  'Si funciona a la primera, desconfía.',
  'La prisa de hoy es la migración de la semana que viene.',
  'Todo sistema acaba teniendo un cron que nadie se atreve a tocar.',
  'El log que no lees es el que tiene la respuesta.',
  'Documentar es escribirle una carta a quien serás dentro de seis meses.',
];

export function fortune(): string {
  return FORTUNES[Math.floor(Math.random() * FORTUNES.length)];
}
