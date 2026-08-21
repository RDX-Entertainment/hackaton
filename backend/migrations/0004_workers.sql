-- State used by the Rust replacement for cron/ and cron2/.
CREATE TABLE users_premium (
  user_id BIGINT PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
  premium_until TIMESTAMPTZ NOT NULL
);
CREATE TABLE system_stats (
  key VARCHAR(40) PRIMARY KEY,
  payload JSONB NOT NULL DEFAULT '{}'::jsonb,
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
