-- Las copias heredadas tambien participan del sistema modular. No todas tienen una
-- capacidad que podamos afirmar sin inventarla; esas conservan el perfil y aparecen como
-- legado, mientras que los tipos cuya equivalencia es directa quedan utilizables.
INSERT INTO software_profiles(
  software_id, origin, capability_id, effectiveness, stealth, detection, removal,
  penetration, stability, cpu_need, ram_need, net_need, weight
)
SELECT
  s.id,
  'legacy',
  c.id,
  LEAST(100, GREATEST(20, 30 + s.soft_version)),
  CASE WHEN s.soft_type IN (13, 14, 99) THEN 35 ELSE 0 END,
  CASE WHEN s.soft_type = 4 THEN 45 ELSE 0 END,
  CASE WHEN s.soft_type = 7 THEN 35 ELSE 0 END,
  CASE WHEN s.soft_type IN (1, 13, 14, 99) THEN 35 ELSE 0 END,
  55,
  GREATEST(1, s.soft_ram),
  GREATEST(1, s.soft_ram),
  1,
  GREATEST(1, s.soft_size)
FROM software s
LEFT JOIN software_capabilities c ON c.code = CASE s.soft_type
  WHEN 1 THEN 'cracker'
  WHEN 4 THEN 'firewall_ssh'
  WHEN 7 THEN 'antivirus_remove'
  WHEN 13 THEN 'ftp_exploit'
  WHEN 14 THEN 'ssh_exploit'
  WHEN 99 THEN 'virus_infect'
  ELSE NULL
END
ON CONFLICT (software_id) DO NOTHING;

-- La accion 91 es interna al sistema modular: no reusa un tipo heredado ni materializa
-- una copia al pulsar el boton. Su descripcion, atributos y consumo quedan congelados en
-- process_payload para que cambiar un diseno no cambie una compilacion ya en curso.
CREATE INDEX processes_active_compilation_idx
  ON processes(creator_id)
  WHERE action = 91 AND completed_at IS NULL;
