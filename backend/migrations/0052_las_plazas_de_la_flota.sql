-- Fase 4 de «cada máquina es un ordenador»: los dos techos, y lo que se compra es la plaza.
--
-- `MAX_MACHINES = 4` está justificado en el código con estas palabras: *«sin techo, comprar
-- equipos sería una forma de sumar capacidad sin límite: lo que cuenta el juego es la suma de
-- todas las máquinas de alguien, no la mejor»*. Era verdad, y era una tapadera: existía para
-- ocultar que el hardware se sumaba entre equipos.
--
-- **La fase 1 quitó eso de en medio** —el disco bajó a la máquina, y detrás la memoria y el
-- procesador— así que el límite se quedó sin motivo. Cuatro equipos ya no son cuatro veces la
-- potencia de uno; son cuatro sitios distintos donde estar, con sus propias cuentas.
--
-- ## Dos techos que hacen cosas distintas
--
-- **El del servidor** dice cuántas plazas admite el juego, y no es una regla de juego sino de
-- capacidad: vive en el entorno, junto al puerto y la base de datos, porque depende de la
-- máquina donde corre esto y no de la partida.
--
-- **El del jugador empieza en uno** y sube pagando. Ahí está el cambio de verdad: hasta ahora
-- comprar una máquina era comprar potencia, y a partir de aquí lo que se compra es **la
-- plaza**. Qué metes dentro es decisión tuya, y si la vacías la plaza sigue siendo tuya: dar
-- de baja un equipo ya no te devuelve el dinero, pero tampoco te quita el sitio.
--
-- Eso convierte una compra en una decisión con consecuencias en vez de un trámite. Un jugador
-- con tres plazas y un equipo bueno no es lo mismo que uno con tres equipos mediocres, y ahora
-- el juego sabe distinguirlos.
--
-- ## Nadie pierde nada al entrar esto
--
-- Cada jugador arranca con tantas plazas como equipos tenga hoy. Quien tenga cuatro sigue con
-- cuatro; quien tenga uno, con uno, que es también con lo que nace el que se registre mañana.

ALTER TABLE users
    ADD COLUMN machine_slots INTEGER NOT NULL DEFAULT 1
        CONSTRAINT users_machine_slots_positivo CHECK (machine_slots >= 1);

-- Las que ya se tienen. `GREATEST` porque un jugador sin ninguna máquina —no debería haberlo,
-- pero la columna no es quien para suponerlo— se queda con su plaza de salida.
UPDATE users u
   SET machine_slots = GREATEST(
        (SELECT count(*) FROM hardware h WHERE h.user_id = u.id AND NOT h.is_npc),
        1
   );

--
-- La comprobación: nadie se queda con menos plazas que equipos, que es la única forma de que
-- esto rompiera algo el día que entra.
--
DO $$
DECLARE
    apretados bigint;
BEGIN
    SELECT count(*) INTO apretados
      FROM users u
     WHERE (SELECT count(*) FROM hardware h WHERE h.user_id = u.id AND NOT h.is_npc)
           > u.machine_slots;

    IF apretados > 0 THEN
        RAISE EXCEPTION
            '% jugadores tendrian mas equipos que plazas', apretados;
    END IF;
END
$$;
