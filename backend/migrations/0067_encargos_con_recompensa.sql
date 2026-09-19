-- An accepted mission becomes a real contract: one successful access to its target pays once.
ALTER TABLE missions
  ADD COLUMN completed_at timestamptz,
  ADD COLUMN reward_module_id bigint REFERENCES research_modules(id) ON DELETE SET NULL,
  ADD COLUMN reward_module_tier integer CHECK (reward_module_tier BETWEEN 0 AND 4);

CREATE INDEX missions_active_target_idx
  ON missions(user_id, victim)
  WHERE status = 2;

-- The temporary free batch endpoint has been removed; contracts are the source of modules.
DROP TABLE module_collection;
