-- El tutorial deja de estar escrito en el codigo y pasa a ser un dato que se edita.
--
-- A3 dejo el circuito en una constante de Rust: cinco pasos, en orden, deducidos del mundo.
-- Eso resolvio lo dificil --el paso no se guarda, se calcula-- pero dejo lo demas donde no
-- sirve: para cambiar que ensena el tutorial hay que tocar Rust, recompilar y desplegar.
--
-- Aqui se separan dos cosas que no son la misma:
--
--   * **El progreso se sigue deduciendo del mundo.** No hay, y no va a haber, una columna
--     `tutorial_step`. Lo que cierra un paso es un hecho --hay un diseno, hay un fichero, hay
--     algo en memoria-- y el hecho se consulta, no se marca.
--   * **La definicion es un dato.** Que pasos hay, en que orden, que se puede tocar mientras
--     dura cada uno y donde mira el jugador: eso se escribe en el panel.
--
-- ## Por que el hecho es un vocabulario cerrado
--
-- `fact` no es una consulta que escriba el administrador: es una de las preguntas que el
-- servidor sabe hacerle al mundo. Dejar escribir SQL desde el panel seria dejar que un
-- tutorial mal escrito reviente la portada de todo el mundo, y ademas mentiria: la pantalla
-- de A3 responde con seis `EXISTS` en una sola consulta, y esa consulta la escribe el
-- servidor. El vocabulario crece anadiendo un `EXISTS` y un valor aqui, que es una linea en
-- cada sitio.
--
-- ## `NULL` no es lo mismo que la lista vacia
--
-- En `views` y `actions`, **`NULL` significa «todas»** y una lista significa «solo estas».
-- La lista vacia, entonces, es «ninguna», que es un tutorial que no deja jugar y tiene que
-- poder escribirse igual: si el administrador quiere una pantalla sin escapatoria, es su
-- decision y la base no es quien para discutirla. Lo que no puede pasar es que «no he tocado
-- esto» y «no dejo nada» se escriban igual.
--
-- ## El bloqueo vive en el servidor
--
-- Estas listas las lee la API antes de dejar hacer nada --`TUTORIAL_LOCKED`, al lado del
-- `NO_CERTIFICATION` que ya existia-- y ademas viajan al cliente para que la interfaz lo
-- ensene apagado y con su razon. Las dos cosas: el cliente que solo apaga botones es un
-- adorno que se salta con `curl`, y el servidor que solo veta deja una pantalla que miente.

create table tutorial_steps (
    id           bigserial   primary key,

    -- El orden del circuito. Se mira en orden y se para en el primero sin hacer.
    position     int         not null,

    -- El codigo del paso. Es lo que viaja al cliente y lo que este traduce: el servidor no
    -- sabe en que lengua lee quien pregunta.
    code         varchar(32) not null unique,

    -- Que hecho del mundo lo cierra.
    fact         varchar(32) not null check (fact in (
                     'modules', 'blueprints', 'programs', 'running', 'intrusions'
                 )),

    -- Las pantallas que se pueden abrir mientras este paso es el actual. NULL = todas.
    views        text[],

    -- Lo que se puede hacer mientras este paso es el actual. NULL = todo.
    actions      text[],

    -- Donde mira el jugador: la pantalla y el ancla dentro de ella. El ancla es un sitio que
    -- el cliente marca --`data-foco`--, no un selector CSS: un selector se rompe en silencio
    -- al primer cambio de maquetacion y nadie se entera hasta que alguien juega.
    focus_view   varchar(64),
    focus_anchor varchar(64),

    -- La frase, cuando el cliente no conoce el codigo. Los cinco pasos sembrados no la
    -- necesitan --su texto esta en los catalogos de idioma-- pero un paso escrito hoy en el
    -- panel no esta en ningun catalogo, y quedarse mudo seria peor que hablar un solo idioma.
    hint         text,

    active       boolean     not null default true
);

--
-- Unica y **aplazada al final de la transaccion**.
--
-- Meter un paso en medio es apartar a los de detras --`position = position + 1`-- y esa
-- cuenta pasa por estados donde dos filas comparten numero un instante. Con una unica normal
-- Postgres comprueba fila a fila y revienta ahi; aplazada, comprueba al confirmar, que es
-- cuando el orden vuelve a tener sentido.
--
alter table tutorial_steps
    add constraint tutorial_steps_position_unica unique (position)
    deferrable initially deferred;

comment on table tutorial_steps is
    'La definicion del tutorial. El progreso no se guarda en ninguna parte: se deduce del mundo con el hecho de cada paso.';
comment on column tutorial_steps.views is
    'Pantallas abiertas mientras dura el paso. NULL es todas; la lista vacia es ninguna.';
comment on column tutorial_steps.actions is
    'Acciones permitidas mientras dura el paso. NULL es todas; la lista vacia es ninguna.';

--
-- El interruptor. Una fila, y la clave lo garantiza.
--
-- Sirve para lo que sirve un interruptor: apagar el tutorial entero sin borrar lo escrito
-- cuando algo sale mal en produccion un domingo. Sin el, la unica forma de soltar a un
-- jugador atascado seria vaciar las listas de los cinco pasos a mano.
--
create table tutorial_config (
    id       boolean primary key default true check (id),
    enforced boolean not null default true
);

insert into tutorial_config (id, enforced) values (true, true);

comment on table tutorial_config is
    'Una sola fila. `enforced` en falso deja el tutorial como guia: sigue diciendo el paso, deja de bloquear.';

--
-- El circuito de A3, tal cual, y con la correa que decidio Raul: se ensena bloqueando lo
-- demas y resaltando el boton, hasta lo basico, y ahi se suelta.
--
-- Tres cosas del reparto que no son obvias:
--
--   * **Tareas sale en casi todos los pasos.** Compilar y cargar en memoria no terminan al
--     pulsar: encolan trabajo. Un tutorial que manda compilar y esconde la pantalla donde se
--     ve la compilacion ensena a pulsar un boton y a esperar a ciegas.
--   * **Ajustes y Perfil no se secuestran nunca.** Ahi se cambia el idioma, y encerrar a
--     alguien en una lengua que no lee mientras le explicas el juego es la unica forma
--     segura de que no vuelva.
--   * **El ultimo paso deja Discos abierto** aunque lo que toca sea entrar en una maquina:
--     el programa que se lanza hay que verlo.
--
insert into tutorial_steps (position, code, fact, views, actions, focus_view, focus_anchor) values
    (1, 'modules',   'modules',
        array['lab', 'settings', 'profile'],
        array['lab.investigar'],
        'lab', 'lab.investigar'),

    (2, 'blueprint', 'blueprints',
        array['lab', 'settings', 'profile'],
        array['lab.investigar', 'lab.disenar'],
        'lab', 'lab.disenar'),

    (3, 'compile',   'programs',
        array['lab', 'processes', 'settings', 'profile'],
        array['lab.disenar', 'lab.compilar'],
        'lab', 'lab.compilar'),

    (4, 'install',   'running',
        array['software', 'processes', 'lab', 'settings', 'profile'],
        array['discos.cargar', 'discos.descargar'],
        'software', 'discos.cargar'),

    (5, 'intrusion', 'intrusions',
        array['internet', 'software', 'processes', 'accesses', 'settings', 'profile'],
        array['red.entrar', 'red.hackear', 'red.analizar', 'discos.cargar', 'discos.descargar'],
        'internet', 'red.entrar');
