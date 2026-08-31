-- Fase 1 de «cada máquina es un ordenador»: el software aterriza en un disco concreto.
--
-- Hasta aquí un programa colgaba de `user_id` y de nadie más. No estaba en `Server #1` ni en
-- `Servidor`: estaba «en el jugador». Y el sitio para guardarlo salía de sumar el disco de
-- todas sus máquinas, así que meter un disco en cualquier caja daba espacio a todo lo demás.
--
-- Eso no es tener ordenadores, es tener un depósito y unas cajas que le suman. El propio
-- código lo sabía: `MAX_MACHINES = 4` está justificado como tapadera de esto mismo —«comprar
-- equipos sería una forma de sumar capacidad sin límite»—, y al arreglarlo se queda sin
-- motivo.
--
-- ## Dónde aterriza cada cosa
--
-- **Del jugador**, en su máquina con más espacio y potencia. Es una decisión que se toma una
-- sola vez y se nota, así que se elige la que puede sostenerlo: ordenar por disco y desempatar
-- por procesador y memoria deja el software donde cabe y donde corre.
--
-- **De los NPC**, en la suya. Comprobado antes de escribir esto: hay 36 NPC con software, los
-- 36 tienen máquina, y ninguno tiene más de una. No hay caso ambiguo que resolver.
--
-- ## `ON DELETE RESTRICT`, y no CASCADE
--
-- Un equipo que se da de baja con software dentro no debe llevárselo por delante. Hoy
-- `scrap_machine` ya exige que esté vacío de piezas por una razón parecida —vaciarlo pieza a
-- pieza es lo que comprueba que nada se quede fuera—, y el software merece el mismo trato: o
-- se mueve a otro equipo o se borra a mano. Un CASCADE aquí borraría en silencio lo que costó
-- conseguir.

ALTER TABLE software
  ADD COLUMN IF NOT EXISTS server_id bigint;

-- El software del jugador, a su mejor máquina.
UPDATE software s
SET server_id = (
  SELECT h.server_id
  FROM hardware h
  WHERE h.user_id = s.user_id
    AND NOT h.is_npc
  ORDER BY h.hdd DESC, h.cpu DESC, h.ram DESC, h.server_id
  LIMIT 1
)
WHERE s.server_id IS NULL
  AND NOT s.is_npc;

-- El de cada NPC, a la suya.
UPDATE software s
SET server_id = (
  SELECT h.server_id
  FROM hardware h
  WHERE h.user_id = s.user_id
    AND h.is_npc
  ORDER BY h.server_id
  LIMIT 1
)
WHERE s.server_id IS NULL
  AND s.is_npc;

-- Si algo quedó sin sitio es que su dueño no tiene ninguna máquina, y eso es un estado que no
-- deberia existir: mejor detenerse aqui que dejar software flotando sin disco.
DO $$
DECLARE
  huerfanos bigint;
BEGIN
  SELECT count(*) INTO huerfanos FROM software WHERE server_id IS NULL;

  IF huerfanos > 0 THEN
    RAISE EXCEPTION 'quedan % programas sin maquina donde ponerlos', huerfanos;
  END IF;
END $$;

ALTER TABLE software
  ALTER COLUMN server_id SET NOT NULL;

ALTER TABLE software
  ADD CONSTRAINT software_server_id_fkey
  FOREIGN KEY (server_id) REFERENCES hardware(server_id) ON DELETE RESTRICT;

-- Lo que se pregunta a esta tabla a partir de ahora es «qué hay en este disco», y ya no
-- «qué tiene este jugador».
CREATE INDEX IF NOT EXISTS software_server_idx ON software (server_id);

COMMENT ON COLUMN software.server_id IS
  'En que maquina vive el programa. El espacio se cuenta contra el disco de esa, no contra la suma de las del dueno.';
