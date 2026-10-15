import { test } from 'node:test';
import assert from 'node:assert/strict';
import { chromium } from 'playwright';

test('pointer capture cannot restart after pause, stop or a newer capture request', async () => {
  const browser=await chromium.launch({headless:true});
  try {
    const page=await browser.newPage();
    const errors=[];page.on('pageerror',error=>errors.push(error.message));
    await page.route('**/__runtime_input.html',route=>route.fulfill({contentType:'text/html',body:'<!doctype html><body></body>'}));
    await page.goto(new URL('/__runtime_input.html',process.env.BLACKSITE_TEST_URL??'http://127.0.0.1:8080/').href);
    const result=await page.evaluate(async()=>{
      const {BlacksiteRuntime}=await import('/src/game/runtime.ts');
      const flush=()=>new Promise(resolve=>setTimeout(resolve,0));
      const create=()=>{
        const canvas=document.createElement('canvas');document.body.append(canvas);
        const rt=new BlacksiteRuntime(canvas,{onHud(){},onState(){}});
        // These are input-only scenarios: no simulation, renderer or media is needed.
        rt.audio={unlock(){},setMusic(){},clearEnemies(){},ui(){},setBoss(){},dispose(){}};
        return {canvas,rt};
      };
      const cases={};
      for(const action of ['paused','resumed','stopped']) {
        const {canvas,rt}=create();let reject,calls=0;
        canvas.requestPointerLock=()=>{calls++;return calls===1?new Promise((_,fail)=>{reject=fail;}):Promise.resolve();};
        rt.setPlaying(true);rt.requestLock();
        if(action==='stopped')rt.stop();
        else {rt.setPlaying(false);if(action==='resumed')rt.setPlaying(true);}
        reject(new Error('raw mouse unavailable'));await flush();
        cases[action]=calls;
      }
      {
        const {canvas,rt}=create();let calls=0;
        canvas.requestPointerLock=()=>{calls++;return Promise.resolve();};
        rt.requestLock();await flush();cases.inactive=calls;
      }
      {
        const {canvas,rt}=create();let calls=0;const failures=[];
        canvas.requestPointerLock=options=>{calls++;return options?new Promise((_,reject)=>failures.push(reject)):Promise.resolve();};
        rt.setPlaying(true);rt.requestLock();rt.requestLock();
        failures[0](new Error('superseded raw mouse request'));await flush();
        cases.superseded=calls;
        failures[1](new Error('raw mouse unavailable'));await flush();
        cases.activeFallback=calls;
      }
      {
        const {canvas,rt}=create();let resolve,exits=0;
        canvas.requestPointerLock=()=>new Promise(done=>{resolve=done;});
        const oldExit=document.exitPointerLock;
        document.exitPointerLock=()=>{exits++;};
        Object.defineProperty(document,'pointerLockElement',{configurable:true,get:()=>canvas});
        rt.setPlaying(true);rt.requestLock();rt.setPlaying(false);
        resolve();await flush();cases.lateCaptureReleased=exits;
        delete document.pointerLockElement;document.exitPointerLock=oldExit;
      }
      {
        const {canvas,rt}=create();let exits=0;
        const oldExit=document.exitPointerLock;
        document.exitPointerLock=()=>{exits++;};
        Object.defineProperty(document,'pointerLockElement',{configurable:true,get:()=>canvas});
        rt.bind();
        document.dispatchEvent(new Event('pointerlockchange'));
        rt.unbind();cases.legacyCaptureReleased=exits;
        delete document.pointerLockElement;document.exitPointerLock=oldExit;
      }
      return cases;
    });
    assert.deepEqual(result,{paused:1,resumed:1,stopped:1,inactive:0,superseded:2,activeFallback:3,lateCaptureReleased:1,legacyCaptureReleased:1});
    assert.deepEqual(errors,[]);
  } finally {await browser.close();}
});
