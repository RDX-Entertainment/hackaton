-- Que un jugador tambien tenga pagina.
--
-- Hasta aqui solo servian algo los NPC, con `npc_info.web`. Navegar a la maquina de otro
-- jugador daba una direccion muda, y eso deja fuera la mitad de lo que hace interesante
-- un internet: que lo que hay al otro lado lo haya escrito alguien.
--
-- La columna va en `player_profiles`, que es donde ya vive lo que un jugador escribe
-- sobre si mismo. La biografia y la pagina son la misma clase de dato -- texto suyo, que
-- ve cualquiera -- y separarlas en dos tablas no diria nada nuevo.
--
-- Lo que se puede escribir dentro lo decide `internet::sanitize` al guardar, no la base:
-- la regla necesita leer el texto, y un CHECK no es sitio para eso.

ALTER TABLE player_profiles
  ADD COLUMN web TEXT NOT NULL DEFAULT '';

-- Una pagina de partida para quien ya existe. Un textarea vacio no invita a nada; algo
-- puesto que se pueda romper y arreglar, si.
UPDATE player_profiles p
SET web = replace(
  '<style>
    body{margin:0;background:#0d0f0e;color:#d8e6de;font:14px/1.7 monospace}
    .box{max-width:34rem;margin:3rem auto;padding:0 1.5rem}
    h1{margin:0;font-size:22px;color:#7ee2b0;letter-spacing:2px}
    .rule{height:1px;margin:14px 0 20px;background:#1e2b25}
    p{margin:.7rem 0;color:#93a89e}
    .tag{display:inline-block;margin-top:18px;padding:3px 8px;border:1px solid #2a3b33;color:#5f7a6d;font-size:11px}
  </style>
  <div class="box">
    <h1>{{login}}</h1>
    <div class="rule"></div>
    <p>Todavia no he escrito nada aqui.</p>
    <p>Esta pagina es mia y la sirve mi maquina. Puedo cambiarla entera desde mi perfil.</p>
    <span class="tag">sin firmar</span>
  </div>',
  '{{login}}',
  u.login
)
FROM users u
WHERE u.id = p.user_id
  AND p.web = '';
