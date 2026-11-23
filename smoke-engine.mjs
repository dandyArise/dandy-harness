import { spawn } from 'node:child_process';
import { mkdtempSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
const root = resolve(process.argv[2]);
const home = mkdtempSync(join(tmpdir(), 'dandy-isolated-'));
const child = spawn(process.execPath, ['node_modules/@deepseek-ai/dsh/lib/bin.js','web','--no-open','--host','127.0.0.1','--port','18644'],
  {cwd:root,windowsHide:true,env:{...process.env,DSH_HOME:home}});
let log = '';
child.stdout.on('data', x => log += x);
child.stderr.on('data', x => log += x);
try {
  let ready = false;
  for (let i=0;i<60;i++) {
    await new Promise(r=>setTimeout(r,500));
    if(child.exitCode !== null) throw new Error(`Engine exited with ${child.exitCode}`);
    try {
      const url = log.match(/http:\/\/127\.0\.0\.1:18644\/\?token=[^\s\u001b]+/)?.[0];
      if (!url) continue;
      let r = await fetch(url,{redirect:'manual',signal:AbortSignal.timeout(1000)});
      if (r.status === 302 || r.status === 303) {
        const cookie = r.headers.getSetCookie().map(value => value.split(';')[0]).join('; ');
        r = await fetch('http://127.0.0.1:18644/',{headers:{cookie},signal:AbortSignal.timeout(1000)});
      }
      if (i === 0 || i === 10) console.log(`Readiness HTTP ${r.status}`);
      if(r.ok && (await r.text()).includes('<html')) { ready=true; break; }
    } catch {}
  }
  if(!ready) throw new Error('Engine did not serve HTML within 30 seconds');
  console.log('Isolated engine: HTTP 200, HTML served');
} finally {
  writeFileSync(join(home,'smoke.log'),log);
  if(child.exitCode === null) {
    const exited = new Promise(resolve=>child.once('exit',resolve));
    child.kill();
    await exited;
  }
  console.log(`Own process stopped; diagnostics in ${home}`);
}
