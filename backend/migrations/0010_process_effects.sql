-- The last piece of Internet.class.php state that lived in $_SESSION.
--
-- When a PORT_SCAN finished, completeProcess only set $_SESSION['PORT_SCAN'] = victimIP so
-- the next page render could show the open ports. Same reasoning as 0009: a stateless API
-- has no session to hang that on, so the scan leaves a row instead.
CREATE TABLE internet_scan (
  user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  ip BIGINT NOT NULL,
  scanned_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY (user_id, ip)
);
