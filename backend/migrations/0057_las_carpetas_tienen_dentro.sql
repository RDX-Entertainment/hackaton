-- Una carpeta que no puede contener nada no es una carpeta.
--
-- `software.is_folder` existe desde la 0001 y en cinco anos no sirvio para nada:
-- marcaba una fila como carpeta y no habia forma de meterle un fichero, porque
-- nunca hubo columna de padre. Aqui se la damos.
--
-- **El sitio de un fichero son dos cosas, no una**: `server_id` dice en que
-- maquina esta y `parent_id` en que carpeta dentro de esa maquina. `NULL` es la
-- raiz del equipo, que es lo que el jugador ve al entrar en el.
--
-- `on delete restrict` a proposito: borrar una carpeta con cosas dentro tendria
-- que decidir si se borra el contenido o se sube un nivel, y esa es una decision
-- del juego, no de la base. La regla la pone el servicio --una carpeta se borra
-- vacia-- y la base la respalda.
--
-- No se comprueba aqui que el padre sea una carpeta, ni que sea del mismo dueno,
-- ni de la misma maquina: eso vive en `software::services`, junto a las otras
-- reglas de sitio (que quepa en el disco, que haya ruta entre los dos equipos).
alter table software
    add column parent_id bigint references software(id) on delete restrict;

create index if not exists software_parent_idx on software(parent_id);

comment on column software.parent_id is
    'La carpeta que lo contiene, dentro de la misma maquina. NULL es la raiz del equipo.';
