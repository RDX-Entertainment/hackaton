-- Las zonas y las coordenadas son propiedades del equipo, no solo del segmento.
ALTER TABLE organization_network_npcs
  ADD COLUMN zone VARCHAR(16) NOT NULL DEFAULT 'internal',
  ADD COLUMN map_x INTEGER NOT NULL DEFAULT 0,
  ADD COLUMN map_y INTEGER NOT NULL DEFAULT 0,
  ADD CONSTRAINT organization_network_npcs_zone_check
    CHECK (zone IN ('public', 'dmz', 'internal', 'restricted'));

UPDATE organization_network_npcs AS membership
SET zone = network.zone
FROM organization_networks AS network
WHERE network.id = membership.network_id;
