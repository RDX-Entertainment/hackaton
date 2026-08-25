-- La zona de un nodo tenia dos fuentes de verdad que no se hablaban.
--
-- `organization_network_npcs.zone` era una columna que se heredaba de la red al asignar el
-- nodo y que despues actualizaban `edit_node`, `connect_web` y `disconnect_web`. Pero la
-- consulta que alimenta el mapa nunca la leyo: derivaba la zona de si la maquina tenia una
-- pieza `net` instalada. Ganaba la derivada, y la columna se quedaba diciendo otra cosa.
--
-- El desacuerdo no era cosmetico. `internet` decidia con la columna a quien se le responde
-- una IP, asi que un nodo al que se le quitaba la interfaz de red seguia sirviendo su web
-- —la columna seguia en 'public'— y uno nacido en una red interna era invisible aunque
-- tuviera interfaz. Se corregia una y la otra seguia mintiendo.
--
-- Manda el hardware, que es lo que el propio panel promete: «un nodo pasa a Internet al
-- instalarle una interfaz de red desde Computer». La columna se va y la regla queda escrita
-- una sola vez, aqui.
--
-- Con ella se van 'dmz' y 'restricted' como zona de un nodo: la derivacion solo distingue si
-- hay interfaz o no. Las cuatro etiquetas siguen vivas en la red, que es donde se eligen a
-- mano.

CREATE VIEW npc_internet_exposure AS
SELECT
    n.id AS npc_id,

    -- Publico es tener por donde salir. Una pieza `net` instalada es la interfaz de Internet.
    iface.present AS is_public,

    -- Los NPCs antiguos no estan en ninguna organizacion y siguen respondiendo como siempre:
    -- son maquinas sueltas en Internet, no nodos de una red que alguien monto.
    (m.npc_id IS NULL OR (iface.present AND org.active)) AS reachable

FROM npc n

LEFT JOIN organization_network_npcs m
    ON m.npc_id = n.id
LEFT JOIN organization_networks net
    ON net.id = m.network_id
LEFT JOIN organizations org
    ON org.id = net.organization_id

CROSS JOIN LATERAL (
    SELECT EXISTS (
        SELECT 1
        FROM hardware_components c
        JOIN hardware h
            ON h.server_id = c.installed_in
        JOIN hardware_parts p
            ON p.id = c.part_id
        WHERE h.is_npc
          AND h.user_id = n.id
          AND p.kind = 'net'
    ) AS present
) AS iface;

ALTER TABLE organization_network_npcs
    DROP COLUMN zone;
