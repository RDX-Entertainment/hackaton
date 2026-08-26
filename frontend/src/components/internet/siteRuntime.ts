/**
 * El puente entre un sitio y el navegador del juego.
 *
 * Los botones, los campos y los huecos de un sitio son HTML del autor —donde quiera y con su
 * estilo—, pero el marco donde se pinta está aislado: nada de ahí dentro llega solo hasta
 * aquí. Este guion lo pone el juego dentro del marco, y hace tres cosas y solo tres:
 *
 *   1. al pulsar algo con `data-action`, recoge los `data-field` de la página y lo avisa;
 *   2. rellena los `data-bind` con lo que el juego le mande de vuelta;
 *   3. repite la plantilla de un `data-list` una vez por fila.
 *
 * **El HTML del autor no puede traer código**: el servidor rechaza `<script>` y los atributos
 * `on…=` al guardar. El único guion que corre aquí es este, y por eso el marco lleva
 * `allow-scripts` pero nunca `allow-same-origin`: sin lo segundo el documento queda en un
 * origen opaco, sin acceso al DOM de la aplicación, ni a su almacenamiento, ni a sus cookies.
 * Lo único que cruza son los dos mensajes de abajo.
 *
 * Si alguna vez se relajara la regla del servidor y un sitio pudiera traer su propio
 * `<script>`, esto dejaría de ser seguro. La combinación se sostiene entera, no por partes.
 */

export const SITE_MESSAGE = 'hackaton:site-action';
export const SITE_FILL = 'hackaton:site-fill';

/** Lo que el marco manda al pulsar. Se comprueba el emisor antes de hacerle caso. */
export type SiteMessage = {
  source: typeof SITE_MESSAGE;
  kind: string;
  target: string;
  params: Record<string, string>;
  /** Lo tecleado en los `data-field` de la página. Solo lo mandan las webs autorizadas. */
  fields: Record<string, string>;
};

/** Lo que el juego manda de vuelta para que el sitio se pinte. */
export type SiteFill = {
  source: typeof SITE_FILL;
  values?: Record<string, string>;
  lists?: Record<string, Record<string, string>[]>;
  /** Vaciar los campos tecleados, tras entrar o enviar. */
  clear?: boolean;
  /** Si hay sesión abierta: enseña los `data-when="in"` y esconde los `data-when="out"`. */
  session?: boolean;
};

const RUNTIME = `
<script>
(function () {
  var RESERVED = ['action', 'target'];

  function collectFields() {
    var fields = {};
    var nodes = document.querySelectorAll('[data-field]');

    for (var i = 0; i < nodes.length; i++) {
      var node = nodes[i];
      var name = node.dataset.field;
      if (name) { fields[name] = node.value != null ? String(node.value) : ''; }
    }

    return fields;
  }

  document.addEventListener('click', function (event) {
    var node = event.target;

    while (node && node !== document.body) {
      if (node.dataset && node.dataset.action) { break; }
      node = node.parentElement;
    }

    if (!node || !node.dataset || !node.dataset.action) { return; }

    event.preventDefault();

    var params = {};
    Object.keys(node.dataset).forEach(function (name) {
      if (RESERVED.indexOf(name) === -1) { params[name] = node.dataset[name]; }
    });

    parent.postMessage({
      source: '${SITE_MESSAGE}',
      kind: node.dataset.action,
      target: node.dataset.target || '',
      params: params,
      fields: collectFields()
    }, '*');
  });

  // Enter dentro de un campo vale por el primer boton que haya: escribir la cuenta y el PIN
  // y tener que buscar el raton con el dedo seria raro en cualquier web.
  document.addEventListener('keydown', function (event) {
    if (event.key !== 'Enter') { return; }
    if (!event.target || !event.target.dataset || !event.target.dataset.field) { return; }

    var button = document.querySelector('[data-action]');
    if (button) { event.preventDefault(); button.click(); }
  });

  function fillInto(root, values) {
    var nodes = root.querySelectorAll('[data-bind]');

    for (var i = 0; i < nodes.length; i++) {
      var name = nodes[i].dataset.bind;
      if (name && values[name] !== undefined) { nodes[i].textContent = values[name]; }
    }
  }

  window.addEventListener('message', function (event) {
    var data = event.data;
    if (!data || data.source !== '${SITE_FILL}') { return; }

    if (data.values) { fillInto(document, data.values); }

    if (data.lists) {
      Object.keys(data.lists).forEach(function (name) {
        var box = document.querySelector('[data-list="' + name + '"]');
        if (!box) { return; }

        // La primera vez se guarda la plantilla que escribio el autor; despues se repite.
        if (!box.dataset.template) { box.dataset.template = box.innerHTML; }

        var rows = data.lists[name];
        box.innerHTML = '';

        rows.forEach(function (row) {
          var holder = document.createElement('div');
          holder.innerHTML = box.dataset.template;
          fillInto(holder, row);

          while (holder.firstChild) { box.appendChild(holder.firstChild); }
        });
      });
    }

    if (typeof data.session === 'boolean') {
      var marked = document.querySelectorAll('[data-when]');

      for (var w = 0; w < marked.length; w++) {
        var quiere = marked[w].dataset.when === 'in';
        // La regla base oculta la vista privada antes de conocer el estado. Al entrar hay
        // que sobrescribirla en linea: quitar el estilo deja que CSS la siga ocultando.
        marked[w].style.display = quiere === data.session ? 'block' : 'none';
      }
    }

    if (data.clear) {
      var fields = document.querySelectorAll('[data-field]');
      for (var i = 0; i < fields.length; i++) { fields[i].value = ''; }
    }
  });

  var style = document.createElement('style');
  style.textContent = '[data-action]{cursor:pointer}[data-when="in"]{display:none}';
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
