// Browser integration against the isolated API, including when inspecting production assets.
import assert from 'node:assert/strict';
import { chromium } from '/tmp/hackaton-tutorial-check/node_modules/playwright/index.mjs';
const api='http://127.0.0.1:18091';
const base=process.argv[2] || 'http://127.0.0.1:5174';
async function session(username) {
  const response=await fetch(`${api}/api/auth/login`,{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify({username,password:'HardwareTestOnly2026'})});
  assert(response.ok);return response.json();
}
const user=await session('lmmtxiy3mn');
const admin=await session('hardwaretest');
const browser=await chromium.launch({headless:true,executablePath:'/home/raul/.cache/ms-playwright/chromium_headless_shell-1243/chrome-headless-shell-linux64/chrome-headless-shell',args:['--no-sandbox']});
async function pageFor(login,width) {
  const page=await browser.newPage({viewport:{width,height:1100},reducedMotion:'reduce'});
  await page.addInitScript(s=>localStorage.setItem('legacy.session',JSON.stringify(s)),login);
  await page.route('**/api/**',async route=>{
    const url=new URL(route.request().url());
    if (!url.pathname.startsWith('/api/')) return route.continue();
    try { return await route.fulfill({response:await route.fetch({url:`${api}${url.pathname}${url.search}`})}); }
    catch { console.log('Isolated API request failed:',url.pathname);await route.abort().catch(()=>{}); }
  });
  return page;
}
try {
  for (const width of [1440,390]) {
    const headers={Authorization:`Bearer ${user.token}`,'Content-Type':'application/json'};
    const programs=await (await fetch(`${api}/api/software-system/programs`,{headers})).json();
    const testProgram=programs.find(p=>p.name==='mw64-ui');assert(testProgram);
    const inventory=await (await fetch(`${api}/api/software-system/workshop`,{headers})).json();
    for (const m of inventory.equipped.filter(m=>m.software_id===testProgram.software_id)) {
      const removed=await fetch(`${api}/api/software-system/workshop/unequip`,{method:'POST',headers,body:JSON.stringify({software_id:testProgram.software_id,slot:m.slot})});assert(removed.ok);
    }
    const freshInventory=await (await fetch(`${api}/api/software-system/workshop`,{headers})).json();
    const sensor=freshInventory.catalog.find(m=>m.code==='heuristic_sensor');
    const stock=freshInventory.stock.find(s=>s.module_id===sensor.id && s.quantity>0);assert(stock);
    const page=await pageFor(user,width);
    const errors=[];page.on('pageerror',e=>errors.push(e.message));
    await page.goto(`${base}/lab`);
    await page.locator('[data-guia="lab.mejorar"]').click();
    const workshop=page.locator('.module-workshop');
    try { await workshop.locator(':scope > label select').selectOption({label:'mw64-ui · nivel 1 · 1 ranuras'}); }
    catch(e) { console.log('UI diagnostic',errors,(await page.locator('body').innerText()).slice(-5000));await page.screenshot({path:'/tmp/hackaton-tutorial-check/modules-debug.png'});throw e; }
    assert.equal(await workshop.locator('.module-slots button:disabled').count(),5);
    const moduleSelect=workshop.locator('.aforge__design select');
    await moduleSelect.selectOption(`${stock.module_id}:${stock.tier}`);
    await page.waitForFunction(value=>document.querySelector('.module-workshop .aforge__preview')?.textContent.includes(`Detección${value}`),Math.round(30*[1,1.35,1.8,2.4,3.2][stock.tier]));
    assert.equal(await workshop.locator('.aforge__preview').getByText('Limpieza',{exact:true}).count(),0);
    await workshop.getByRole('button',{name:'Instalar módulo'}).click();
    await workshop.getByRole('button',{name:'Retirar módulo'}).waitFor();
    await page.waitForFunction(()=>document.querySelector('.module-workshop .aforge__preview')?.textContent.includes('Detectar virus'));
    await workshop.locator('.aforge__workbench').screenshot({path:`/tmp/hackaton-tutorial-check/module-workshop-${width}.png`});
    assert(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth));
    await workshop.getByRole('button',{name:'Retirar módulo'}).click();
    await workshop.getByRole('button',{name:'Instalar módulo'}).waitFor();
    await page.waitForFunction(()=>!document.querySelector('.module-workshop .aforge__preview')?.textContent.includes('Detectar virus'));
    assert.deepEqual(errors,[]);
    await page.close();
    const editor=await pageFor(admin,width);
    const adminErrors=[];editor.on('pageerror',e=>adminErrors.push(e.message));
    await editor.goto(`${base}/admin/software`);
    await editor.getByRole('button',{name:'Módulos'}).click();
    await editor.locator('.module-admin > label select').selectOption({label:'Sensor heurístico'});
    assert.equal(await editor.getByLabel('Detección',{exact:true}).inputValue(),'30');
    assert(await editor.getByLabel('Solo detectar virus',{exact:true}).isChecked());
    assert(!(await editor.getByLabel('Solo eliminar virus',{exact:true}).isChecked()));
    assert(await editor.evaluate(()=>document.documentElement.scrollWidth<=innerWidth));
    await editor.locator('.module-admin').screenshot({path:`/tmp/hackaton-tutorial-check/module-admin-${width}.png`});
    assert.deepEqual(adminErrors,[]);
    await editor.close();
    console.log(`PASS UI ${width}: slots, real install/remove, category and granted stats, admin template editor, no overflow/errors`);
  }
} finally {await browser.close();}
