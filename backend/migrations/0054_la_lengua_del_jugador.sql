-- La lengua del jugador deja de ser una columna que nadie mira.
--
-- `users_language` existe desde la 0001. El alta la rellena, y ahi se acaba la historia: en
-- todo el backend no hay un solo `SELECT` que la lea, y en el frontend no hay ni un catalogo
-- de textos. Es un enganche puesto el primer dia y nunca usado, como las certificaciones de
-- nivel 1 y 2.
--
-- ## Por que `'es'` y no `'en'`
--
-- El valor por defecto era `'en'` por herencia del legado, que era un juego en ingles. Este
-- no lo es: el chrome, el terminal y todas las pantallas reescritas estan en castellano, y
-- las que siguen en ingles lo estan por no haber pasado por ahi todavia, no por decision.
-- Poner `'en'` por defecto habria significado que el juego se enseña en el idioma en el que
-- **menos** terminado esta.
--
-- ## Y por que se puede reescribir lo que ya hay
--
-- Porque nadie eligio nada. La columna no tenia ni forma de cambiarse: no habia pantalla, ni
-- endpoint, ni consulta. Todo `'en'` que hay guardado es el valor por defecto de una decision
-- que nunca se tomo, y reescribirlo no le quita la eleccion a nadie. El dia que exista el
-- selector —a partir de ahora— esto ya no se podria hacer, y con razon.

ALTER TABLE users_language
    ALTER COLUMN lang SET DEFAULT 'es';

UPDATE users_language SET lang = 'es' WHERE lang = 'en';

-- Tres, y solo tres. Que la base lo garantice evita que un dia entre un 'fr' por una ruta que
-- no valido, y que el cliente se quede sin catalogo que buscar.
ALTER TABLE users_language
    ADD CONSTRAINT users_language_conocido
        CHECK (lang IN ('es', 'pt', 'en'));

COMMENT ON COLUMN users_language.lang IS
    'En que lengua lee el jugador. El servidor manda codigos de error, no frases; la frase la escribe el cliente en esta.';
