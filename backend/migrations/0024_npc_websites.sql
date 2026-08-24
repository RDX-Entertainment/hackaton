-- Internet: que cada maquina tenga una pagina.
--
-- `npc_info.web` existe desde 0007 y lleva vacio desde entonces. Es lo que hace que
-- navegar sea navegar y no consultar una ficha: llegas a una direccion y hay algo puesto
-- ahi, escrito por quien la tenga, no una tabla generada por el cliente.
--
-- El contenido es HTML con su CSS dentro. El cliente lo pinta en un marco aislado y sin
-- permisos, asi que no puede ejecutar nada ni tocar la aplicacion que lo rodea. Eso es
-- justo lo que permite que cada sitio traiga su propio estilo sin pisar el del juego.
--
-- Tres plantillas, una por gama, que es la unica division que ya tenian los NPC: las
-- Corp son escaparates de empresa, los Labs presumen de investigacion y los Systems ni
-- disimulan que no quieren visitas.

UPDATE npc_info
SET web = replace(
  CASE
    WHEN name LIKE '% Labs' THEN
      '<style>
        body{margin:0;background:#12100c;color:#e8dfc8;font:14px/1.6 Georgia,serif}
        header{padding:28px 32px;border-bottom:3px double #7a5c1f}
        h1{margin:0;font-size:26px;letter-spacing:.5px;color:#e8c46a}
        .sub{margin:6px 0 0;color:#8c7a55;font-style:italic}
        main{padding:24px 32px;max-width:44rem}
        .notice{margin:0 0 22px;padding:12px 16px;background:#231d10;border-left:4px solid #e8c46a;font-size:13px}
        table{width:100%;border-collapse:collapse;font-size:13px}
        th{text-align:left;padding:6px 8px;border-bottom:2px solid #3a3120;color:#8c7a55;font-variant:small-caps}
        td{padding:6px 8px;border-bottom:1px solid #241f14}
        .ref{color:#e8c46a;font-family:monospace}
        footer{padding:16px 32px;color:#5c5240;font-size:11px;border-top:1px solid #241f14}
      </style>
      <header>
        <h1>{{name}}</h1>
        <p class="sub">Investigacion aplicada desde 1987</p>
      </header>
      <main>
        <p class="notice">Acceso al repositorio restringido al personal acreditado. Las
        credenciales caducan cada 90 dias.</p>
        <table>
          <tr><th>Referencia</th><th>Linea de trabajo</th><th>Estado</th></tr>
          <tr><td class="ref">AX-114</td><td>Sintesis de materiales</td><td>en curso</td></tr>
          <tr><td class="ref">AX-207</td><td>Criptografia aplicada</td><td>revision</td></tr>
          <tr><td class="ref">BX-002</td><td>Sin descripcion publica</td><td>clasificado</td></tr>
        </table>
      </main>
      <footer>{{name}} &mdash; documento generado automaticamente</footer>'

    WHEN name LIKE '% Systems' THEN
      '<style>
        body{margin:0;background:#050505;color:#c9c9c9;font:13px/1.7 monospace}
        .wrap{padding:36px 32px;max-width:40rem}
        h1{margin:0 0 4px;font-size:20px;color:#ff4b3e;letter-spacing:3px;text-transform:uppercase}
        .rule{height:1px;margin:18px 0;background:#2a2a2a}
        .warn{padding:14px 16px;border:1px solid #ff4b3e;color:#ff8a80}
        ul{margin:18px 0 0;padding:0;list-style:none}
        li{display:flex;justify-content:space-between;padding:5px 0;border-bottom:1px solid #171717}
        .ok{color:#4caf50}
        .no{color:#ff4b3e}
        .foot{margin-top:26px;color:#4a4a4a;font-size:11px}
      </style>
      <div class="wrap">
        <h1>{{name}}</h1>
        <div class="rule"></div>
        <div class="warn">SISTEMA RESTRINGIDO. Toda conexion queda registrada con su
        direccion de origen. El acceso no autorizado se persigue.</div>
        <ul>
          <li><span>gateway</span><span class="ok">activo</span></li>
          <li><span>almacenamiento</span><span class="ok">activo</span></li>
          <li><span>acceso publico</span><span class="no">deshabilitado</span></li>
          <li><span>correo</span><span class="no">deshabilitado</span></li>
        </ul>
        <p class="foot">Contacto: no procede.</p>
      </div>'

    ELSE
      '<style>
        body{margin:0;background:#f2f4f7;color:#1f2933;font:14px/1.6 Verdana,Geneva,sans-serif}
        header{padding:26px 32px;background:#1f3350;color:#fff}
        h1{margin:0;font-size:24px}
        .tag{margin:6px 0 0;color:#9fb4d4;font-size:13px}
        .bar{padding:9px 32px;background:#2c4468;color:#c8d8ef;font-size:12px;letter-spacing:1px}
        main{padding:26px 32px;max-width:46rem}
        .cards{display:flex;gap:14px;flex-wrap:wrap;margin-top:18px}
        .card{flex:1 1 12rem;padding:14px 16px;background:#fff;border:1px solid #d7dee8}
        .card h3{margin:0 0 6px;font-size:14px;color:#1f3350}
        .card p{margin:0;font-size:12px;color:#5a6b80}
        footer{padding:16px 32px;background:#e3e8ef;color:#7a8798;font-size:11px}
      </style>
      <header>
        <h1>{{name}}</h1>
        <p class="tag">Soluciones empresariales integradas</p>
      </header>
      <div class="bar">INICIO &nbsp;|&nbsp; PRODUCTOS &nbsp;|&nbsp; INVERSORES &nbsp;|&nbsp; CONTACTO</div>
      <main>
        <p>Somos referencia en nuestro sector desde hace tres decadas. Nuestra
        infraestructura atiende a clientes en cuarenta paises.</p>
        <div class="cards">
          <div class="card"><h3>Consultoria</h3><p>Acompanamos su transformacion de principio a fin.</p></div>
          <div class="card"><h3>Infraestructura</h3><p>Centros de datos propios con disponibilidad continua.</p></div>
          <div class="card"><h3>Soporte</h3><p>Atencion 24/7 para clientes con contrato vigente.</p></div>
        </div>
      </main>
      <footer>&copy; {{name}}. Todos los derechos reservados.</footer>'
  END,
  '{{name}}',
  name
)
WHERE web = '';
