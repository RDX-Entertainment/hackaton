-- Software inicial para la cuenta de administrador.
--
-- La cuenta que crea el arranque (`ADMIN_LOGIN` del .env) nacía con el disco
-- vacío: cero filas en `software`. Eso deja la pantalla de Software sin nada que
-- listar y la hoja de atributos sin nada que dibujar, así que el sistema modular
-- —que es lo que se está construyendo— no se puede ni mirar sin escribir antes
-- ocho ficheros a mano.
--
-- Cada programa lleva su fila en `software_profiles`, que es donde viven los
-- doce atributos. Sin perfil, un software existe pero no sabe hacer nada: los
-- valores por defecto de la tabla dejarían a todos iguales, y lo que hay que ver
-- en pantalla es justo lo contrario —que un Firewall y un Hidder no se parecen.
--
-- `origin = 'legacy'` porque no salen de research ni de un botín: son el equipo
-- con el que se empieza.
--
-- Idempotente por partida doble: no hace nada si no existe la cuenta, y no
-- duplica si ya tiene software. Una migración de semilla que se ejecuta dos
-- veces no puede dejar el disco con dos Crackers.

DO $$
DECLARE
  admin_id  bigint;
  nuevo_id  bigint;
  fila      record;
BEGIN
  SELECT id INTO admin_id FROM users ORDER BY id LIMIT 1;

  IF admin_id IS NULL THEN
    RAISE NOTICE 'sin cuentas todavia: no hay a quien darle software';
    RETURN;
  END IF;

  IF EXISTS (SELECT 1 FROM software WHERE user_id = admin_id AND NOT is_npc) THEN
    RAISE NOTICE 'la cuenta % ya tiene software: no se toca', admin_id;
    RETURN;
  END IF;

  -- Los ocho de partida. El reparto de atributos es lo que los diferencia: cada
  -- uno destaca en la fase en la que actúa y es flojo en las demás, para que la
  -- hoja de atributos enseñe una silueta distinta por programa y no doce barras
  -- iguales.
  --
  -- ataque:  effectiveness penetration stealth manipulation anti_forensics persistence
  -- defensa: resistance    hardening   detection integrity   forensics      removal
  FOR fila IN
    SELECT * FROM (VALUES
      --  nombre         tipo size ram  eff pen ste man anf per  res har det int for rem  sta cpu ram_n net wgt
      ('Cracker',        1,  300, 128,  78, 85, 30, 20, 25, 15,   10, 10, 15, 10, 10, 10,  70, 40, 128, 20, 3),
      ('Firewall',       4,  400, 256,  20, 10, 15, 10, 10, 60,   88, 82, 45, 70, 35, 30,  85, 55, 256, 15, 4),
      ('Hidder',         5,  250, 128,  35, 30, 90, 25, 82, 55,   15, 20, 10, 15, 10, 10,  65, 35, 128, 10, 2),
      ('Seeker',         6,  260, 128,  40, 20, 20, 15, 15, 20,   25, 20, 88, 30, 80, 45,  72, 38, 128, 25, 2),
      ('Antivirus',      7,  350, 256,  45, 15, 10, 20, 12, 25,   60, 55, 70, 65, 62, 90,  80, 50, 256, 12, 3),
      ('Port Scan',      3,  120,  64,  55, 35, 45, 10, 20, 10,   10, 10, 30, 10, 15, 10,  75, 20,  64, 40, 1),
      ('Nmap',          15,  180,  96,  62, 40, 50, 15, 22, 12,   12, 15, 55, 12, 25, 10,  78, 28,  96, 55, 2),
      ('Analyzer',      16,  220, 128,  50, 25, 25, 85, 18, 20,   20, 25, 40, 78, 45, 20,  68, 42, 128, 18, 2)
    ) AS t(nombre, tipo, size, ram,
           eff, pen, ste, man, anf, per,
           res, har, det, int_, fore, rem,
           sta, cpu, ram_n, net, wgt)
  LOOP
    -- `soft_version` va en décimas: 10 es la 1.0, que es con lo que se empieza.
    INSERT INTO software (user_id, soft_name, soft_version, soft_size, soft_ram, soft_type)
    VALUES (admin_id, fila.nombre, 10, fila.size, fila.ram, fila.tipo::smallint)
    RETURNING id INTO nuevo_id;

    INSERT INTO software_profiles (
      software_id, origin,
      effectiveness, penetration, stealth, manipulation, anti_forensics, persistence,
      resistance, hardening, detection, integrity, forensics, removal,
      stability, cpu_need, ram_need, net_need, weight
    ) VALUES (
      nuevo_id, 'legacy',
      fila.eff, fila.pen, fila.ste, fila.man, fila.anf, fila.per,
      fila.res, fila.har, fila.det, fila.int_, fila.fore, fila.rem,
      fila.sta, fila.cpu, fila.ram_n, fila.net, fila.wgt
    );
  END LOOP;

  RAISE NOTICE 'software de partida entregado a la cuenta %', admin_id;
END $$;
