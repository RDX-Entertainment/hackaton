// UI timing contract. All API reads use localhost; compilation/progress are simulated.
import assert from 'node:assert/strict';
import { chromium } from '/tmp/hackaton-tutorial-check/node_modules/playwright/index.mjs';
const api='http://127.0.0.1:18091';
const base=process.argv[2] || 'http://127.0.0.1:5174';
const response=await fetch(`${api}/api/auth/login`,{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify({username:'lmmtxiy3mn',password:'HardwareTestOnly2026'})});
assert(response.ok,`isolated login ${response.status}`);
const session=await response.json();
const browser=await chromium.launch({headless:true,executablePath:'/home/raul/.cache/ms-playwright/chromium_headless_shell-1243/chrome-headless-shell-linux64/chrome-headless-shell',args:['--no-sandbox']});
try {
 for(const width of [1440,390]) {
  const page=await browser.newPage({viewport:{width,height:1100}});
  const errors=[];page.on('pageerror',e=>errors.push(e.message));
  await page.addInitScript(s=>{if(window===window.top)localStorage.setItem('legacy.session',JSON.stringify(s));},session);
  let active=false, mode='running', calls=0, started=0, selected;
  const pid=900001;
  const snapshot=()=>({id:pid,action:91,info:'compile',details:'compile',ideal_seconds:60,worked_seconds:0,seconds_left:mode==='paused'?40:Math.max(0,60-Math.floor((Date.now()-started)/1000)),paused:mode==='paused',completed:mode==='succeeded',status:mode,result_code:null,result_message:mode==='succeeded'?'Compilación de prueba completada':null});
  await page.route('**/api/**',async route=>{
   try {
   const url=new URL(route.request().url());
   if(!url.pathname.startsWith('/api/'))return route.continue();
   if(url.pathname==='/api/software-system/compile') {
    active=true;started=Date.now();selected=route.request().postDataJSON().blueprint_id;
    return route.fulfill({status:201,json:{process_id:pid,blueprint_id:selected,name:'Prueba de progreso',capability:'cracker',seconds:60,stats:{}}});
   }
   if(url.pathname===`/api/processes/${pid}`) {calls++;return route.fulfill({json:snapshot()});}
   const upstream=await route.fetch({url:`${api}${url.pathname}${url.search}`});
   if(url.pathname==='/api/processes') {
    const rows=await upstream.json();
    return route.fulfill({json:[...(active?[snapshot()]:[]),...rows.filter(r=>r.action!==91)]});
   }
   return await route.fulfill({response:upstream});
   } catch { if (!page.isClosed()) await route.abort().catch(()=>{}); }
  });
  await page.goto(`${base}/lab`, {waitUntil:'domcontentloaded',timeout:60000});
  const card=page.locator('article.tarjeta').filter({has:page.getByRole('button',{name:'Compilar'})}).first();
  await card.getByRole('button',{name:'Compilar'}).click();
  const bar=card.locator('.process-progress');
  await bar.waitFor();
  assert(selected>0);
  assert.equal(await page.locator('.process-progress').count(),1);
  assert.match(await bar.innerText(),/Compilando/);
  await page.waitForTimeout(250);
  const scale=()=>bar.locator('.process-progress__fill').evaluate(e=>Number(getComputedStyle(e).transform.match(/matrix\(([^,]+)/)?.[1]??0));
  const before=await scale();await page.waitForTimeout(250);assert(await scale()>before,'smooth progress between polls');
  assert(!/\d+\.\d+\s*\//.test(await bar.innerText()),'no fractional-second noise');
  assert(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth));
  await card.screenshot({path:`/tmp/hackaton-tutorial-check/inline-progress-${width}.png`});
  mode='paused';await bar.locator('small').filter({hasText:'en pausa'}).waitFor();
  const paused=await scale();await page.waitForTimeout(300);assert.equal(await scale(),paused);
  await page.evaluate(()=>{Object.defineProperty(document,'hidden',{configurable:true,value:true});document.dispatchEvent(new Event('visibilitychange'));});
  const hiddenCalls=calls;await page.waitForTimeout(1250);assert.equal(calls,hiddenCalls);
  await page.evaluate(()=>{Object.defineProperty(document,'hidden',{configurable:true,value:false});document.dispatchEvent(new Event('visibilitychange'));});
  mode='succeeded';await bar.getByRole('status').waitFor();
  assert.equal(await bar.getByRole('progressbar').getAttribute('aria-valuenow'),'100');
  const finalCalls=calls;await page.waitForTimeout(1500);assert.equal(calls,finalCalls,'completed process stops polling');
  assert.deepEqual(errors,[]);
  await page.close();
  console.log(`PASS ${width}: compilation bar next to button, smooth glow, readable seconds, pause/hidden/completed timers, no overflow`);
 }
} finally {await browser.close();}
