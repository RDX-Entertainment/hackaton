-- Un kit de servidor completo, y una máquina montada con él para la cuenta admin.
--
-- El kit 3 («Corp test - servidor sin salida») deja fuera la tarjeta de red a
-- propósito, para probar un nodo que solo habla por LAN. Este es el otro caso: la
-- placa de servidor con **todas** sus capacidades saturadas, que es lo que hace
-- falta para ver el techo del hardware sin ir comprando pieza a pieza.
--
-- ## Qué quiere decir «al 100%»
--
-- No es llenar las ranuras. `fitting::check` impone dos límites por tipo de
-- pieza y el que manda es el más bajo de los dos:
--
--   ranuras:  used_slots >= board.slots(kind)        -> NO_SLOTS
--   potencia: used_power + piece.power > ceiling     -> OVER_CAPACITY
--
-- Sobre la Xeon C621 (tier server, DDR4, chipset 5) con las piezas de gama
-- servidor, el reparto sale así:
--
--   tipo  ranuras  tope     pieza                 potencia  caben  total    tope
--   cpu   2        16000    Xeon Gold 6248        8000      2      16000  = 16000
--   ram   8        262144   ECC Registered 32G    32768     8      262144 = 262144
--   hdd   12       400000   NVMe Array 50T        50000     8      400000 = 400000
--   net   4        10000    Fibre 10G             10000     1      10000  = 10000
--
-- En cpu y ram el límite son las ranuras; en hdd y net es la potencia —caben 12
-- discos y 4 tarjetas, pero la placa no admite tanto—. Poner un disco más daría
-- OVER_CAPACITY, así que 20 piezas es el máximo real: los cuatro topes quedan
-- clavados al 100%, ni uno por debajo ni uno por encima.
--
-- La máquina de arranque de la cuenta no se toca: esta es una segunda, para poder
-- comparar la de partida con el techo.
--
-- Idempotente: no duplica el kit ni la máquina si ya existen.

DO $$
DECLARE
  admin_id  bigint;
  -- `v_kit` y no `kit_id`: PL/pgSQL resuelve un nombre suelto contra la variable
  -- **y** contra la columna, y `WHERE kit_id = kit_id` no significa nada.
  v_kit     bigint;
  maquina   bigint;
  pieza     record;
  i         int;
BEGIN
  SELECT id INTO admin_id FROM users ORDER BY id LIMIT 1;

  IF admin_id IS NULL THEN
    RAISE NOTICE 'sin cuentas todavia: no hay a quien darle el kit';
    RETURN;
  END IF;

  -- ── El kit, como objeto del catálogo ──────────────────────────────────────
  SELECT id INTO v_kit FROM hardware_kits WHERE name = 'Servidor completo';

  IF v_kit IS NULL THEN
    -- El precio es la suma de sus piezas, sin recargo: un kit es una comodidad,
    -- no un producto distinto.
    INSERT INTO hardware_kits (name, description, price, scope, is_default, listed)
    VALUES (
      'Servidor completo',
      'La placa de servidor con sus cuatro capacidades saturadas: 2 Xeon Gold, 256 GB ECC, 400 TB NVMe y la fibra de 10G. No admite una pieza mas.',
      1022000,
      'marketplace',
      false,
      true
    )
    RETURNING id INTO v_kit;

    INSERT INTO hardware_kit_items (kit_id, part_id, amount) VALUES
      (v_kit, 36, 1),   -- mobo  Xeon C621
      (v_kit, 38, 2),   -- cpu   Xeon Gold 6248
      (v_kit, 39, 8),   -- ram   ECC Registered 32G
      (v_kit, 40, 8),   -- hdd   NVMe Array 50T
      (v_kit, 41, 1);   -- net   Fibre 10G
  END IF;

  -- ── La máquina montada, para la cuenta admin ──────────────────────────────
  IF EXISTS (SELECT 1 FROM hardware WHERE user_id = admin_id AND name = 'Servidor') THEN
    RAISE NOTICE 'la cuenta % ya tiene la maquina Servidor: no se toca', admin_id;
    RETURN;
  END IF;

  INSERT INTO hardware (user_id, name, cpu, hdd, ram, net)
  VALUES (admin_id, 'Servidor', 0, 0, 0, 0)
  RETURNING server_id INTO maquina;

  -- Las piezas del kit, ya instaladas. `installed_in` es lo que las pone dentro;
  -- una componente sin eso existe pero esta en el cajon.
  FOR pieza IN
    SELECT part_id, amount FROM hardware_kit_items WHERE kit_id = v_kit
  LOOP
    FOR i IN 1..pieza.amount LOOP
      INSERT INTO hardware_components (part_id, user_id, installed_in)
      VALUES (pieza.part_id, admin_id, maquina);
    END LOOP;
  END LOOP;

  -- Los totales de la maquina son la suma de la potencia de lo que lleva dentro.
  -- Es la misma consulta que `repo::recompute_machine`, y tiene que seguir
  -- siendolo: si alli cambia la formula, aqui queda una maquina que miente.
  UPDATE hardware h
  SET cpu = COALESCE(s.cpu, 0),
      ram = COALESCE(s.ram, 0),
      hdd = COALESCE(s.hdd, 0),
      net = COALESCE(s.net, 0)
  FROM (
    SELECT
      sum(k.power) FILTER (WHERE k.kind = 'cpu') AS cpu,
      sum(k.power) FILTER (WHERE k.kind = 'ram') AS ram,
      sum(k.power) FILTER (WHERE k.kind = 'hdd') AS hdd,
      sum(k.power) FILTER (WHERE k.kind = 'net') AS net
    FROM hardware_components c
    JOIN hardware_parts k ON k.id = c.part_id
    WHERE c.installed_in = maquina
  ) s
  WHERE h.server_id = maquina;

  RAISE NOTICE 'servidor completo entregado a la cuenta %', admin_id;
END $$;
