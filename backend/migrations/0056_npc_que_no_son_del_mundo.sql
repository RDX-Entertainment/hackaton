-- NPC que no son del mundo: el aula del tutorial y el campo de tiro.
--
-- Los 36 NPC sembrados son el mundo abierto: los ve todo el que juega y estan ahi para
-- siempre. Faltaban dos clases de maquina que no son eso.
--
-- ## El aula: direcciones que se ven de lejos que no son reales
--
-- El tutorial necesita un objetivo, y hasta ahora la unica opcion era señalar uno de los 36.
-- Funciona, pero deja al aprendiz practicando sobre una maquina indistinguible de las otras
-- 35 y sin ninguna forma de decir «esto era el entrenamiento y ya termino».
--
-- `0.0.0.1` no es una direccion que pueda tener nada real, y `random_free_ip()` no la va a
-- repartir jamas porque solo sortea entre bloques publicos. Eso la hace segura por
-- construccion y, mejor todavia, **se lee sola**: cualquiera entiende sin que se lo expliquen
-- que `0.0.0.1` es un simulacro. Una por mision.
--
-- ## El campo de tiro: dos maquinas por jugador, suyas
--
-- Sitio donde probar un programa nuevo sin buscarse un enemigo de verdad. Se crean en el alta
-- y no caducan. Y **solo las ve su dueño**: dos por jugador en el listin comun lo llenarian de
-- maquinas que a nadie mas le sirven.
--
-- ## Por que una tabla y no una columna en `npc`
--
-- Porque lo que comparten no es un atributo del NPC sino una pregunta sobre quien mira: **hay
-- NPC que no todo el mundo ve**. Un NPC sin fila aqui es del mundo abierto, que es lo que son
-- los 36 de siempre, y por eso no hay que tocarlos ni rellenarles nada.

CREATE TABLE npc_purpose (
    npc_id   BIGINT PRIMARY KEY REFERENCES npc(id) ON DELETE CASCADE,
    kind     VARCHAR(16) NOT NULL CHECK (kind IN ('tutorial', 'practice')),

    -- Del campo de tiro. Si se va el jugador se van sus maquinas: no le sirven a nadie mas.
    owner_id BIGINT REFERENCES users(id) ON DELETE CASCADE,

    -- Del aula: que mision del tutorial. Es lo que un dia decidira si sigue respondiendo.
    step     SMALLINT,

    -- Cada clase tiene su forma, y se dice aqui en vez de confiar en que nadie la mezcle.
    CONSTRAINT npc_purpose_coherente CHECK (
        (kind = 'practice' AND owner_id IS NOT NULL AND step IS NULL)
     OR (kind = 'tutorial' AND owner_id IS NULL     AND step IS NOT NULL)
    )
);

CREATE INDEX npc_purpose_owner_idx ON npc_purpose (owner_id) WHERE owner_id IS NOT NULL;

--
-- Quien ve que.
--
-- En una funcion y no repartido por las consultas porque son dos las que preguntan —resolver
-- una direccion y listar el listin— y el dia que el aula se cierre al terminar la mision, esa
-- regla se escribe una vez.
--
CREATE OR REPLACE FUNCTION npc_visible_to(npc BIGINT, who BIGINT) RETURNS BOOLEAN AS $$
  SELECT CASE
    -- Sin fila: mundo abierto. Los 36 de siempre.
    WHEN p.npc_id IS NULL      THEN TRUE
    -- El campo de tiro es de quien es.
    WHEN p.kind = 'practice'   THEN p.owner_id = who
    -- El aula todavia la ve todo el mundo. Cerrarla es cosa del tutorial, que aun no existe.
    ELSE TRUE
  END
  FROM (SELECT npc AS id) q
  LEFT JOIN npc_purpose p ON p.npc_id = q.id;
$$ LANGUAGE sql STABLE;

--
-- Los ids se reparten en bloques para que no se pisen: el aula ocupa el 900000, y lo que
-- genere la secuencia a partir de ahora empieza en el millon. Los 36 del mundo abierto se
-- insertaron con id explicito (1001..1036) y dejaron la secuencia sin tocar, apuntando al 1:
-- el primer NPC generado habria acabado chocando con ellos tarde o temprano.
--
SELECT setval(pg_get_serial_sequence('npc', 'id'), 1000000, FALSE);

--
-- El aula de la mision 1.
--
INSERT INTO npc (id, npc_type, npc_ip, npc_pass)
VALUES (900001, 90, 1, substr(md5('tutorial:1'), 1, 8))
ON CONFLICT (id) DO NOTHING;

INSERT INTO npc_info (npc_id, name, web) VALUES (900001,
    'Aula 1 · Almacen Ortega',
    '<h1>Almacen Ortega</h1>'
    '<p>Distribucion de material de oficina desde 1994.</p>'
    '<p>Nuestra web esta en obras. Para pedidos, llame al almacen.</p>')
ON CONFLICT (npc_id) DO NOTHING;

INSERT INTO npc_purpose (npc_id, kind, step) VALUES (900001, 'tutorial', 1)
ON CONFLICT (npc_id) DO NOTHING;

INSERT INTO hardware (user_id, name, cpu, ram, hdd, net, is_npc, is_gateway, public_ip, machine_pass)
SELECT n.id, 'npc', 1000, 256, 400, 10, TRUE, TRUE, n.npc_ip, n.npc_pass
  FROM npc n
 WHERE n.id = 900001
   AND NOT EXISTS (SELECT 1 FROM hardware h WHERE h.is_npc AND h.user_id = n.id);

INSERT INTO log (user_id, text, is_npc) SELECT 900001, '', TRUE
 WHERE NOT EXISTS (SELECT 1 FROM log WHERE is_npc AND user_id = 900001);

-- Lo que hay dentro. Un cracker mejor del que puede fabricarse el primer dia: el sexto paso
-- del tutorial es llevarse un programa, y llevarse algo peor que lo tuyo no enseña nada.
INSERT INTO software_original (npc_id, soft_name, soft_version, soft_size, soft_ram, soft_type) VALUES
    (900001, 'Cracker',  20, 30, 100, 1),
    (900001, 'Firewall', 10, 50, 150, 4)
ON CONFLICT DO NOTHING;

INSERT INTO software (user_id, soft_name, soft_version, soft_size, soft_ram, soft_type, is_npc, server_id)
SELECT o.npc_id, o.soft_name, o.soft_version, o.soft_size, o.soft_ram, o.soft_type, TRUE, h.server_id
  FROM software_original o
  JOIN hardware h ON h.user_id = o.npc_id AND h.is_npc
 WHERE o.npc_id = 900001
   AND NOT EXISTS (SELECT 1 FROM software s WHERE s.is_npc AND s.user_id = o.npc_id
                     AND s.soft_name = o.soft_name AND s.soft_version = o.soft_version);

-- Y su pagina, en la tabla editable. La 0041 materializo ahi las webs que el juego ya servia,
-- pero solo las que existian ese dia: todo NPC creado despues nace con `npc_info.web` y sin su
-- fila, y quien entre a mirarla no encuentra nada. Se hace aqui lo mismo que hizo la 0041.
INSERT INTO machine_web_content (target_user_id, target_is_npc, path, content)
SELECT i.npc_id, TRUE, '/index.html', i.web
  FROM npc_info i
 WHERE i.npc_id = 900001 AND i.web <> ''
ON CONFLICT (target_user_id, target_is_npc, path) DO NOTHING;

--
-- El campo de tiro de quien ya estaba. Quien se registre a partir de ahora lo recibe en el
-- alta; esto es para los que se registraron antes de que existiera.
--
DO $$
DECLARE
    jugador  bigint;
    tiene    int;
    numero   int;
    nuevo    bigint;
    maquina  bigint;
BEGIN
    FOR jugador IN SELECT id FROM users LOOP
        SELECT count(*) INTO tiene
          FROM npc_purpose
         WHERE kind = 'practice' AND owner_id = jugador;

        FOR numero IN (tiene + 1)..2 LOOP
            INSERT INTO npc (npc_type, npc_ip, npc_pass)
            VALUES (80, random_free_ip(), substr(md5(random()::text), 1, 8))
            RETURNING id INTO nuevo;

            INSERT INTO npc_info (npc_id, name)
            VALUES (nuevo, 'Campo de tiro ' || numero);

            INSERT INTO npc_purpose (npc_id, kind, owner_id)
            VALUES (nuevo, 'practice', jugador);

            INSERT INTO hardware (user_id, name, cpu, ram, hdd, net, is_npc, is_gateway, public_ip, machine_pass)
            SELECT nuevo, 'npc', 800, 256, 300, 10, TRUE, TRUE, n.npc_ip, n.npc_pass
              FROM npc n WHERE n.id = nuevo
            RETURNING server_id INTO maquina;

            INSERT INTO log (user_id, text, is_npc) VALUES (nuevo, '', TRUE);

            INSERT INTO software_original (npc_id, soft_name, soft_version, soft_size, soft_ram, soft_type)
            VALUES (nuevo, 'Firewall', 10 * numero, 50, 150, 4);

            INSERT INTO software (user_id, soft_name, soft_version, soft_size, soft_ram, soft_type, is_npc, server_id)
            VALUES (nuevo, 'Firewall', 10 * numero, 50, 150, 4, TRUE, maquina);
        END LOOP;
    END LOOP;
END
$$;

COMMENT ON TABLE npc_purpose IS
    'NPC que no todo el mundo ve: el aula del tutorial y el campo de tiro de cada jugador. Un NPC sin fila aqui es del mundo abierto.';
