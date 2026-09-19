-- Módulos universales de combate: modifican los doce atributos tácticos, no
-- únicamente los costes de ejecución. Se obtienen con «Recabar módulos».
-- No altera módulos previos ni perfiles de software ya compilado.
WITH catalog(code, name, deltas) AS (
    VALUES
        ('tactical_penetration', 'Motor de penetración', '{"penetration":12,"effectiveness":8}'::jsonb),
        ('tactical_resistance', 'Blindaje reforzado', '{"resistance":12,"hardening":8}'::jsonb),
        ('tactical_evasion', 'Capa de evasión', '{"stealth":12,"anti_forensics":8}'::jsonb),
        ('tactical_detection', 'Análisis de amenazas', '{"detection":12,"forensics":8}'::jsonb),
        ('tactical_persistence', 'Anclaje persistente', '{"persistence":12,"manipulation":8}'::jsonb),
        ('tactical_recovery', 'Recuperación íntegra', '{"removal":12,"integrity":8}'::jsonb)
)
INSERT INTO research_modules (
    capability_id, code, name,
    effectiveness_delta, penetration_delta, resistance_delta, hardening_delta,
    stealth_delta, anti_forensics_delta, detection_delta, forensics_delta,
    persistence_delta, manipulation_delta, removal_delta, integrity_delta,
    cpu_delta, ram_delta, size_delta
)
SELECT NULL, code, name,
    COALESCE((deltas->>'effectiveness')::integer, 0),
    COALESCE((deltas->>'penetration')::integer, 0),
    COALESCE((deltas->>'resistance')::integer, 0),
    COALESCE((deltas->>'hardening')::integer, 0),
    COALESCE((deltas->>'stealth')::integer, 0),
    COALESCE((deltas->>'anti_forensics')::integer, 0),
    COALESCE((deltas->>'detection')::integer, 0),
    COALESCE((deltas->>'forensics')::integer, 0),
    COALESCE((deltas->>'persistence')::integer, 0),
    COALESCE((deltas->>'manipulation')::integer, 0),
    COALESCE((deltas->>'removal')::integer, 0),
    COALESCE((deltas->>'integrity')::integer, 0),
    8, 6, 4
FROM catalog
ON CONFLICT (code) DO NOTHING;
