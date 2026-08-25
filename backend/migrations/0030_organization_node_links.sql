CREATE TABLE organization_node_links (
  id BIGSERIAL PRIMARY KEY,
  organization_id BIGINT NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
  from_npc_id BIGINT NOT NULL REFERENCES npc(id) ON DELETE CASCADE,
  to_npc_id BIGINT NOT NULL REFERENCES npc(id) ON DELETE CASCADE,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  CONSTRAINT organization_node_links_distinct CHECK (from_npc_id <> to_npc_id),
  CONSTRAINT organization_node_links_ordered CHECK (from_npc_id < to_npc_id),
  UNIQUE (organization_id, from_npc_id, to_npc_id)
);
