-- Where an analyzer leaves its reading.
--
-- Case 23 of completeProcess measured a victim's CPU and RAM and showed the answer as a
-- range, narrower the better the analyzer: a v10 reads within 50%, a v20 within 80%, a v30
-- exactly. The legacy handed that to the page it was rendering and kept nothing, which a
-- stateless API cannot do, so the reading is stored per player and target.
--
-- Both bounds are equal when the analyzer is perfect, which is how a v30 reads.
CREATE TABLE analyze_results (
  user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  ip BIGINT NOT NULL,
  cpu_min INTEGER NOT NULL,
  cpu_max INTEGER NOT NULL,
  ram_min INTEGER NOT NULL,
  ram_max INTEGER NOT NULL,
  analyzed_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY (user_id, ip)
);
