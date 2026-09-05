/**
 * El puente entre un sitio y el navegador del juego.
 *
 * Los botones, los campos y los huecos de un sitio son HTML del autor —donde quiera y con su
 * estilo—, y este módulo es quien los reconoce y los obedece. Hace cuatro cosas y solo cuatro:
 *
 *   1. al pulsar algo con `data-action`, recoge los `data-field` de la página y lo avisa;
 *   2. rellena los `data-bind` con lo que el juego le mande de vuelta;
 *   3. repite la plantilla de un `data-list` una vez por fila;
 *   4. enseña u oculta los `data-when` según haya sesión abierta.
 *
 * **Esto corría dentro del marco y ahora corre fuera.** Antes se inyectaba como un `<script>`
 * al final de la página y hablaba con la aplicación por `postMessage`, lo que obligaba a darle
 * al `iframe` el permiso `allow-scripts`. Ahora el marco va con `allow-same-origin` y **sin**
 * `allow-scripts`: el documento sigue siendo suyo —así que su CSS no pisa el del juego ni al
 * revés—, la aplicación alcanza su DOM desde aquí, y **el navegador impide ejecutar cualquier
 * guion ahí dentro**. Si al sanitizador del servidor se le escapara un `<script>`, no correría
 * igualmente: la garantía deja de depender de que el filtro sea perfecto.
 *
 * (El par peligroso es `allow-scripts` **junto a** `allow-same-origin`, porque un guion dentro
 * puede quitarle el `sandbox` a su propio marco. Sin lo primero no hay quien lo intente.)
 */

/** Lo que el marco manda al pulsar. */
export type SiteMessage = {
  kind: string;
  target: string;
  params: Record<string, string>;
  /** Lo tecleado en los `data-field` de la página. Solo lo mandan las webs autorizadas. */
  fields: Record<string, string>;
};

/** Lo que el juego manda de vuelta para que el sitio se pinte. */
export type SiteFill = {
  values?: Record<string, string>;
  lists?: Record<string, Record<string, string>[]>;
  /** Vaciar los campos tecleados, tras entrar o enviar. */
  clear?: boolean;
  /** Si hay sesión abierta: enseña los `data-when="in"` y esconde los `data-when="out"`. */
  session?: boolean;
};

/** Atributos que describen la orden y no son parámetros suyos. */
const RESERVED = ['action', 'target'];

/**
 * Lo poco que el juego añade al documento del sitio: el cursor de los botones y la vista
 * privada escondida hasta saber si hay sesión.
 *
 * Va como hoja propia y no tocando cada elemento porque el CSS del autor tiene que poder
 * ganarle: es su página.
 */
const RUNTIME_CSS = '[data-action]{cursor:pointer}[data-when="in"]{display:none}';

function fieldsOf(root: ParentNode): Record<string, string> {
  const fields: Record<string, string> = {};

  for (const node of root.querySelectorAll<HTMLInputElement>('[data-field]')) {
    const name = node.dataset.field;

    if (name) {
      fields[name] = node.value != null ? String(node.value) : '';
    }
  }

  return fields;
}

/**
 * El elemento con `data-action` que contiene a lo que se ha pulsado, si es que hay alguno.
 *
 * **Nada de `instanceof` aquí.** Este módulo corre en la aplicación y los nodos vienen del
 * documento del marco, que es otro realm: su `HTMLElement` no es el nuestro, así que
 * `nodo instanceof HTMLElement` es falso incluso para un `<button>` de verdad. Se mira lo
 * que el nodo es —`nodeType === 1`, un elemento— y no de qué constructor viene.
 */
const ELEMENT_NODE = 1;

function actionAt(target: EventTarget | null): HTMLElement | null {
  let node = target as HTMLElement | null;

  while (node) {
    if (node.nodeType === ELEMENT_NODE && node.dataset?.action) {
      return node;
    }

    node = node.parentElement;
  }

  return null;
}

function messageFrom(node: HTMLElement, root: ParentNode): SiteMessage {
  const params: Record<string, string> = {};

  for (const [name, value] of Object.entries(node.dataset)) {
    if (!RESERVED.includes(name) && value !== undefined) {
      params[name] = value;
    }
  }

  return {
    kind: node.dataset.action ?? '',
    target: node.dataset.target ?? '',
    params,
    fields: fieldsOf(root),
  };
}

function fillInto(root: ParentNode, values: Record<string, string>) {
  for (const node of root.querySelectorAll<HTMLElement>('[data-bind]')) {
    const name = node.dataset.bind;

    if (name && values[name] !== undefined) {
      node.textContent = values[name];
    }
  }
}

/**
 * Repite la plantilla de una lista una vez por fila.
 *
 * La plantilla se guarda como nodos, clonados del documento, y no como texto: repetir con
 * `innerHTML` volvía a parsear en cada vuelta lo que el autor había escrito, y eso era un
 * segundo camino por el que su HTML se convertía en DOM sin pasar por nadie. Clonar no
 * reinterpreta nada.
 */
const templates = new WeakMap<Element, DocumentFragment>();

function repeat(box: Element, rows: Record<string, string>[]) {
  let template = templates.get(box);

  if (!template) {
    template = box.ownerDocument.createDocumentFragment();

    while (box.firstChild) {
      template.appendChild(box.firstChild);
    }

    templates.set(box, template);
  }

  box.replaceChildren();

  for (const row of rows) {
    const copy = template.cloneNode(true) as DocumentFragment;

    fillInto(copy, row);
    box.appendChild(copy);
  }
}

export function fill(document: Document, data: SiteFill) {
  if (data.values) {
    fillInto(document, data.values);
  }

  if (data.lists) {
    for (const [name, rows] of Object.entries(data.lists)) {
      const box = document.querySelector(`[data-list="${CSS.escape(name)}"]`);

      if (box) {
        repeat(box, rows);
      }
    }
  }

  if (typeof data.session === 'boolean') {
    for (const node of document.querySelectorAll<HTMLElement>('[data-when]')) {
      // La regla base esconde la vista privada antes de conocer el estado. Al entrar hay
      // que sobrescribirla en línea: quitar el estilo deja que el CSS la siga ocultando.
      node.style.display = (node.dataset.when === 'in') === data.session ? 'block' : 'none';
    }
  }

  if (data.clear) {
    for (const node of document.querySelectorAll<HTMLInputElement>('[data-field]')) {
      node.value = '';
    }
  }
}

/**
 * Engancha el juego al documento de un sitio ya pintado.
 *
 * Devuelve la función que lo suelta todo, para cuando el marco cambia de página.
 */
export function attach(
  document: Document,
  onAction: (message: SiteMessage) => void,
): () => void {
  const style = document.createElement('style');
  style.textContent = RUNTIME_CSS;
  document.head.appendChild(style);

  const click = (event: MouseEvent) => {
    const node = actionAt(event.target);

    if (!node) {
      return;
    }

    event.preventDefault();
    onAction(messageFrom(node, document));
  };

  // Enter dentro de un campo vale por el primer botón que haya: escribir la cuenta y el PIN
  // y tener que buscar el ratón con el dedo sería raro en cualquier web.
  const keydown = (event: KeyboardEvent) => {
    if (event.key !== 'Enter') {
      return;
    }

    const field = event.target as HTMLElement | null;

    // Mismo motivo que en `actionAt`: el campo es del realm del marco, no del nuestro.
    if (!field || field.nodeType !== ELEMENT_NODE || !field.dataset?.field) {
      return;
    }

    const button = document.querySelector<HTMLElement>('[data-action]');

    if (button) {
      event.preventDefault();
      button.click();
    }
  };

  document.addEventListener('click', click);
  document.addEventListener('keydown', keydown);

  return () => {
    document.removeEventListener('click', click);
    document.removeEventListener('keydown', keydown);
    style.remove();
  };
}
