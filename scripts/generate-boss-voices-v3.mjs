#!/usr/bin/env node
/** Clean, bounded, resumable Cloudflare Aura-2 regeneration for all 25 bosses. */
import { readFile, writeFile, mkdir } from 'node:fs/promises';
import { spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { parseArgs } from 'node:util';
const {values}=parseArgs({options:{account:{type:'string'},limit:{type:'string',default:'220'},'dry-run':{type:'boolean'}}});
const plan=JSON.parse(await readFile('art/boss-voices-v3/plan.json','utf8'));
const jobs=plan.bosses.flatMap(b=>[
 ...b.lines.map(([cue,text],i)=>({boss:b,id:`${b.id}-${i+1}`,cue,text})),
 ...b.death.map((text,i)=>({boss:b,id:`boss-${b.id}${i?'-v2':''}`,cue:'death',text})),
]);
const characters=jobs.reduce((n,j)=>n+j.text.length,0);
console.log(JSON.stringify({model:plan.model,clips:jobs.length,characters,estimatedUsd:characters*.03/1000,estimatedNeurons:Math.ceil(characters*.03/1000/.011*1000)}));
if(values['dry-run'])process.exit(0);
let envText='';try{envText=await readFile('.env','utf8');}catch{}
const env=Object.fromEntries(envText.split(/\r?\n/).flatMap(l=>{const m=l.match(/^\s*(\w+)\s*=\s*(.*?)\s*$/);return m?[[m[1],m[2].replace(/^['"]|['"]$/g,'')]]:[]}));
const key=process.env.CF_API_KEY||env.CF_API_KEY,account=values.account||process.env.CF_ACCOUNT_ID||env.CF_ACCOUNT_ID;
if(!key||!/^[a-f0-9]{32}$/i.test(account??''))throw Error('CF_API_KEY and Cloudflare account ID required.');
const limit=Number(values.limit);if(!Number.isInteger(limit)||limit<1||limit>220)throw Error('limit must be 1–220');
const filter='highpass=f=65,acompressor=threshold=0.18:ratio=1.4:attack=12:release=180:makeup=1,loudnorm=I=-16:TP=-1.5:LRA=9,afade=t=in:d=0.012';
await mkdir('art/boss-voices-v3/raw',{recursive:true});await mkdir('public/game/voices',{recursive:true});
let generated=0;const report=[];
for(const j of jobs){
 const raw=`art/boss-voices-v3/raw/${j.id}.wav`,path=`public/game/voices/${j.id}.mp3`;
 const request={text:j.text,speaker:j.boss.speaker,encoding:'linear16',container:'wav',sample_rate:24000};
 const signature=createHash('sha256').update(JSON.stringify([3,plan.model,request])).digest('hex');
 let cached=false;try{cached=JSON.parse(await readFile(raw+'.json','utf8')).signature===signature;}catch{}
 if(!cached){
  if(generated>=limit)throw Error('Generation limit reached; rerun to resume.');
  let bytes;
  for(let attempt=0;attempt<3;attempt++){
   const response=await fetch(`https://api.cloudflare.com/client/v4/accounts/${account}/ai/run/${plan.model}`,{method:'POST',headers:{Authorization:`Bearer ${key}`,'Content-Type':'application/json'},body:JSON.stringify(request),signal:AbortSignal.timeout(90000)});
   bytes=Buffer.from(await response.arrayBuffer());
   if(response.ok&&bytes.subarray(0,4).toString()==='RIFF')break;
   if((response.status===429||response.status>=500)&&attempt<2){await new Promise(r=>setTimeout(r,2000*(attempt+1)));continue;}
   throw Error(`Cloudflare ${response.status}: ${bytes.toString('utf8').slice(0,400)}`);
  }
  await writeFile(raw,bytes);await writeFile(raw+'.json',JSON.stringify({signature,model:plan.model,request,generatedAt:new Date().toISOString()},null,2)+'\n');generated++;
 }
 const encoded=spawnSync('ffmpeg',['-hide_banner','-loglevel','error','-y','-i',raw,'-af',filter,'-ac','1','-ar','24000','-codec:a','libmp3lame','-b:a','192k',path],{encoding:'utf8'});
 if(encoded.status!==0)throw Error(encoded.stderr);
 const probed=spawnSync('ffprobe',['-v','error','-show_entries','format=duration','-of','default=nw=1:nk=1',path],{encoding:'utf8'});
 const duration=Number(probed.stdout.trim());if(!(duration>.2&&duration<12))throw Error(`Invalid clip ${j.id}: ${duration}`);
 report.push({id:j.id,cue:j.cue,text:j.text,url:`/game/voices/${j.id}.mp3`,duration});
 console.log(`${cached?'cached':'generated'} ${j.id} ${duration.toFixed(2)}s`);
 await writeFile('art/boss-voices-v3/usage.json',JSON.stringify({model:plan.model,completed:report.length,total:jobs.length,generated,generatedCharacters:jobs.filter(x=>report.some(y=>y.id===x.id)).reduce((n,x)=>n+x.text.length,0),estimatedUsd:characters*.03/1000,report},null,2)+'\n');
}
const manifestPath='public/game/voices/manifest.json';const manifest=JSON.parse(await readFile(manifestPath,'utf8'));
manifest.enemies=manifest.enemies.filter(e=>e.skin<12);
for(const b of plan.bosses)manifest.enemies.push({skin:b.skin,id:b.id,name:b.name,style:b.style,speaker:b.speaker,lines:report.filter(r=>r.id.startsWith(b.id+'-'))});
await writeFile(manifestPath,JSON.stringify(manifest,null,2)+'\n');
const regular=JSON.parse(await readFile('art/blacksite-voices.json','utf8'));
regular.enemies=[...regular.enemies.filter(e=>e.skin<12),...plan.bosses.map(b=>({skin:b.skin,id:b.id,name:b.name,speaker:b.speaker,style:b.style,effect:'clean',lines:b.lines}))];
await writeFile('art/blacksite-voices.json',JSON.stringify(regular,null,2)+'\n');
const deathPlan={model:plan.model,source:'Clean Aura-2 v3 boss performances',bosses:plan.bosses.flatMap(b=>b.death.map((text,i)=>({id:`boss-${b.id}${i?'-v2':''}`,speaker:b.speaker,style:b.style,effect:'clean',text})))};
await writeFile('art/boss-voices.json',JSON.stringify(deathPlan,null,2)+'\n');
await writeFile('art/source_hd/voices/boss-usage.json',JSON.stringify({report:report.filter(r=>r.cue==='death')},null,2)+'\n');
const catalogPath='src/lib/asset-catalog-data.json';const catalog=JSON.parse(await readFile(catalogPath,'utf8'));
for(const r of report){const bytes=await readFile('public'+r.url);let row=catalog.find(e=>e.file===r.url);if(!row){row={file:r.url,name:r.id+'.mp3',group:'voices',category:'voice',size:0,hash:'',shortHash:''};catalog.push(row);}row.size=bytes.length;row.hash=createHash('sha256').update(bytes).digest('hex');row.shortHash=row.hash.slice(0,8);}
await writeFile(catalogPath,JSON.stringify(catalog,null,2)+'\n');
console.log(`Complete: ${report.length} clean boss clips.`);
