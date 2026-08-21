ALTER TABLE processes ADD COLUMN completed_at TIMESTAMPTZ;
CREATE INDEX processes_active_idx ON processes(creator_id, time_end) WHERE completed_at IS NULL AND NOT is_paused;
