-- Evolución aditiva del software modular. Las columnas antiguas permanecen durante la
-- transición: `effectiveness`, `stealth`, `detection`, `removal` y `penetration` siguen
-- describiendo las copias existentes, mientras que los nuevos pares permiten resolver
-- cada fase de forma independiente.

ALTER TABLE software_profiles
  ADD COLUMN resistance INTEGER NOT NULL DEFAULT 0 CHECK(resistance BETWEEN 0 AND 100),
  ADD COLUMN hardening INTEGER NOT NULL DEFAULT 0 CHECK(hardening BETWEEN 0 AND 100),
  ADD COLUMN manipulation INTEGER NOT NULL DEFAULT 0 CHECK(manipulation BETWEEN 0 AND 100),
  ADD COLUMN integrity INTEGER NOT NULL DEFAULT 0 CHECK(integrity BETWEEN 0 AND 100),
  ADD COLUMN anti_forensics INTEGER NOT NULL DEFAULT 0 CHECK(anti_forensics BETWEEN 0 AND 100),
  ADD COLUMN forensics INTEGER NOT NULL DEFAULT 0 CHECK(forensics BETWEEN 0 AND 100),
  ADD COLUMN persistence INTEGER NOT NULL DEFAULT 0 CHECK(persistence BETWEEN 0 AND 100);

ALTER TABLE research_modules
  ADD COLUMN resistance_delta INTEGER NOT NULL DEFAULT 0,
  ADD COLUMN hardening_delta INTEGER NOT NULL DEFAULT 0,
  ADD COLUMN manipulation_delta INTEGER NOT NULL DEFAULT 0,
  ADD COLUMN integrity_delta INTEGER NOT NULL DEFAULT 0,
  ADD COLUMN anti_forensics_delta INTEGER NOT NULL DEFAULT 0,
  ADD COLUMN forensics_delta INTEGER NOT NULL DEFAULT 0,
  ADD COLUMN persistence_delta INTEGER NOT NULL DEFAULT 0;

-- La correspondencia es datos del dominio, no una convención perdida en una fórmula.
CREATE TABLE combat_attribute_pairs (
  attacker_attribute VARCHAR(32) PRIMARY KEY,
  defender_attribute VARCHAR(32) NOT NULL UNIQUE,
  phase VARCHAR(32) NOT NULL,
  CHECK (attacker_attribute IN ('effectiveness','penetration','stealth','manipulation','anti_forensics','persistence')),
  CHECK (defender_attribute IN ('resistance','hardening','detection','integrity','forensics','removal'))
);

INSERT INTO combat_attribute_pairs(attacker_attribute, defender_attribute, phase) VALUES
  ('effectiveness', 'resistance', 'execution'),
  ('penetration', 'hardening', 'penetration'),
  ('stealth', 'detection', 'detection'),
  ('manipulation', 'integrity', 'manipulation'),
  ('anti_forensics', 'forensics', 'anti_forensics'),
  ('persistence', 'removal', 'persistence');

ALTER TABLE software_capabilities
  ADD COLUMN relevant_attributes JSONB NOT NULL DEFAULT '[]'::jsonb,
  ADD COLUMN secondary_attributes JSONB NOT NULL DEFAULT '[]'::jsonb,
  ADD COLUMN available_actions JSONB NOT NULL DEFAULT '[]'::jsonb,
  ADD COLUMN max_access_scope VARCHAR(24) NOT NULL DEFAULT 'service',
  ADD COLUMN granted_permissions JSONB NOT NULL DEFAULT '[]'::jsonb;

-- Catálogo declarativo inicial. Añadir una capacidad cambia datos, no el selector.
UPDATE software_capabilities SET
  relevant_attributes='["effectiveness","penetration","stealth"]'::jsonb,
  secondary_attributes='["stability"]'::jsonb,
  available_actions='["intrude","escalate"]'::jsonb,
  max_access_scope='service',
  granted_permissions='["read_files","execute_programs"]'::jsonb
WHERE code IN ('ssh_exploit','ftp_exploit','cracker');

UPDATE software_capabilities SET
  relevant_attributes='["effectiveness","penetration","persistence"]'::jsonb,
  secondary_attributes='["stealth","stability"]'::jsonb,
  available_actions='["infect","install_payload","install_persistence"]'::jsonb,
  max_access_scope='user',
  granted_permissions='["read_files","execute_programs","install_payload","install_persistence"]'::jsonb
WHERE code='virus_infect';

UPDATE software_capabilities SET
  relevant_attributes='["effectiveness","resistance","hardening"]'::jsonb,
  secondary_attributes='["detection"]'::jsonb,
  available_actions='["defend"]'::jsonb
WHERE code IN ('firewall_ssh','firewall_ftp','anti_bruteforce');

UPDATE software_capabilities SET
  relevant_attributes='["detection","forensics"]'::jsonb,
  secondary_attributes='["resistance"]'::jsonb,
  available_actions='["detect","investigate"]'::jsonb
WHERE code IN ('intrusion_ids','antivirus_detect');

UPDATE software_capabilities SET
  relevant_attributes='["removal","detection"]'::jsonb,
  secondary_attributes='["forensics"]'::jsonb,
  available_actions='["remove","revoke"]'::jsonb
WHERE code='antivirus_remove';

-- Conserva el valor de las copias existentes, interpretándolo según su rol/capacidad.
UPDATE software_profiles p SET
  resistance = CASE WHEN c.role='defender' THEN p.effectiveness ELSE 0 END,
  hardening = CASE WHEN c.code IN ('firewall_ssh','firewall_ftp','anti_bruteforce') THEN GREATEST(20,p.penetration) ELSE 0 END,
  manipulation = CASE WHEN c.role='attacker' THEN LEAST(100, p.effectiveness / 2) ELSE 0 END,
  integrity = CASE WHEN c.role='defender' THEN LEAST(100, p.effectiveness / 2) ELSE 0 END,
  anti_forensics = CASE WHEN c.role='attacker' THEN LEAST(100, p.stealth / 2) ELSE 0 END,
  forensics = CASE WHEN c.code IN ('intrusion_ids','antivirus_detect','antivirus_remove') THEN LEAST(100, p.detection + p.removal / 2) ELSE 0 END,
  persistence = CASE WHEN c.code='virus_infect' THEN LEAST(100, p.stability) ELSE 0 END
FROM software_capabilities c
WHERE c.id=p.capability_id;

-- Acceso de juego con alcance, estado y permisos. `internet_access` sigue alimentando la
-- navegación heredada y se sincroniza cuando una intrusión obtiene acceso.
CREATE TABLE machine_accesses (
  id BIGINT GENERATED BY DEFAULT AS IDENTITY PRIMARY KEY,
  attacker_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  -- El objetivo puede ser jugador o NPC; `target_is_npc` decide el dominio dueño.
  target_user_id BIGINT NOT NULL,
  target_is_npc BOOLEAN NOT NULL DEFAULT FALSE,
  target_ip BIGINT NOT NULL,
  identity_name VARCHAR(32) NOT NULL,
  origin VARCHAR(24) NOT NULL,
  privilege VARCHAR(24) NOT NULL DEFAULT 'service',
  scope VARCHAR(24) NOT NULL DEFAULT 'service',
  permissions JSONB NOT NULL DEFAULT '[]'::jsonb,
  status VARCHAR(16) NOT NULL DEFAULT 'active' CHECK(status IN ('active','revoked','removed','expired')),
  obtained_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  expires_at TIMESTAMPTZ,
  persistence_installed BOOLEAN NOT NULL DEFAULT FALSE,
  UNIQUE(attacker_id, target_ip)
);
CREATE INDEX machine_accesses_target_idx ON machine_accesses(target_user_id, target_is_npc, status);

-- Evidencia de juego separada del log técnico. Nunca almacena contraseñas reales.
CREATE TABLE game_evidence (
  id BIGINT GENERATED BY DEFAULT AS IDENTITY PRIMARY KEY,
  process_id BIGINT NOT NULL REFERENCES processes(pid) ON DELETE CASCADE,
  target_user_id BIGINT NOT NULL,
  target_is_npc BOOLEAN NOT NULL DEFAULT FALSE,
  action_code VARCHAR(48) NOT NULL,
  real_origin_ip BIGINT,
  apparent_origin_ip BIGINT,
  real_identity VARCHAR(32),
  attributed_identity VARCHAR(32),
  real_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  apparent_at TIMESTAMPTZ,
  program_signature VARCHAR(80),
  integrity INTEGER NOT NULL DEFAULT 100 CHECK(integrity BETWEEN 0 AND 100),
  concealment INTEGER NOT NULL DEFAULT 0 CHECK(concealment BETWEEN 0 AND 100),
  state VARCHAR(24) NOT NULL DEFAULT 'normal' CHECK(state IN ('normal','hidden','degraded','altered','recoverable_deleted','recovered')),
  attribution_confidence INTEGER NOT NULL DEFAULT 100 CHECK(attribution_confidence BETWEEN 0 AND 100),
  visible_to_target BOOLEAN NOT NULL DEFAULT FALSE,
  details JSONB NOT NULL DEFAULT '{}'::jsonb
);
CREATE INDEX game_evidence_target_idx ON game_evidence(target_user_id, target_is_npc, real_at DESC);
CREATE UNIQUE INDEX game_evidence_process_action_idx ON game_evidence(process_id, action_code);

-- Resultado congelado por fase. Da idempotencia al worker y permite que Progresos muestre
-- más que un único código final.
CREATE TABLE operation_phase_results (
  process_id BIGINT NOT NULL REFERENCES processes(pid) ON DELETE CASCADE,
  phase VARCHAR(32) NOT NULL,
  probability INTEGER NOT NULL CHECK(probability BETWEEN 0 AND 100),
  roll INTEGER NOT NULL CHECK(roll BETWEEN 1 AND 100),
  outcome VARCHAR(32) NOT NULL,
  details JSONB NOT NULL DEFAULT '{}'::jsonb,
  resolved_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY(process_id, phase)
);

CREATE TABLE machine_web_content (
  target_user_id BIGINT NOT NULL,
  target_is_npc BOOLEAN NOT NULL DEFAULT FALSE,
  path VARCHAR(128) NOT NULL DEFAULT '/index.html',
  content TEXT NOT NULL DEFAULT '',
  integrity_hash VARCHAR(64),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY(target_user_id, target_is_npc, path)
);

-- Los accesos heredados conservan su sesión de juego sin repetir ni revelar contraseñas.
INSERT INTO machine_accesses(attacker_id,target_user_id,target_is_npc,target_ip,identity_name,origin,privilege,scope,permissions)
SELECT ia.user_id, COALESCE(u.id,n.id), (n.id IS NOT NULL), ia.ip,
       COALESCE(cl.login,'service'), CASE WHEN ia.via_exploit THEN 'legacy_exploit' ELSE 'legacy_login' END,
       CASE WHEN ia.via_exploit THEN 'service' ELSE 'root' END,
       CASE WHEN ia.via_exploit THEN 'service' ELSE 'root' END,
       CASE WHEN ia.via_exploit THEN '["read_files","execute_programs"]'::jsonb ELSE '["read_files","write_files","execute_programs","manage_services","read_logs","modify_logs","delete_logs","forge_logs","install_payload","install_persistence","escalate_privileges","access_database","pivot_network"]'::jsonb END
FROM internet_access ia
LEFT JOIN users u ON u.game_ip=ia.ip
LEFT JOIN npc n ON n.npc_ip=ia.ip
LEFT JOIN LATERAL (SELECT login FROM compromised_lists cl WHERE cl.user_id=ia.user_id AND cl.ip=ia.ip ORDER BY hacked_at DESC LIMIT 1) cl ON true
WHERE COALESCE(u.id,n.id) IS NOT NULL
ON CONFLICT(attacker_id,target_ip) DO NOTHING;
