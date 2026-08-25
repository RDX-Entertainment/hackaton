-- Un banco deja de ser un numero suelto y pasa a ser una organizacion.
--
-- Hasta ahora `bank_accounts.bank_id` era un bigint sin clave ajena ni tabla detras: la
-- unica regla era estar entre 1 y 9999. Se podia abrir una cuenta en el «banco 4711», que
-- no existia en ningun sitio, y la regla de una cuenta por banco se aplicaba contra un
-- numero inventado.
--
-- Ahora un banco es una organizacion con el comportamiento `banking`, y quien lo garantiza
-- es la base, no el servicio. El truco es la columna generada de abajo: vale siempre
-- 'banking', y la clave ajena compuesta apunta a (organization_id, code) de
-- organization_behaviors. De ahi salen las dos garantias que se quieren:
--
--   * no se puede abrir una cuenta en una organizacion que no sea un banco;
--   * no se le puede quitar el `banking` a una organizacion que ya tiene cuentas abiertas.
--
-- Ninguna de las dos depende de que alguien se acuerde de comprobarlas en Rust.
--
-- El titular tambien cambia. Antes una cuenta solo podia ser de un jugador, porque
-- `bank_user` era clave ajena a users. Ahora una organizacion puede tener la suya: el FBI
-- tiene cuenta en Pinance sin ser un banco. Se hace con dos columnas excluyentes en vez de
-- un dueno polimorfico para no perder las dos claves ajenas de verdad; el CHECK obliga a que
-- haya exactamente un titular, nunca dos ni ninguno.
--
-- La tabla esta vacia (cero cuentas, cero dinero), asi que esto no migra datos de nadie.
-- Es justo el momento de cambiar lo que significa `bank_id`: con cuentas abiertas ya no
-- saldria gratis.

ALTER TABLE bank_accounts
    ADD COLUMN owner_organization_id bigint
        REFERENCES organizations(id) ON DELETE RESTRICT;

ALTER TABLE bank_accounts
    ALTER COLUMN bank_user DROP NOT NULL;

-- Exactamente un titular: o un jugador, o una organizacion.
ALTER TABLE bank_accounts
    ADD CONSTRAINT bank_accounts_one_owner_check
        CHECK ((bank_user IS NULL) <> (owner_organization_id IS NULL));

-- El banco: una organizacion que ademas sea un banco.
ALTER TABLE bank_accounts
    ADD COLUMN bank_behavior varchar(60)
        GENERATED ALWAYS AS ('banking') STORED;

ALTER TABLE bank_accounts
    ADD CONSTRAINT bank_accounts_bank_is_a_bank_fkey
        FOREIGN KEY (bank_id, bank_behavior)
        REFERENCES organization_behaviors(organization_id, code)
        ON DELETE RESTRICT;

-- Una cuenta por banco y titular, que antes se comprobaba contando en el servicio.
CREATE UNIQUE INDEX bank_accounts_user_per_bank_idx
    ON bank_accounts(bank_id, bank_user)
    WHERE bank_user IS NOT NULL;

CREATE UNIQUE INDEX bank_accounts_organization_per_bank_idx
    ON bank_accounts(bank_id, owner_organization_id)
    WHERE owner_organization_id IS NOT NULL;

CREATE INDEX bank_accounts_bank_idx ON bank_accounts(bank_id);

-- El bitcoin pasa a entero, como el resto del dinero del juego.
--
-- Se guardaba en numeric(12,7) y el codigo ya lo trataba como entero de siete decimales:
-- multiplicaba por 10.000.000 al entrar y volvia a formatear al salir. El viaje completo era
-- numeric -> texto -> f64 -> entero -> texto -> numeric, y ese f64 en medio es la unica
-- pieza del recorrido capaz de redondear mal.
--
-- Guardado en satoshis, 1,0000001 BTC es 10000001 y la aritmetica es exacta. De paso el
-- techo sube: numeric(12,7) solo dejaba cinco digitos enteros —99.999 BTC— y un bigint de
-- satoshis llega a unos 922.337 millones.
--
-- Quien pone la coma es el cliente. La API habla de satoshis y nada mas.
ALTER TABLE bitcoin_wallets
    ALTER COLUMN amount TYPE bigint
        USING round(amount * 10000000)::bigint;

ALTER TABLE bitcoin_wallets
    ADD CONSTRAINT bitcoin_wallets_amount_not_negative_check
        CHECK (amount >= 0);

COMMENT ON COLUMN bitcoin_wallets.amount IS
    'Satoshis: siete decimales. 10000001 son 1,0000001 BTC.';
