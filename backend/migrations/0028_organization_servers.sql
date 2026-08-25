-- Servidores creados desde una infraestructura.
--
-- Hardware, software y logs ya distinguen al propietario por (user_id, is_npc). Los
-- componentes eran la unica excepcion y por eso un kit solo podia montar equipos de
-- jugadores. Se les aplica la misma identidad para que un NPC pueda poseer las piezas
-- reales de su servidor.

ALTER TABLE hardware_components DROP CONSTRAINT hardware_components_user_id_fkey;
ALTER TABLE hardware_components ADD COLUMN is_npc BOOLEAN NOT NULL DEFAULT FALSE;
CREATE INDEX hardware_components_npc_owner_idx ON hardware_components(user_id, is_npc);

-- Un servidor creado desde el panel recuerda el kit que lo origino. Las piezas instaladas
-- son la verdad actual; cambiar el kit despues nunca cambia equipos que ya existen.
ALTER TABLE organization_network_npcs
  ADD COLUMN source_kit_id BIGINT REFERENCES hardware_kits(id) ON DELETE RESTRICT;
