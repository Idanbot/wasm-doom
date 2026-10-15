import { test } from 'node:test';
import assert from 'node:assert/strict';
import { chromium } from 'playwright';

test('returning from the local asset catalog preserves the live game canvas and inputs', {timeout:180000}, async()=>{
  const browser=await chromium.launch({headless:true});
  try {
    const page=await browser.newPage({viewport:{width:960,height:600}});
    const errors=[];page.on('pageerror',error=>errors.push(error.message));
    await page.addInitScript(()=>localStorage.setItem('blacksite-res','320'));
    await page.goto(process.env.BLACKSITE_TEST_URL ?? 'http://127.0.0.1:8080/');
    await page.waitForFunction(()=>document.querySelector('.deploy-button')?.disabled===false || document.querySelector('[role=alert]'),null,{timeout:120000});
    assert.equal(await page.getByRole('alert').count(),0,await page.locator('body').innerText());
    const canvas=await page.locator('.game-canvas').elementHandle();
    await page.getByRole('button',{name:'Asset Catalog & Weapon Preview'}).click();
    await page.getByRole('button',{name:'BACK TO GAME'}).waitFor();
    assert.equal(await canvas.evaluate(element=>element.isConnected),true,'catalog must preserve the renderer canvas');
    await page.locator('body').press('Enter');
    await page.getByRole('button',{name:'BACK TO GAME'}).click();
    assert.equal(await canvas.evaluate(element=>element===document.querySelector('.game-canvas')),true);
    await page.locator('.deploy-button').click();
    await page.waitForFunction(()=>document.body.innerText.includes('HEALTH'));
    const before=await page.evaluate(()=>window.__controlsTest.getX());
    await page.keyboard.down('w');await page.waitForTimeout(350);await page.keyboard.up('w');
    assert.notEqual(await page.evaluate(()=>window.__controlsTest.getX()),before,'restored game accepts movement');
    await page.screenshot({path:'screenshots/catalog-return.png'});
    assert.deepEqual(errors,[]);
  } finally {await browser.close();}
});
