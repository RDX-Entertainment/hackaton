-- El kit «Servidor completo» pasa de `marketplace` a `shop`.
--
-- La 0043 lo creó con `scope = 'marketplace'`, y eso lo dejaba invisible: la
-- consulta que alimenta la tienda —`kits::repo::find_offered`— excluye ese ámbito
-- a propósito, porque está reservado para el mercado de equipos de NPC, que
-- todavía no existe. El kit aparecía solo en el panel de administración, que es
-- el único sitio que los lista todos.
--
-- No se corrige editando la 0043: sqlx guarda el hash de cada migración aplicada
-- y cambiarla dejaría la base sin poder arrancar. Se corrige encima, que además
-- deja escrito el porqué.
--
-- La máquina que la 0043 montó para la cuenta admin no se toca: ya está entregada
-- y esto solo cambia dónde se ofrece el kit.

UPDATE hardware_kits
SET scope = 'shop'
WHERE name = 'Servidor completo'
  AND scope = 'marketplace';
