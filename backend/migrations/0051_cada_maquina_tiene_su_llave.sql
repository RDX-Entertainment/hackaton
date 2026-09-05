-- Fase 4 de direcciones: cada máquina tiene su llave, y un proceso sabe a cuál va.
--
-- Las tres fases anteriores le dieron a cada equipo dirección, identidad y cuerpo. Faltaba lo
-- que hacía que todo eso siguiera siendo decorativo: **la contraseña era del dueño**.
--
--   * `users.game_pass` es del jugador, una para toda su flota.
--   * El login remoto comprobaba la del dueño, así que acertarla en cualquiera de sus
--     direcciones abría todas.
--   * Y el crackeo guardaba en `compromised_lists` la dirección **de la principal** y la
--     contraseña de la cuenta, atacaras la máquina que atacaras.
--
-- Así, separar direcciones no servía de nada: crackear una era crackearlo todo. Los NPC ya
-- funcionaban bien —cada uno trae su `npc_pass`—, y eran los jugadores los que tenían el
-- modelo a medias.
--
-- ## La llave baja a la máquina
--
-- Cada equipo hereda la contraseña que ya tenía su dueño, así que **el día que esto entra no
-- deja de funcionar ni un acceso**: quien supiera la de alguien sigue entrando en sus
-- máquinas, y a partir de ahí cada una sigue su camino. Cambiar la de una no cambia las otras.
--
-- Las nuevas nacen con una propia por defecto, con la misma forma que usa el juego desde el
-- legado —ocho caracteres de un md5—, y así ningún sitio que cree una máquina tiene que
-- acordarse de ponerle llave.
--
-- ## Y un proceso apunta a un nodo, no a un dueño
--
-- `processes.victim_id` es el dueño. Mientras un jugador era una caja daba igual; ahora no,
-- porque el crackeo tiene que revelar la llave **de la máquina que se atacó** y no la de la
-- principal. Es la misma pieza que la fase 3 de «cada máquina es un ordenador» puso del otro
-- lado —`origin_server` dice de dónde sale— y las dos juntas cierran la frase: quién ataca,
-- desde qué equipo, y a qué equipo.
--
-- Se rellena con la principal de cada víctima para lo que ya existe. Es la única respuesta
-- disponible para el pasado, y era literalmente la que se estaba usando.

ALTER TABLE hardware
    ADD COLUMN machine_pass VARCHAR(8) NOT NULL
        DEFAULT substr(md5(random()::text), 1, 8);

UPDATE hardware h
   SET machine_pass = u.game_pass
  FROM users u
 WHERE NOT h.is_npc
   AND u.id = h.user_id;

UPDATE hardware h
   SET machine_pass = n.npc_pass
  FROM npc n
 WHERE h.is_npc
   AND n.id = h.user_id;

ALTER TABLE processes
    ADD COLUMN victim_server BIGINT
        REFERENCES hardware(server_id) ON DELETE SET NULL;

-- Lo que ya está en marcha apuntaba, de hecho, a la principal de su víctima: era la única
-- dirección que `victim_ip` sabía resolver. Se deja escrito lo que ya pasaba.
UPDATE processes p
   SET victim_server = h.server_id
  FROM hardware h
 WHERE h.is_gateway
   AND h.user_id = p.victim_id
   AND h.is_npc = (p.info = 'npc')
   AND p.victim_server IS NULL;

CREATE INDEX processes_victim_server_idx
    ON processes (victim_server)
    WHERE victim_server IS NOT NULL;

--
-- La comprobación: nadie se queda sin llave, y ninguna máquina la pierde por el camino.
--
DO $$
DECLARE
    sin_llave bigint;
BEGIN
    SELECT count(*) INTO sin_llave
      FROM hardware
     WHERE machine_pass IS NULL
        OR length(machine_pass) = 0;

    IF sin_llave > 0 THEN
        RAISE EXCEPTION '% maquinas se han quedado sin contrasena', sin_llave;
    END IF;
END
$$;
