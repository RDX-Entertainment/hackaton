-- El catálogo original solo cubría cuatro capacidades de ataque. Estos módulos
-- de ejecución son universales (capability_id NULL), sin cambiar los existentes
-- ni los perfiles de programas ya compilados. Se recaban con el reparto temporal.
INSERT INTO research_modules (
    capability_id, code, name, stability_delta, cpu_delta, ram_delta, size_delta
)
VALUES
    (NULL, 'runtime_stability', 'Núcleo de estabilidad', 6, 2, 2, 2),
    (NULL, 'memory_optimizer', 'Optimizador de memoria', 0, 2, -8, 2),
    (NULL, 'execution_optimizer', 'Optimizador de ejecución', 0, -10, 4, 2)
ON CONFLICT (code) DO NOTHING;
