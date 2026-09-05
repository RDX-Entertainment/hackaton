-- Fase 3 de «cada máquina es un ordenador»: el proceso nace en un equipo.
--
-- La fase 1 bajó el software a un disco concreto y la 2 dio forma de moverlo. La memoria
-- siguió detrás el mismo día. Faltaba lo que queda aquí: que un proceso —un ataque, una
-- descarga, una investigación— **salga de una máquina** y no de una cuenta.
--
-- Hasta ahora `processes` guardaba `creator_id` y nada más: quién lo lanzó, no desde dónde.
-- Con una sola caja por jugador daba igual. Con varias no: dos equipos del mismo dueño tienen
-- procesador, memoria y salida distintos, y desde el 2026-08-31 tienen hasta dirección propia.
-- Un ataque que sale de la máquina buena y otro que sale de la vieja no son el mismo ataque.
--
-- ## Por qué se guarda y no se deduce
--
-- Se podría sacar de `soft_id` mirando dónde vive el programa hoy. **Y estaría mal**, porque la
-- fase 2 permite mover software entre equipos: deducirlo reescribiría el pasado cada vez que
-- alguien mueve un programa, y un proceso lanzado ayer desde el servidor pasaría a decir que
-- salió del portátil. El origen es un hecho del momento en que arrancó, así que se escribe
-- entonces y ya no cambia.
--
-- ## Nullable, y `ON DELETE SET NULL`
--
-- Hay procesos sin programa —el escaneo de una máquina, un intento de acceso—, y esos no tienen
-- de dónde salir mientras no exista el gateway de la fase 3 de direcciones. Se quedan en `NULL`,
-- que aquí significa «no se sabe», no «ninguna».
--
-- Y al dar de baja un equipo, sus procesos **no se van con él**: son historia y la historia no se
-- borra porque se venda la caja. `SET NULL` los deja huérfanos a propósito. Es distinto del
-- `RESTRICT` que la fase 1 le puso al software, y por un motivo claro: el software es contenido
-- vivo que se perdería, un proceso terminado es un registro de lo que pasó.

ALTER TABLE processes
    ADD COLUMN origin_server BIGINT
        REFERENCES hardware(server_id) ON DELETE SET NULL;

-- Lo que ya está en marcha sale de donde vive su programa. Es la única respuesta disponible
-- para el pasado y es la correcta: nadie ha movido nada todavía entre que se lanzaron y ahora.
UPDATE processes p
   SET origin_server = s.server_id
  FROM software s
 WHERE s.id = p.soft_id
   AND p.origin_server IS NULL;

-- Se consulta siempre por equipo —«qué está corriendo aquí»—, nunca al revés.
CREATE INDEX processes_origin_server_idx
    ON processes (origin_server)
    WHERE origin_server IS NOT NULL;
