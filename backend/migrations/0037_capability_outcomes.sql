-- Que consigue una intrusion cuando sale bien.
--
-- Hasta ahora una operacion modular terminaba escribiendo un `status` y un mensaje, y ahi
-- se acababa: no abria la maquina, no dejaba rastro y no pagaba experiencia. Para darle
-- consecuencia hace falta un dato que el modelo no tenia, porque no todas las intrusiones
-- consiguen lo mismo:
--
--   * un `cracker` saca la contrasena de verdad, y con ella se entra como `root`. Es lo que
--     hacia el caso 11 del legacy.
--   * un exploit no saca ninguna contrasena: se cuela por un servicio y deja al atacante
--     dentro con el usuario de ese servicio, `ssh` o `ftp`. Es el caso 16.
--
-- La diferencia se guarda aqui y no en Rust, que es la regla del modulo: anadir una
-- capacidad nueva es una fila, no una rama en el worker.
--
--   `grants_login` NULL  la operacion revela la credencial real y entra como root
--   `grants_login` 'ssh' entra como ese usuario, sin conocer la contrasena
ALTER TABLE software_capabilities ADD COLUMN grants_login VARCHAR(15);

UPDATE software_capabilities SET grants_login = 'ssh' WHERE code = 'ssh_exploit';
UPDATE software_capabilities SET grants_login = 'ftp' WHERE code = 'ftp_exploit';

-- El cracker se queda en NULL a proposito: es el unico que se lleva la contrasena.

-- Solo las capacidades que entran en una maquina conceden un usuario. Que una defensora
-- o un virus tuvieran uno seria una fila sin sentido que alguien acabaria leyendo.
ALTER TABLE software_capabilities ADD CONSTRAINT software_capabilities_login_only_intrudes
  CHECK (grants_login IS NULL OR action_kind = 'intrude');
