-- Antesala de la fase 2 de direcciones: la dirección por la que se navega pasa a vivir en una
-- máquina.
--
-- La fase 1 le dio dirección propia a cada equipo, y ahí quedó una cosa a medias que solo se ve
-- mirando los datos: **repartió direcciones nuevas sin reconciliarlas con las que el juego usa
-- para navegar**. El resultado, comprobado antes de escribir esto:
--
--   * el jugador `adm` responde en su `game_ip`, y ninguna de sus dos máquinas tenía esa
--     dirección: tenían otras dos, recién sorteadas;
--   * de los 36 NPC, **ninguno** tenía `public_ip` en su máquina, aunque los 36 responden en
--     `npc_ip` y están expuestos a Internet.
--
-- Y `hardware.public_ip` no lo lee nadie: se escribe al poner una tarjeta de red, se pinta en la
-- pantalla de hardware, y se acabó. Las 55 referencias que navegan van por `game_ip` y `npc_ip`.
--
-- Así que había dos juegos de direcciones sin relación entre sí. Mientras siga así, hacer que
-- `resolve_ip` devuelva el **nodo** es imposible para un jugador: su dirección resoluble no
-- pertenece a ninguna de sus máquinas.
--
-- ## Qué se hace, y qué NO se hace
--
-- **La máquina adopta la dirección, no al revés.** `users.game_ip` y `npc.npc_ip` no se tocan: de
-- ellas cuelgan el perfil, el ranking, `game_evidence.real_origin_ip` y
-- `bank_transactions.origin_ip`. Cambiarlas movería el suelo debajo de todo eso. Lo que se mueve
-- es cuál de las máquinas lleva esa dirección puesta.
--
-- La dirección que la máquina tuviera antes **se libera sola**: `random_free_ip()` considera libre
-- lo que no aparece en `users`, `npc`, `clans` ni `hardware`, así que sobrescribirla la devuelve al
-- fondo común sin más.
--
-- ## Cuál es la máquina principal
--
-- La misma regla que usó la fase 1 para repartir el software: más disco, y se desempata por
-- procesador y memoria. Es la que puede sostener lo que se le eche encima, y la que ya tiene el
-- software del jugador.
--
-- Se prefiere una con tarjeta de red —que es lo que hoy da `public_ip`—, porque la tarjeta es el
-- interruptor de la salida y ponerle dirección pública a un equipo sin ella contradiría la fase 1.
-- Si ninguna la tiene, se coge igualmente la principal: **lo que no puede pasar es que alguien que
-- hoy responde en Internet deje de responder**. Es el caso de los 36 NPC, cuyas máquinas se
-- crearon sin componentes. La fase 5 —esconder algo es dejarlo sin tarjeta— revisará eso cuando
-- exista el pivoting.

--
-- Los NPC: una máquina cada uno, comprobado, así que no hay nada que elegir.
--
UPDATE hardware h
   SET public_ip = n.npc_ip
  FROM npc n
 WHERE h.is_npc
   AND h.user_id = n.id
   AND h.public_ip IS DISTINCT FROM n.npc_ip;

--
-- Los jugadores: la principal se lleva la dirección por la que ya se les navega.
--
WITH principal AS (
    SELECT DISTINCT ON (h.user_id)
           h.server_id,
           u.game_ip
      FROM hardware h
      JOIN users u
        ON u.id = h.user_id
     WHERE NOT h.is_npc
     ORDER BY h.user_id,
              (h.public_ip IS NOT NULL) DESC,
              h.hdd DESC,
              h.cpu DESC,
              h.ram DESC,
              h.server_id
)
UPDATE hardware h
   SET public_ip = p.game_ip
  FROM principal p
 WHERE h.server_id = p.server_id
   AND h.public_ip IS DISTINCT FROM p.game_ip;

--
-- Y la comprobación de que esto no dejó a nadie fuera de Internet. Es el único riesgo real de la
-- migración, así que se mira aquí y no después: quien respondía en una dirección tiene que seguir
-- teniendo una máquina que la lleve.
--
DO $$
DECLARE
    huerfanos bigint;
BEGIN
    SELECT count(*) INTO huerfanos
      FROM (
        SELECT u.game_ip AS ip FROM users u
        UNION ALL
        SELECT n.npc_ip FROM npc n
             JOIN npc_internet_exposure e ON e.npc_id = n.id
      ) alcanzable
     WHERE NOT EXISTS (
        SELECT 1 FROM hardware h WHERE h.public_ip = alcanzable.ip
     );

    IF huerfanos > 0 THEN
        RAISE EXCEPTION
            'la reconciliacion dejo % direcciones sin maquina que las lleve', huerfanos;
    END IF;
END
$$;
