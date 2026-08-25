/**
 * El puente entre un sitio y el navegador del juego.
 *
 * Los botones de un sitio son HTML del autor —donde quiera y con su estilo—, pero el marco
 * donde se pinta está aislado: un clic ahí dentro no llega solo hasta aquí. Este guion lo
 * pone el juego dentro del marco, junto a la página, y lo único que hace es avisar de los
 * clics en los elementos marcados con `data-action`.
 *
 * **El HTML del autor no puede traer código**: el servidor rechaza `<script>` y los
 * atributos `on…=` al guardar. El único guion que corre aquí es este, y por eso el marco
 * lleva `allow-scripts` pero nunca `allow-same-origin`: sin lo segundo el documento queda en
 * un origen opaco, sin acceso al DOM de la aplicación, ni a su almacenamiento, ni a sus
 * cookies. Lo único que puede cruzar es el mensaje de abajo.
 *
 * Si alguna vez se relajara la regla del servidor y un sitio pudiera traer su propio
 * `<script>`, esto dejaría de ser seguro. La combinación se sostiene entera, no por partes.
 */

/** Lo que el marco manda al hacer clic. Se comprueba el emisor antes de hacerle caso. */
export const SITE_MESSAGE = 'hackaton:site-action';

export type SiteMessage = {
  source: typeof SITE_MESSAGE;
  kind: string;
  target: string;
  params: Record<string, string>;
};

/** Lee de un elemento los `data-*` que no son la propia orden ni el destino. */
const RUNTIME = `
<script>
(function () {
  var RESERVED = ['action', 'target'];

  document.addEventListener('click', function (event) {
    var node = event.target;

    while (node && node !== document.body) {
      if (node.dataset && node.dataset.action) {
        break;
      }
      node = node.parentElement;
    }

    if (!node || !node.dataset || !node.dataset.action) {
      return;
    }

    event.preventDefault();

    var params = {};
    Object.keys(node.dataset).forEach(function (name) {
      if (RESERVED.indexOf(name) === -1) {
        params[name] = node.dataset[name];
      }
    });

    parent.postMessage({
      source: '${SITE_MESSAGE}',
      kind: node.dataset.action,
      target: node.dataset.target || '',
      params: params
    }, '*');
  });

  // Un elemento accionable se comporta como tal aunque el autor lo pinte de otra cosa.
  var style = document.createElement('style');
  style.textContent = '[data-action]{cursor:pointer}';
  document.head.appendChild(style);
})();
</script>
`;

/** Pega el puente al final de la página que se va a pintar. */
export function withRuntime(html: string): string {
  return html + RUNTIME;
}

/** Si un mensaje que llega de un marco es uno de los nuestros. */
export function isSiteMessage(data: unknown): data is SiteMessage {
  return (
    typeof data === 'object' &&
    data !== null &&
    (data as SiteMessage).source === SITE_MESSAGE &&
    typeof (data as SiteMessage).kind === 'string'
  );
}
