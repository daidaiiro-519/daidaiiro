const {chromium}=require('/home/daidaiiro/.npm/_npx/705bc6b22212b352/node_modules/playwright');
const fs=require('fs'),path=require('path'),assert=require('assert/strict');
const root=path.resolve(__dirname,'../workspace');
const layout=JSON.parse(fs.readFileSync(path.join(root,'review-layout.json'),'utf8'));
(async()=>{
 const browser=await chromium.launch({executablePath:'/home/daidaiiro/.cache/ms-playwright/chromium-1234/chrome-linux64/chrome',headless:true,args:['--no-sandbox']});
 try {
  const page=await browser.newPage({viewport:{width:1440,height:1100}}),errors=[];
  page.on('pageerror',e=>errors.push(String(e)));
  await page.goto('http://127.0.0.1:8772/design-review.html?deck=lead&slide=5#lead');
  assert.equal(await page.locator('#slide option').count(),8);
  assert((await page.locator('#change-summary').textContent()).includes('新規追加'));
  assert(await page.locator('[data-script-mode="diff"]').isDisabled());
  const data=await page.locator('#data').textContent();const lead=JSON.parse(data).find(d=>d.id==='lead');
  assert.equal(lead.beforeNotes[4],null);assert.equal(lead.notes.length,8);
  for(let n=1;n<=8;n++){
   await page.locator('#slide').selectOption(String(n));
   const expected=JSON.parse(fs.readFileSync(path.join(root,layout.sources[n-1]),'utf8'));
   assert.equal(await page.locator('#current-title').textContent(),expected.heading);
   assert((await page.locator('#after-narration').textContent()).includes(expected.narration.split('\n\n')[0]));
   const source=layout.before_sources[n-1];
   if(source){
    const before=JSON.parse(fs.readFileSync(path.join(root,layout.baseline,source),'utf8'));
    assert(lead.beforeNotes[n-1].body.includes(before.heading));
   }
  }
  await page.locator('#slide').selectOption('5');await page.locator('#reader-link').click();
  await page.waitForFunction(()=>document.querySelector('#slide-select')?.value==='5');
  assert.equal(await page.locator('#page-number').textContent(),'5 / 8');
  assert.equal(await page.locator('#script-title').textContent(),lead.notes[4].title);
  await page.locator('#slide-select').selectOption('8');await page.reload();
  await page.waitForFunction(()=>document.querySelector('#slide-select')?.value==='8');
  assert.equal(await page.locator('#script-title').textContent(),lead.notes[7].title);
  await page.locator('#compare-current').click();
  await page.waitForFunction(()=>document.querySelector('#slide')?.value==='8');
  assert.equal(await page.locator('#current-title').textContent(),lead.notes[7].title);
  await page.setViewportSize({width:390,height:844});
  for(const url of ['design-review.html?deck=lead&slide=5#lead','materials.html?slide=5#lead']){
   await page.goto('http://127.0.0.1:8772/'+url);
   assert(!(await page.evaluate(()=>document.documentElement.scrollWidth>innerWidth)));
  }
  assert.deepEqual(errors,[]);
  const result={slides:8,addedSlideHasNoInventedBefore:true,beforeMapping:true,narrationMapping:true,readerRoundTrip:true,mobileOverflow:false,errors};
  fs.writeFileSync(path.join(root,'seven-slide-revision-20261019/eight-slide-layout-check.json'),JSON.stringify(result,null,2)+'\n');console.log(JSON.stringify(result));
 } finally {await browser.close();}
})().catch(e=>{console.error(e);process.exit(1)});
