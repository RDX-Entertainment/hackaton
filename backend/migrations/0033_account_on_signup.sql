-- Un jugador nuevo nace con cuenta en el banco, y sin PIN todavia.
--
-- Hasta ahora el alta daba equipo, log, perfil y ranking, pero no cuenta: para tener donde
-- guardar dinero habia que ir a Finanzas y abrirla a mano. El equipo sale del kit por
-- defecto dentro de la transaccion del alta, y la cuenta pasa a salir igual: o se crea todo
-- o no se crea nada.
--
-- El PIN no se puede inventar por el jugador. Si el servidor sorteara uno, tendria que
-- ensenarselo en el alta y guardarlo en algun sitio hasta que lo leyera; y un PIN que el
-- jugador no eligio es un PIN que no recuerda. Asi que la cuenta nace **sin PIN** y se lo
-- pide la primera vez que entra en Finanzas.
--
-- Sin PIN se guarda como NULL, y no hace falta una columna aparte para saberlo. Ademas sale
-- gratis una garantia: la transferencia compara la clave recibida con `bank_pass`, y en SQL
-- nada es igual a NULL, ni siquiera NULL. Una cuenta sin PIN no puede mover dinero aunque
-- alguien acierte que esta vacia.

ALTER TABLE bank_accounts
    ALTER COLUMN bank_pass DROP NOT NULL;

-- El numero de cuenta se sorteaba en Rust con cinco reintentos, porque puede chocar con uno
-- ya dado. Desde el alta se necesita el mismo sorteo dentro de una sola sentencia SQL, asi
-- que la regla baja aqui y queda en un solo sitio, igual que `random_free_ip()` para las
-- direcciones.
CREATE OR REPLACE FUNCTION random_free_bank_account() RETURNS BIGINT AS $$
DECLARE
  candidate BIGINT;
BEGIN
  LOOP
    candidate := floor(random() * (999999999 - 111111111 + 1))::BIGINT + 111111111;

    EXIT WHEN NOT EXISTS (
      SELECT 1 FROM bank_accounts WHERE bank_acc = candidate
    );
  END LOOP;

  RETURN candidate;
END;
$$ LANGUAGE plpgsql VOLATILE;

-- Cual es «el banco» cuando hay varios.
--
-- Los kits resuelven esto con `is_default` y un indice parcial unico. Aqui el banco es una
-- organizacion y su condicion de banco vive en el `config` del comportamiento, asi que la
-- marca va dentro: `{"default": true}`. No se fuerza que sea unica —son datos libres— y por
-- eso el desempate esta escrito: primero el marcado, y si hay varios o ninguno, el primero
-- por nombre. Una regla aburrida y siempre la misma es mejor que una fila al azar.
CREATE OR REPLACE FUNCTION default_bank() RETURNS BIGINT AS $$
  SELECT b.organization_id
  FROM organization_behaviors b
  JOIN organizations o ON o.id = b.organization_id
  WHERE b.code = 'banking'
    AND o.active
  ORDER BY (b.config->>'default' = 'true') DESC, o.name, o.id
  LIMIT 1;
$$ LANGUAGE sql STABLE;
