import {test} from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {chromium} from 'playwright';
const root=new URL('../',import.meta.url);
const weapons=JSON.parse(readFileSync(new URL('src/lib/draft-weapons-v2-data.json',root)));
test('all 33 weapons have no disconnected panel strips in animation frames',async()=>{
 const browser=await chromium.launch({headless:true});
 try{const page=await browser.newPage();
  for(const weapon of weapons){
   const bytes=readFileSync(new URL('public'+weapon.sheet_file,root));
   const bad=await page.evaluate(async(src)=>{
    const im=new Image();im.src=src;await im.decode();const c=document.createElement('canvas');c.width=512;c.height=384;const ctx=c.getContext('2d',{willReadFrequently:true});const failures=[];
    for(const fr of [1,2,3,4,5,6,7,8,10,11,12,13,15,16,17,18,20,21,22,23]){
     ctx.clearRect(0,0,512,384);ctx.drawImage(im,fr%5*512,Math.floor(fr/5)*384,512,384,0,0,512,384);const data=ctx.getImageData(0,0,512,384).data;
     const seen=new Uint8Array(512*384),queue=new Int32Array(512*384);
     for(let start=0;start<seen.length;start++){
      if(seen[start]||data[start*4+3]<=10)continue;
      let head=0,tail=1;queue[0]=start;seen[start]=1;let x0=512,x1=0,y0=384,y1=0;
      while(head<tail){const i=queue[head++],x=i%512,y=Math.floor(i/512);x0=Math.min(x0,x);x1=Math.max(x1,x);y0=Math.min(y0,y);y1=Math.max(y1,y);
       for(const j of [x>0?i-1:-1,x<511?i+1:-1,y>0?i-512:-1,y<383?i+512:-1])if(j>=0&&!seen[j]&&data[j*4+3]>10){seen[j]=1;queue[tail++]=j;}
      }
      const w=x1-x0+1,h=y1-y0+1;if(tail>10&&Math.min(w,h)<=3&&Math.max(w,h)>12)failures.push({fr,w,h,pixels:tail});
     }
    }
    return failures;
   },'data:image/png;base64,'+bytes.toString('base64'));
   assert.deepEqual(bad,[],weapon.id+': neighboring panel strips remain');
  }
 }finally{await browser.close();}
});
