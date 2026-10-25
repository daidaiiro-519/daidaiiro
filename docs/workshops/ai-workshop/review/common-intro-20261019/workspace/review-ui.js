const data=JSON.parse(document.querySelector('#data').textContent);
const reviewDiffs=JSON.parse(document.querySelector('#review-diffs').textContent);
const el=id=>document.getElementById(id);
let current=data[0],n=1,visualMode='pair',scriptMode='diff';

function frameSource(id,base){
  const frame=el(id),url=base+'#'+n;
  if(frame.getAttribute('src')!==url)frame.setAttribute('src',url);
}
function showDiff(){
  const row=reviewDiffs[current.id]?.[n-1];
  el('script-diff').replaceChildren();
  if(!row){
    const p=document.createElement('p');
    p.textContent='この枚は変更前の原稿が保存されていないため、原稿差分を表示できません。変更後の全文を確認できます。';
    el('script-diff').append(p);
    el('change-summary').textContent=current.addedSlides?.includes(n)?'新規追加：変更前のスライド・原稿はありません':'原稿差分：変更前の記録なし';
    return;
  }
  const labels={heading:'見出し',lede:'導入文',narration:'原稿'};
  const changed=Object.keys(labels).filter(key=>row[key].changed).map(key=>labels[key]);
  el('change-summary').textContent=changed.length?'文章の変更：'+changed.join('・'):'見出し・導入文・原稿の文章は変更なし';
  for(const [key,label] of Object.entries(labels)){
    const diff=row[key],details=document.createElement('details');
    details.className='diff-field';details.dataset.field=key;details.open=diff.changed;
    const summary=document.createElement('summary');summary.append(label);
    const count=document.createElement('span');
    count.textContent=diff.changed?`${diff.added}字追加 / ${diff.removed}字削除`:'変更なし';
    summary.append(count);details.append(summary);
    const panels=document.createElement('div');panels.className='pair diff-panels';
    for(const [side,title] of [['before','変更前'],['after','変更後']]){
      const panel=document.createElement('div');panel.className='diff-side';
      const heading=document.createElement('h3');heading.textContent=title;panel.append(heading);
      const content=document.createElement('div');content.className='diff-content';content.dataset.side=side;
      content.innerHTML=diff[side+'_html'];if(!content.textContent)content.textContent='記載なし';
      panel.append(content);panels.append(panel);
    }
    details.append(panels);el('script-diff').append(details);
  }
}
function displayModes(){
  el('slide-pair').hidden=visualMode!=='pair';
  el('overlay-panel').hidden=visualMode!=='overlay';
  if(visualMode==='overlay'){
    frameSource('overlay-before',current.before);frameSource('overlay-after',current.after);
  }
  const comparing=!!current.beforeNotes?.[n-1];
  el('script-diff').hidden=scriptMode!=='diff'||!comparing;
  el('narration-pair').hidden=scriptMode!=='pair'||!comparing;
  el('normal-notes').hidden=comparing;
  document.querySelector('.diff-legend').hidden=scriptMode!=='diff'||!comparing;
  for(const button of document.querySelectorAll('[data-visual-mode]'))button.setAttribute('aria-pressed',String(button.dataset.visualMode===visualMode));
  for(const button of document.querySelectorAll('[data-script-mode]')){
    button.setAttribute('aria-pressed',String(button.dataset.scriptMode===scriptMode));button.disabled=!comparing;
  }
}
function show(){
  n=Math.max(1,Math.min(n,current.notes.length));el('slide').value=String(n);
  frameSource('before',current.before);frameSource('after',current.after);
  el('before-open').href=current.before+'#'+n;el('after-open').href=current.after+'#'+n;
  el('reader-link').href='materials.html?slide='+(n+(current.readerSlideOffset||0))+'#'+encodeURIComponent(current.id);
  el('design-reason').textContent=current.reasons?.[String(n)]||'この枚の変更理由は未記録です。';
  el('before-narration').innerHTML=current.beforeNotes?.[n-1]?.body||'';
  el('after-narration').innerHTML=(current.afterNotes||current.notes)[n-1].body;
  el('heading').textContent=current.notes[n-1].title;el('notes').innerHTML=current.notes[n-1].body;
  el('current-title').textContent=current.notes[n-1].title;
  const hasStructure=Array.isArray(current.structureAppliedSlides);
  const structureApplied=hasStructure&&current.structureAppliedSlides.includes(n);
  const structureApproved=current.structureApprovedSlides?.includes(n);
  el('structure-status').hidden=!hasStructure;
  el('structure-status').textContent=structureApplied
    ?(structureApproved?'今回の構成を図・原稿へ反映済みです。利用者確認済みです。':'今回の構成を図・原稿へ反映済みです。利用者確認待ちです。')
    :'この資料の構成案は、まだ図・原稿へ反映していません。上部の構成案を確認してください。';
  el('after-version-label').textContent=hasStructure
    ?(structureApplied?'変更後（今回の構成を反映済み）':'変更後（今回の構成案は未反映）'):'変更後';
  el('slide-count').textContent=n+' / '+current.notes.length;
  el('bottom-count').textContent=n+' / '+current.notes.length;
  el('prev').disabled=n===1;el('next').disabled=n===current.notes.length;
  el('prev-bottom').disabled=n===1;el('next-bottom').disabled=n===current.notes.length;
  showDiff();displayModes();
  const url=new URL(location.href);url.searchParams.set('deck',current.id);url.searchParams.set('slide',String(n));
  history.replaceState(null,'',url);
}
function choose(initial=false){
  current=data.find(deck=>deck.id===el('deck').value)||data[0];
  if(el('orientation-plan'))el('orientation-plan').hidden=current.id!=='orientation';
  el('slide').replaceChildren(...current.notes.map((note,index)=>{
    const option=document.createElement('option');option.value=String(index+1);
    const status=Array.isArray(current.structureAppliedSlides)
      ?(current.structureAppliedSlides.includes(index+1)?'［構成反映済み］':'［構成未反映］'):'';
    option.textContent=(index+1)+'　'+status+note.title;return option;
  }));
  n=1;
  if(initial){
    const requested=Number(new URLSearchParams(location.search).get('slide'));
    if(Number.isInteger(requested)&&requested>=1&&requested<=current.notes.length)n=requested;
  }
  show();
}
for(const deck of data){
  const option=document.createElement('option');option.value=deck.id;option.textContent=deck.name;el('deck').append(option);
}
for(const id of ['before','after','overlay-before','overlay-after'])el(id).onload=()=>{
  const doc=el(id).contentDocument;if(!doc?.head)return;
  const style=doc.createElement('style');style.textContent='.slide.on{animation:none!important}';doc.head.append(style);
};
el('deck').onchange=()=>choose();
el('slide').onchange=()=>{n=Number(el('slide').value);show();};
el('prev').onclick=()=>{n--;show();};el('next').onclick=()=>{n++;show();};
el('prev-bottom').onclick=()=>{n--;show();el('review-workspace').scrollIntoView();};
el('next-bottom').onclick=()=>{n++;show();el('review-workspace').scrollIntoView();};
for(const button of document.querySelectorAll('[data-visual-mode]'))button.onclick=()=>{visualMode=button.dataset.visualMode;displayModes();};
for(const button of document.querySelectorAll('[data-script-mode]'))button.onclick=()=>{scriptMode=button.dataset.scriptMode;displayModes();};
el('overlay-range').oninput=()=>{
  const value=el('overlay-range').value;el('overlay-stage').style.setProperty('--reveal',value+'%');
  el('overlay-value').textContent=value+'%';el('overlay-range').setAttribute('aria-valuetext','変更後を'+value+'パーセント表示');
};
document.addEventListener('keydown',event=>{
  if(event.altKey||event.ctrlKey||event.metaKey||event.shiftKey||/^(INPUT|SELECT|TEXTAREA|BUTTON)$/.test(event.target.tagName)||event.target.isContentEditable)return;
  if(event.key==='ArrowRight'&&n<current.notes.length){event.preventDefault();n++;show();}
  if(event.key==='ArrowLeft'&&n>1){event.preventDefault();n--;show();}
});
for(const link of document.querySelectorAll('.task-nav a'))link.addEventListener('click',()=>{
  if(!link.getAttribute('href').startsWith('#'))return;
  const id=link.getAttribute('href').slice(1),target=document.getElementById(id);
  if(target?.tagName==='DETAILS')target.open=true;
});
document.addEventListener('play',event=>{if(event.target.tagName==='VIDEO')document.querySelectorAll('video').forEach(video=>{if(video!==event.target)video.pause();});},true);
const params=new URLSearchParams(location.search),requestedDeck=params.get('deck')||location.hash.slice(1);
el('deck').value=data.some(deck=>deck.id===requestedDeck)?requestedDeck:data[0].id;
choose(true);
