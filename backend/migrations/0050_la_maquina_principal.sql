-- Fase 3 de direcciones: la máquina principal, y `game_ip` deja de ir por libre.
--
-- La 0049 puso la dirección navegable dentro de una máquina, pero **cuál de ellas la lleva
-- seguía siendo un accidente**: se dedujo una vez, por una regla de migración, y nada la
-- sostenía después. Eso se nota en cuanto algo se mueve, y ya se notaba en dos sitios:
--
--   * **`reset_ip_effect`** cambiaba `users.game_ip` y no tocaba ninguna máquina. Desde que
--     se resuelve por la máquina, cambiarse de IP dejaba al jugador respondiendo en la vieja
--     y su dirección nueva no llevaba a ninguna parte.
--   * **El alta de un jugador** crea su `Server #1` sin dirección pública —las componentes
--     del kit entran por SQL y no pasan por `assign_public_ip`—, así que un jugador recién
--     registrado tenía `game_ip` y no respondía en ella. Comprobado con un alta de prueba
--     antes de escribir esto: `404`.
--
-- Los dos fallos son el mismo: nadie era responsable de que la dirección de alguien viviera
-- en una de sus máquinas. Esta migración nombra a esa responsable.
--
-- ## Qué es la principal
--
-- La máquina por la que se le conoce a uno: la que lleva puesta la dirección con la que
-- aparece en el ranking, la que firma sus movimientos en el banco y la que se anota en las
-- evidencias. Las demás siguen teniendo la suya y respondiendo por ella —eso es lo que trajo
-- la fase 2—, pero solo una es **la** dirección del dueño.
--
-- Se marca con una columna y se cierra con un índice: **una y solo una por dueño**. Un
-- jugador con dos principales o con ninguna es justo el estado que ha causado los dos fallos
-- de arriba, y la base deja de admitirlo.
--
-- ## `game_ip` sigue existiendo, y ya no se escribe sola
--
-- No se borra: de ella cuelgan el ranking, el perfil, `game_evidence.real_origin_ip` y
-- `bank_transactions.origin_ip`, y 22 consultas la leen. Lo que cambia es que **deja de ser
-- una fuente**: pasa a reflejar la dirección de la principal, y los dos únicos sitios que
-- cambiaban una dirección —el alta y el cambio de IP— ahora mueven la máquina y dejan que
-- `game_ip` siga detrás.

ALTER TABLE hardware
    ADD COLUMN is_gateway BOOLEAN NOT NULL DEFAULT false;

--
-- La que ya lleva la dirección del dueño es la principal. Es lo que dejó la 0049.
--
UPDATE hardware h
   SET is_gateway = true
  FROM users u
 WHERE NOT h.is_npc
   AND u.id = h.user_id
   AND h.public_ip = u.game_ip;

UPDATE hardware h
   SET is_gateway = true
  FROM npc n
 WHERE h.is_npc
   AND n.id = h.user_id
   AND h.public_ip = n.npc_ip;

--
-- Y los que se quedaron sin ninguna: el jugador recién registrado cuya máquina nació sin
-- dirección. Se le nombra principal a la de siempre —más disco, desempatando por procesador y
-- memoria— y se le pone encima la dirección por la que ya se le navega. Sin esto seguiría
-- existiendo sin responder.
--
WITH huerfanos AS (
    SELECT DISTINCT ON (h.user_id)
           h.server_id,
           u.game_ip
      FROM hardware h
      JOIN users u
        ON u.id = h.user_id
     WHERE NOT h.is_npc
       AND NOT EXISTS (
            SELECT 1 FROM hardware g
             WHERE g.user_id = h.user_id
               AND NOT g.is_npc
               AND g.is_gateway
       )
     ORDER BY h.user_id, h.hdd DESC, h.cpu DESC, h.ram DESC, h.server_id
)
UPDATE hardware h
   SET is_gateway = true,
       public_ip  = o.game_ip
  FROM huerfanos o
 WHERE h.server_id = o.server_id;

--
-- Una y solo una por dueño. A partir de aquí el estado que causó los dos fallos no cabe.
--
CREATE UNIQUE INDEX hardware_gateway_idx
    ON hardware (user_id, is_npc)
    WHERE is_gateway;

--
-- Y la comprobación: todo dueño con máquinas tiene su principal, y esa principal lleva puesta
-- la dirección por la que se le navega. Si algo de esto no se cumple, mejor no arrancar.
--
DO $$
DECLARE
    sin_principal bigint;
    descuadradas  bigint;
BEGIN
    SELECT count(*) INTO sin_principal
      FROM (
        SELECT u.id FROM users u
         WHERE EXISTS (SELECT 1 FROM hardware h WHERE h.user_id = u.id AND NOT h.is_npc)
           AND NOT EXISTS (SELECT 1 FROM hardware h
                            WHERE h.user_id = u.id AND NOT h.is_npc AND h.is_gateway)
        UNION ALL
        SELECT n.id FROM npc n
         WHERE EXISTS (SELECT 1 FROM hardware h WHERE h.user_id = n.id AND h.is_npc)
           AND NOT EXISTS (SELECT 1 FROM hardware h
                            WHERE h.user_id = n.id AND h.is_npc AND h.is_gateway)
      ) t;

    IF sin_principal > 0 THEN
        RAISE EXCEPTION '% duenos con maquinas se han quedado sin principal', sin_principal;
    END IF;

    SELECT count(*) INTO descuadradas
      FROM hardware h
      LEFT JOIN users u ON u.id = h.user_id AND NOT h.is_npc
      LEFT JOIN npc   n ON n.id = h.user_id AND h.is_npc
     WHERE h.is_gateway
       AND h.public_ip IS DISTINCT FROM COALESCE(u.game_ip, n.npc_ip);

    IF descuadradas > 0 THEN
        RAISE EXCEPTION
            '% maquinas principales no llevan la direccion de su dueno', descuadradas;
    END IF;
END
$$;
