-- El mundo tiene bancos, y quien nacio sin cuenta la recibe.
--
-- El alta abre una cuenta desde la 0033, pero solo `WHERE default_bank() IS NOT NULL`. Y
-- `default_bank()` busca una organizacion activa con el comportamiento `banking`, de las que
-- **no habia ninguna**: ninguna migracion ha insertado nunca una organizacion. Asi que el
-- INSERT metia cero filas, en silencio, y el alta seguia tan contenta. El jugador se enteraba
-- al entrar en Finanzas y leer que le pidiera la cuenta a un administrador.
--
-- Era una eleccion deliberada y sigue siendo la correcta —un juego al que todavia no se le ha
-- creado la banca no puede impedir que alguien se registre— pero le faltaba la otra mitad:
-- **alguien tenia que crear la banca**.
--
-- ## Tres, y no uno
--
-- Con un solo banco la eleccion no existe: abrir cuenta seria un tramite con una sola opcion
-- y `default_bank()` no tendria que desempatar nada. Con tres hay decision, la pantalla de
-- Finanzas tiene algo que enseñar, y el dia que se pueda robar a un banco habra bancos
-- distintos a los que robar.
--
-- El marcado por defecto es el primero, y esa marca es la que ya sabe leer `default_bank()`:
-- `{"default": true}` en el `config` del comportamiento. No se inventa nada.
--
-- ## Y la reconciliacion, que es lo que arregla el pasado
--
-- Sembrar bancos deja bien a quien se registre a partir de ahora y no toca a quien ya estaba:
-- ese se quedaria sin cuenta para siempre, porque el alta solo ocurre una vez y no hay nada
-- que la repase. Por eso el final de esto abre cuenta a todo jugador que no tenga ninguna,
-- con las mismas reglas que usa el alta: numero libre sorteado y **sin PIN**, que lo elige el
-- jugador la primera vez que entra en Finanzas.

--
-- ## Y de paso, la marca de «por defecto» no funcionaba
--
-- `default_bank()` desempata con `ORDER BY (config->>'default' = 'true') DESC`. Para un banco
-- sin esa clave la expresion no vale `false`: vale **NULL**, porque `NULL = 'true'` es NULL. Y
-- en Postgres un `ORDER BY ... DESC` pone los NULL **primero**. O sea que el marcado perdia
-- contra cualquier otro que no dijera nada.
--
-- Con un solo banco en el mundo eso nunca se noto, porque no habia contra quien perder. Al
-- sembrar tres se nota a la primera: la cuenta se abria en «Caja Lenta».
--
CREATE OR REPLACE FUNCTION default_bank() RETURNS BIGINT AS $$
  SELECT b.organization_id
  FROM organization_behaviors b
  JOIN organizations o ON o.id = b.organization_id
  WHERE b.code = 'banking'
    AND o.active
  ORDER BY COALESCE(b.config->>'default' = 'true', FALSE) DESC, o.name, o.id
  LIMIT 1;
$$ LANGUAGE sql STABLE;

INSERT INTO organizations (slug, name, category, description) VALUES
    ('pinance',      'Pinance',              'bank', 'El banco de toda la vida. Nadie sabe de que epoca es su web y a nadie le preocupa.'),
    ('nubank-cerro', 'Nubank do Cerrado',    'bank', 'Solo movil, sin oficinas y con una tarjeta morada. Creceria mas rapido si sus servidores tambien.'),
    ('caja-lenta',   'Caja Lenta',           'bank', 'Comisiones altas, seguridad tambien. Cobra por todo y por eso puede pagarsela.')
ON CONFLICT (slug) DO NOTHING;

INSERT INTO organization_behaviors (organization_id, code, config)
SELECT o.id, 'banking',
       CASE WHEN o.slug = 'pinance' THEN '{"default": true}'::jsonb ELSE '{}'::jsonb END
  FROM organizations o
 WHERE o.slug IN ('pinance', 'nubank-cerro', 'caja-lenta')
ON CONFLICT (organization_id, code) DO NOTHING;

-- A quien se quedo sin ella. Un jugador, una cuenta: la condicion es no tener ninguna, no
-- tenerla en este banco.
INSERT INTO bank_accounts (bank_acc, bank_pass, bank_id, bank_user)
SELECT random_free_bank_account(), NULL, default_bank(), u.id
  FROM users u
 WHERE default_bank() IS NOT NULL
   AND NOT EXISTS (SELECT 1 FROM bank_accounts a WHERE a.bank_user = u.id);

--
-- La comprobacion: ni un jugador sin cuenta, y un banco por defecto que existe.
--
DO $$
DECLARE
    sin_cuenta bigint;
    porDefecto text;
BEGIN
    SELECT o.slug INTO porDefecto FROM organizations o WHERE o.id = default_bank();
    IF porDefecto IS DISTINCT FROM 'pinance' THEN
        RAISE EXCEPTION 'el banco por defecto deberia ser pinance y es %', COALESCE(porDefecto, 'ninguno');
    END IF;

    IF default_bank() IS NULL THEN
        RAISE EXCEPTION 'se han sembrado los bancos y default_bank() sigue sin devolver ninguno';
    END IF;

    SELECT count(*) INTO sin_cuenta
      FROM users u
     WHERE NOT EXISTS (SELECT 1 FROM bank_accounts a WHERE a.bank_user = u.id);

    IF sin_cuenta > 0 THEN
        RAISE EXCEPTION '% jugadores se quedan sin cuenta', sin_cuenta;
    END IF;
END
$$;
