// Actual compiled UI, with every API request confined to the isolated test server.
import assert from 'node:assert/strict';
import { chromium } from '/tmp/hackaton-tutorial-check/node_modules/playwright/index.mjs';
const api='http://127.0.0.1:18091';
const base=process.argv[2] || 'http://127.0.0.1:5174';
const loginResponse=await fetch(`${api}/api/auth/login`,{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify({username:'lmmtxiy3mn',password:'HardwareTestOnly2026'})});
assert(loginResponse.ok,`Isolated login failed: ${loginResponse.status}`);
const login=await loginResponse.json();
assert(login.token);
const browser=await chromium.launch({headless:true,executablePath:'/home/raul/.cache/ms-playwright/chromium_headless_shell-1243/chrome-headless-shell-linux64/chrome-headless-shell',args:['--no-sandbox']});
const ip=866344883;
try {
  for(const width of [1440,390]) {
    const page=await browser.newPage({viewport:{width,height:1000},reducedMotion:'reduce'});
    const errors=[];page.on('pageerror',e=>errors.push(e.message));
    let rejectPublication = true;
    await page.addInitScript(s=>{ if(window===window.top) localStorage.setItem('legacy.session',JSON.stringify(s)); },login);
    await page.route('**/api/**',async route=>{
      try {
      const url=new URL(route.request().url());
      if(!url.pathname.startsWith('/api/')) return route.continue();
      if(url.pathname===`/api/software-system/accesses/${ip}/web` && route.request().method()==='PUT' && rejectPublication) {
        rejectPublication=false;
        return await route.fulfill({status:400,json:{error:'Publicación rechazada de prueba'}});
      }
      return await route.fulfill({response:await route.fetch({url:`${api}${url.pathname}${url.search}`})});
      } catch { if (!page.isClosed()) await route.abort().catch(()=>{}); }
    });
    await page.goto(`${base}/internet?ip=${ip}`, {waitUntil:'domcontentloaded',timeout:60000});
    await page.getByRole('heading',{name:'Elige tu punto de entrada'}).waitFor();
    assert.equal(await page.locator('.intrusion__service').count(),3);
    for(const service of ['Web','Almacenamiento','SSH']) assert(await page.getByRole('button',{name:`Abrir ${service}`}).isEnabled());
    await page.locator('.intrusion').screenshot({path:`/tmp/hackaton-tutorial-check/services-${width}.png`});
    if (!(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth))) console.log(await page.evaluate(()=>[...document.querySelectorAll('*')].filter(e=>e.getBoundingClientRect().right>innerWidth).slice(0,20).map(e=>({tag:e.tagName,cl:e.className,right:e.getBoundingClientRect().right}))));
    assert(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth));
    await page.getByRole('button',{name:'Abrir Web'}).click();
    const editor=page.getByLabel('HTML del sitio');
    await page.waitForFunction(()=>document.querySelector('.webedit textarea')?.value.includes('intr65 published'));
    const original = await editor.inputValue();
    await editor.fill('<h1>Vista previa de prueba</h1>');
    await page.getByRole('button',{name:'Actualizar vista previa'}).click();
    await page.frameLocator('.webedit iframe').getByRole('heading',{name:'Vista previa de prueba'}).waitFor();
    assert.equal(await page.locator('.webedit iframe').getAttribute('sandbox'),'');
    assert(await page.getByRole('button',{name:'Publicar modificación'}).isEnabled());
    await editor.fill('x'.repeat(16385));
    assert(!(await page.getByRole('button',{name:'Publicar modificación'}).isEnabled()));
    await editor.fill('<h1>Vista previa de prueba</h1>');
    await page.locator('.webedit').screenshot({path:`/tmp/hackaton-tutorial-check/web-editor-${width}.png`});
    assert(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth));
    await editor.fill('<p>hola</p>');
    await page.getByRole('button',{name:'Publicar modificación'}).click();
    await page.locator('.webedit [role="alert"]').waitFor();
    assert.equal(await editor.inputValue(),'<p>hola</p>','failed publication preserves the draft');
    try {
      await page.getByRole('button',{name:'Publicar modificación'}).click();
      await page.frameLocator('.site__frame').getByText('hola',{exact:true}).waitFor();
      assert(await page.locator('[data-guia="red.public"]').getAttribute('aria-current')==='true');
      await page.reload({waitUntil:'domcontentloaded',timeout:60000});
      await page.locator('[data-guia="red.public"]').click();
      await page.frameLocator('.site__frame').getByText('hola',{exact:true}).waitFor();
      await page.locator('[data-guia="red.webedit"]').click();
      await page.waitForFunction(()=>document.querySelector('.webedit textarea')?.value==='<p>hola</p>');
    } finally {
      // Node fetch goes directly to the guarded isolated API, never to the public host.
      const restored=await fetch(`${api}/api/software-system/accesses/${ip}/web`,{method:'PUT',headers:{'Content-Type':'application/json',Authorization:`Bearer ${login.token}`},body:JSON.stringify({content:original})});
      assert(restored.ok,'restore isolated fixture HTML');
    }
    await page.locator('[data-guia="red.files"]').click();
    await page.getByRole('button',{name:'Subir archivo'}).waitFor();
    await page.locator('[data-guia="red.access"]').click();
    await page.getByRole('heading',{name:'Registro',exact:true}).waitFor();
    assert.deepEqual(errors,[]);
    await page.close();
    console.log(`PASS ${width}: actual HTML publication, public iframe and reload persistence, inline error/draft, service doors, storage/SSH, no JS errors/overflow`);
  }
  const page=await browser.newPage({viewport:{width:1440,height:1000},reducedMotion:'reduce'});
  await page.addInitScript(s=>{if(window===window.top)localStorage.setItem('legacy.session',JSON.stringify(s));},login);
  const attempts=[];
  const scans=[];
  await page.route('**/api/**',async route=>{
      try {
    const url=new URL(route.request().url());
    if(!url.pathname.startsWith('/api/')) return route.continue();
    if(url.pathname==='/api/internet/hack') {
      scans.push(route.request().postDataJSON());
      return route.fulfill({status:400,contentType:'application/json',body:JSON.stringify({error:'UI contract test: scan not enqueued'})});
    }
    if(url.pathname==='/api/internet/intrusions') {
      attempts.push(route.request().postDataJSON());
      return route.fulfill({status:400,contentType:'application/json',body:JSON.stringify({error:'UI contract test: no operation enqueued'})});
    }
    const response=await route.fetch({url:`${api}${url.pathname}${url.search}`});
    if(url.pathname===`/api/internet/${ip}`) {
      const machine=await response.json();
      return route.fulfill({json:{...machine,access:false,access_as:null,permissions:[],scanned:true}});
    }
    return await route.fulfill({response});
      } catch { if (!page.isClosed()) await route.abort().catch(()=>{}); }
  });
  await page.goto(`${base}/internet?ip=${ip}`, {waitUntil:'domcontentloaded',timeout:60000});
  await page.locator('[data-guia="red.force"]').click();
  await page.getByRole('button',{name:'Atacar Web'}).waitFor();
  for(const service of ['Web','Almacenamiento','SSH']) {
    await page.getByRole('button',{name:`Atacar ${service}`}).click();
    await page.waitForFunction(()=>document.querySelector('.status')?.textContent.includes('UI contract test'),null,{timeout:1000}).catch(()=>{});
  }
  assert.deepEqual(attempts.map(a=>a.capability),['web_exploit','ftp_exploit','ssh_exploit']);
  assert(attempts.every(a=>a.ip===ip && a.software_id>0));
  await page.getByRole('button',{name:'Volver a escanear'}).click();
  await page.waitForTimeout(200);
  assert.equal(scans.length,1);
  assert(scans[0].kind==='scan' && scans[0].ip===ip && scans[0].software_id>0);
  await page.locator('.intrusion').screenshot({path:'/tmp/hackaton-tutorial-check/service-attack-selection.png'});
  await page.goto(`${base}/lab?capability=port_scan`, {waitUntil:'domcontentloaded',timeout:60000});
  await page.waitForFunction(()=>document.querySelector('[data-guia="lab.capacidad"]')?.value==='port_scan');
  await page.close();
  console.log('PASS attack/scan UI contract: selected software and exact IP; Lab link preselects port scanner');
} finally {await browser.close();}
