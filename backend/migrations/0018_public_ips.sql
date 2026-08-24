-- Las direcciones del juego eran un entero cualquiera entre 1 y 2^32-1, asi que al
-- imprimirlas salian numeros sin sentido y, si se leian como IPv4, caian tan a menudo en
-- rangos privados o reservados (10/8, 127/8, 192.168/16, multicast) como en el espacio
-- publico. Aqui pasan a sortearse solo dentro de bloques /8 realmente asignados y
-- enrutables, para que cualquier direccion del juego se lea como una IP publica de verdad.
--
-- El almacenamiento no cambia: sigue siendo el mismo BIGINT con la IPv4 empaquetada en 32
-- bits, que es como lo hacia el juego original con ip2long(). Lo unico que cambia es de
-- donde se sortea, y que ahora hay una sola definicion -- esta -- en vez de una en el API
-- y otra en el worker.

-- Bloques /8 asignados y enrutables. Se excluye 172.16/12 por ser RFC1918: es el unico
-- rango privado que cae dentro de un bloque de esta lista.
CREATE OR REPLACE FUNCTION random_public_ip() RETURNS BIGINT AS $$
DECLARE
  blocks INT[] := ARRAY[
    23, 45, 51, 63, 72, 91, 96, 104, 138, 142,
    151, 159, 162, 172, 176, 185, 188, 193, 198, 201, 208, 213, 216
  ];
  block INT;
  b2 INT;
  b3 INT;
  b4 INT;
BEGIN
  LOOP
    block := blocks[1 + floor(random() * array_length(blocks, 1))::INT];
    b2 := floor(random() * 256)::INT;
    b3 := floor(random() * 256)::INT;
    -- Ni .0 ni .255: son la red y el broadcast, y ninguna maquina responde ahi.
    b4 := 1 + floor(random() * 254)::INT;

    EXIT WHEN NOT (block = 172 AND b2 BETWEEN 16 AND 31);
  END LOOP;

  RETURN (block::BIGINT << 24) | (b2::BIGINT << 16) | (b3::BIGINT << 8) | b4::BIGINT;
END;
$$ LANGUAGE plpgsql VOLATILE;

-- Una direccion publica que no tenga ya ningun jugador, NPC ni clan. El API la usa al dar
-- de alta y el worker al cambiar de IP, de modo que la regla vive en un solo sitio.
CREATE OR REPLACE FUNCTION random_free_ip() RETURNS BIGINT AS $$
DECLARE
  candidate BIGINT;
BEGIN
  LOOP
    candidate := random_public_ip();

    EXIT WHEN NOT EXISTS (SELECT 1 FROM users WHERE game_ip = candidate)
          AND NOT EXISTS (SELECT 1 FROM npc   WHERE npc_ip  = candidate)
          AND NOT EXISTS (SELECT 1 FROM clans WHERE clan_ip = candidate);
  END LOOP;

  RETURN candidate;
END;
$$ LANGUAGE plpgsql VOLATILE;

-- Reasignacion de lo que ya existe.
--
-- Seis tablas mas guardan direcciones sueltas que apuntan a un jugador o a un NPC
-- (favoritos, historial, accesos, escaneos, analisis y la lista de comprometidos). Si solo
-- se reescribieran users y npc, todas esas filas quedarian apuntando a direcciones que ya
-- no existen. Por eso se construye primero un mapa vieja -> nueva y se aplica a todas.
CREATE TEMP TABLE ip_remap (
  old_ip BIGINT PRIMARY KEY,
  new_ip BIGINT
) ON COMMIT DROP;

INSERT INTO ip_remap (old_ip)
SELECT game_ip FROM users
UNION
SELECT npc_ip FROM npc
UNION
SELECT clan_ip FROM clans;

DO $$
DECLARE
  r RECORD;
  candidate BIGINT;
BEGIN
  FOR r IN SELECT old_ip FROM ip_remap ORDER BY old_ip LOOP
    LOOP
      candidate := random_public_ip();

      -- Tampoco puede chocar con una direccion vieja todavia sin reasignar, o la
      -- actualizacion violaria transitoriamente el UNIQUE de npc_ip.
      EXIT WHEN NOT EXISTS (
        SELECT 1 FROM ip_remap WHERE new_ip = candidate OR old_ip = candidate
      );
    END LOOP;

    UPDATE ip_remap SET new_ip = candidate WHERE old_ip = r.old_ip;
  END LOOP;
END;
$$;

UPDATE users              u SET game_ip = m.new_ip FROM ip_remap m WHERE u.game_ip = m.old_ip;
UPDATE npc                n SET npc_ip  = m.new_ip FROM ip_remap m WHERE n.npc_ip  = m.old_ip;
UPDATE clans              c SET clan_ip = m.new_ip FROM ip_remap m WHERE c.clan_ip = m.old_ip;
UPDATE compromised_lists  t SET ip      = m.new_ip FROM ip_remap m WHERE t.ip      = m.old_ip;
UPDATE internet_important t SET ip      = m.new_ip FROM ip_remap m WHERE t.ip      = m.old_ip;
UPDATE internet_history   t SET ip      = m.new_ip FROM ip_remap m WHERE t.ip      = m.old_ip;
UPDATE internet_access    t SET ip      = m.new_ip FROM ip_remap m WHERE t.ip      = m.old_ip;
UPDATE internet_scan      t SET ip      = m.new_ip FROM ip_remap m WHERE t.ip      = m.old_ip;
UPDATE analyze_results    t SET ip      = m.new_ip FROM ip_remap m WHERE t.ip      = m.old_ip;

-- El worker ya trataba game_ip como unica al sortear una nueva; ahora lo dice el esquema.
ALTER TABLE users ADD CONSTRAINT users_game_ip_key UNIQUE (game_ip);
