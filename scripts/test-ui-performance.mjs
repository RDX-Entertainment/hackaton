// Read-only browser measurements. API reads are redirected to isolated localhost.
import assert from 'node:assert/strict';
import { chromium } from '/tmp/hackaton-tutorial-check/node_modules/playwright/index.mjs';
const local='http://127.0.0.1:5174';
const optimized=process.argv[2] || local;
const api='http://127.0.0.1:18091';
const browser=await chromium.launch({headless:true,executablePath:'/home/raul/.cache/ms-playwright/chromium_headless_shell-1243/chrome-headless-shell-linux64/chrome-headless-shell',args:['--no-sandbox']});
try {
  const results=[];
  for (const base of ['https://hacketon.raulabz.online',optimized]) {
    const page=await browser.newPage({viewport:{width:1920,height:1080},reducedMotion:'no-preference'});
    await page.goto(`${base}/login`);await page.waitForTimeout(2000);
    const client=await page.context().newCDPSession(page);await client.send('Performance.enable');
    const metrics=async()=>Object.fromEntries((await client.send('Performance.getMetrics')).metrics.map(m=>[m.name,m.value]));
    const a=await metrics();await page.waitForTimeout(6000);const b=await metrics();
    const result={base,layouts:b.LayoutCount-a.LayoutCount,styles:b.RecalcStyleCount-a.RecalcStyleCount,mainThreadMs:Math.round((b.TaskDuration-a.TaskDuration)*1000)};
    console.log(JSON.stringify(result));results.push(result);
    if (base===optimized) {
      assert(result.layouts<10,'CRT should not trigger continuous layout');
      const info=await page.evaluate(()=>({grain:getComputedStyle(document.querySelector('.capa--grano')).animationName,keyframes:document.getAnimations().filter(a=>a.animationName==='rueda-tubo').flatMap(a=>a.effect.getKeyframes())}));
      assert.equal(info.grain,'none');assert(info.keyframes.length);assert(info.keyframes.every(k=>k.transform && !('top' in k)));
      await page.emulateMedia({reducedMotion:'reduce'});
      assert.equal(await page.locator('.capa--barrido').evaluate(el=>getComputedStyle(el,'::after').display),'none');
    }
    await page.close();
  }
  // Count the component function itself, not DOM mutations or child countdown commits.
  const login=await (await fetch(`${api}/api/auth/login`,{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify({username:'lmmtxiy3mn',password:'HardwareTestOnly2026'})})).json();
  assert(login.token);
  const page=await browser.newPage({viewport:{width:1440,height:1100},reducedMotion:'reduce'});
  await page.addInitScript(s=>localStorage.setItem('legacy.session',JSON.stringify(s)),login);
  const calls={};
  let instrumented=false;
  await page.route('**/src/components/arsenal/ModuleWorkshop.tsx*',async route=>{
    const response=await route.fetch();let body=await response.text();
    const marker=/function ModuleWorkshop\d*\(\{ programs, capabilities, onChanged \}\) \{/;
    assert(marker.test(body),'Vite instrumentation target missing');
    body=body.replace(marker,match=>`${match} globalThis.__workshopRenders = (globalThis.__workshopRenders || 0) + 1;`);
    instrumented=true;await route.fulfill({response,body});
  });
  await page.route('**/api/**',async route=>{
    const url=new URL(route.request().url());if(!url.pathname.startsWith('/api/'))return route.continue();
    calls[url.pathname]=(calls[url.pathname]??0)+1;
    try {
      const response=await route.fetch({url:`${api}${url.pathname}${url.search}`});
      return await route.fulfill({response});
    } catch {await route.abort().catch(()=>{});}
  });
  await page.goto(`${local}/lab`);await page.locator('[data-guia="lab.mejorar"]').click();
  await page.waitForTimeout(1500);
  const before=await page.evaluate(()=>globalThis.__workshopRenders);
  const catalogCalls=calls['/api/software-system/capabilities'];
  await page.waitForTimeout(12000);
  const after=await page.evaluate(()=>globalThis.__workshopRenders);
  assert(instrumented);assert.equal(after-before,0,'Inventory must not render on countdown or unchanged polling');
  assert.equal(calls['/api/software-system/capabilities'],catalogCalls,'Catalog should not poll');
  // Simulate the browser visibility API locally; this does not alter application data.
  await page.evaluate(()=>{Object.defineProperty(document,'hidden',{configurable:true,get:()=>true});document.dispatchEvent(new Event('visibilitychange'));});
  await page.waitForTimeout(500);
  const beforeHidden=calls['/api/software-system/programs'];
  await page.waitForTimeout(5500);
  assert.equal(calls['/api/software-system/programs'],beforeHidden,'Hidden lab should not poll');
  await page.evaluate(()=>{delete document.hidden;document.dispatchEvent(new Event('visibilitychange'));});
  await page.waitForTimeout(1000);
  assert(calls['/api/software-system/programs']>beforeHidden,'Visible lab should refresh');
  console.log('PASS no idle inventory renders, static catalogs, hidden-tab polling pauses/resumes');
  await page.close();
} finally {await browser.close();}
