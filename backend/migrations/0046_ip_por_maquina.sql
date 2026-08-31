-- Fase 1 del rediseño de direcciones: la IP baja de la cuenta a la máquina.
--
-- Hasta aquí la dirección vivía en `users.game_ip` y `hardware` no tenía ninguna, que es
-- como lo hacía el juego original —comprobado en su `game.sql`: allí la tabla `hardware`
-- tampoco llevaba dirección—. Eso valía mientras un jugador fuese una caja. Con varias
-- máquinas por cuenta y un mapa que las dibuja, no: dos equipos con tarjeta de red se
-- pintaban los dos con salida y una sola dirección entre ambos, y el modelo no sabía decir
-- a cuál llegaba un ataque.
--
-- ## Dos direcciones, y cada una dice una cosa
--
-- `public_ip` es cómo se te alcanza desde fuera, y **solo la tiene quien lleva tarjeta de
-- red puesta**. Es nullable a propósito: el `NULL` no es un hueco por rellenar, es la
-- respuesta —esta máquina no sale a Internet—. Es la misma regla que ya aplica el mapa para
-- dibujar la línea al nodo INTERNET y para contar la boca que ocupa, así que la tarjeta pasa
-- a ser el interruptor de las tres cosas a la vez.
--
-- `lan_ip` la tienen todas, con o sin tarjeta, y es cómo se las nombra **desde dentro de la
-- red del jugador**. Sin ella no se puede pivotar: para saltar del equipo expuesto al que no
-- lo está hace falta poder apuntar a algo, y un cable que no lleva a ninguna dirección es un
-- dibujo. Va en 10/8, que es privado de verdad y por eso `random_public_ip()` nunca lo
-- sortea: no hay forma de que una LAN choque con una dirección pública del juego.
--
-- ## Lo que esta fase NO toca
--
-- `users.game_ip` se queda como está y sigue significando lo mismo. De ella cuelgan el
-- perfil, el ranking, `game_evidence.real_origin_ip` y `bank_transactions.origin_ip`, y
-- moverla ahora sería cambiar la identidad del jugador antes de tener dónde apoyarla. Eso es
-- la fase 3, la del gateway. Hasta entonces conviven: la cuenta tiene su dirección y las
-- máquinas la suya.

ALTER TABLE hardware
  ADD COLUMN IF NOT EXISTS public_ip bigint,
  ADD COLUMN IF NOT EXISTS lan_ip    bigint;

-- Única en toda la partida, como `npc_ip`. El índice es parcial porque el `NULL` significa
-- «sin salida» y de esos hay tantos como se quiera: un UNIQUE normal ya los deja pasar, pero
-- el parcial además no los indexa.
CREATE UNIQUE INDEX IF NOT EXISTS hardware_public_ip_idx
  ON hardware (public_ip) WHERE public_ip IS NOT NULL;

-- La privada solo tiene que ser única dentro de la red de su dueño: dos jugadores pueden
-- tener cada uno su 10.0.0.2 igual que dos casas repiten el 192.168.1.1.
CREATE UNIQUE INDEX IF NOT EXISTS hardware_lan_ip_idx
  ON hardware (user_id, lan_ip) WHERE lan_ip IS NOT NULL;

COMMENT ON COLUMN hardware.public_ip IS
  'Como se alcanza desde fuera. NULL = sin tarjeta de red, no sale a Internet.';
COMMENT ON COLUMN hardware.lan_ip IS
  'Como se la nombra dentro de la red de su dueno. La tienen todas, en 10/8.';

-- ── `random_free_ip()` ahora mira cuatro tablas ─────────────────────────────
--
-- Faltaba `hardware`, que hasta hoy no tenia direcciones. Sin esto, la primera maquina que
-- recibiera una podria pisar la de un NPC o la de un jugador, y dos cosas distintas
-- respondiendo a la misma direccion es un fallo que aparece tarde y se depura peor.
CREATE OR REPLACE FUNCTION random_free_ip() RETURNS BIGINT AS $$
DECLARE
  candidate BIGINT;
BEGIN
  LOOP
    candidate := random_public_ip();

    EXIT WHEN NOT EXISTS (SELECT 1 FROM users    WHERE game_ip   = candidate)
          AND NOT EXISTS (SELECT 1 FROM npc      WHERE npc_ip    = candidate)
          AND NOT EXISTS (SELECT 1 FROM clans    WHERE clan_ip   = candidate)
          AND NOT EXISTS (SELECT 1 FROM hardware WHERE public_ip = candidate);
  END LOOP;

  RETURN candidate;
END;
$$ LANGUAGE plpgsql VOLATILE;

-- ── Reparto para lo que ya existe ───────────────────────────────────────────

-- Una privada para cada maquina, numeradas por dueno desde 10.0.0.1. El `10 << 24` es la
-- misma IPv4 empaquetada en un BIGINT que usa el resto del juego.
WITH numeradas AS (
  SELECT server_id,
         row_number() OVER (PARTITION BY user_id ORDER BY server_id) AS n
  FROM hardware
  -- Los NPC quedan fuera: su direccion es `npc.npc_ip` y su red la modelan
  -- `organization_networks` y `organization_node_links`. Darles ademas una privada aqui
  -- seria una segunda direccion para lo mismo, y dos sitios donde mirar.
  WHERE lan_ip IS NULL
    AND NOT is_npc
)
UPDATE hardware h
SET lan_ip = (10::bigint << 24) | numeradas.n
FROM numeradas
WHERE h.server_id = numeradas.server_id;

-- Y una publica a las que ya llevan tarjeta de red puesta. Se sortea de una en una porque
-- `random_free_ip()` comprueba contra la propia tabla: un UPDATE masivo evaluaria todas las
-- candidatas contra el estado anterior y podria repetir.
DO $$
DECLARE
  maquina bigint;
BEGIN
  FOR maquina IN
    SELECT DISTINCT c.installed_in
    FROM hardware_components c
    JOIN hardware_parts p ON p.id = c.part_id
    WHERE p.kind = 'net'
      AND c.installed_in IS NOT NULL
      AND EXISTS (
        SELECT 1 FROM hardware h
        WHERE h.server_id = c.installed_in
          AND NOT h.is_npc
      )
  LOOP
    UPDATE hardware
    SET public_ip = random_free_ip()
    WHERE server_id = maquina
      AND public_ip IS NULL;
  END LOOP;
END $$;
