const {chromium}=require('/home/daidaiiro/.npm/_npx/705bc6b22212b352/node_modules/playwright');
const fs=require('fs'),path=require('path');
const root=path.resolve(__dirname,'../workspace');
const out=path.join(root,'structure-review-20261024');fs.mkdirSync(out,{recursive:true});
(async()=>{
 const browser=await chromium.launch({executablePath:'/home/daidaiiro/.cache/ms-playwright/chromium-1234/chrome-linux64/chrome',headless:true,args:['--no-sandbox']});
 try{
  const page=await browser.newPage({viewport:{width:1440,height:1050}}),errors=[];
  page.on('pageerror',error=>errors.push(String(error)));
  await page.goto('http://127.0.0.1:8772/design-review.html?deck=lead&structure=1#lead');
  await page.waitForTimeout(400);
  const result={cards:await page.locator('.structure-card').count(),open:await page.locator('#intro-structure').evaluate(e=>e.open),desktopOverflow:await page.evaluate(()=>document.documentElement.scrollWidth>innerWidth),errors};
  await page.screenshot({path:path.join(out,'desktop.png')});
  await page.locator('#structure-slide-6').screenshot({path:path.join(out,'slide-6-structure.png')});
  await page.locator('#structure-slide-6 details').evaluate(e=>e.open=true);
  await page.locator('#structure-slide-6').screenshot({path:path.join(out,'slide-6-narration.png')});
  await page.setViewportSize({width:390,height:844});
  await page.locator('#structure-slide-6 details').evaluate(e=>e.open=false);
  await page.evaluate(()=>{document.activeElement?.blur();document.documentElement.style.scrollBehavior='auto';document.getElementById('structure-slide-6').scrollIntoView({block:'start',behavior:'instant'});});
  await page.screenshot({path:path.join(out,'mobile.png')});
  result.mobileOverflow=await page.evaluate(()=>document.documentElement.scrollWidth>innerWidth);
  fs.writeFileSync(path.join(out,'render-result.json'),JSON.stringify(result,null,2)+'\n');
  console.log(JSON.stringify(result));
 }finally{await browser.close();}
})().catch(error=>{console.error(error);process.exit(1)});
