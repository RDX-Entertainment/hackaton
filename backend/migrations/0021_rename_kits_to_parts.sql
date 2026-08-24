-- "Kit" estaba mal elegido.
--
-- Un kit es un equipo entero: compras el kit y te llevas el PC montado. Lo que esta tabla
-- guarda no es eso, sino la ficha de UNA pieza suelta -- una memoria, un disco, una placa --
-- que es lo que la tienda vende de una en una.
--
-- Asi que la plantilla pasa a llamarse `parte`, y `componente` sigue siendo la instancia
-- concreta que alguien posee. Compras una parte del catalogo y pasas a tener un componente
-- de esa parte. La palabra "kit" queda libre para lo que de verdad significa, si algun dia
-- se venden equipos montados.
ALTER TABLE hardware_kits RENAME TO hardware_parts;

ALTER TABLE hardware_components RENAME COLUMN kit_id TO part_id;

ALTER INDEX hardware_kits_pkey RENAME TO hardware_parts_pkey;
ALTER INDEX hardware_kits_kind_idx RENAME TO hardware_parts_kind_idx;
