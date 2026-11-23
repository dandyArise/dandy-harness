// Test the runtime extracted from the distributable, not the source checkout.
import { readdirSync, existsSync, mkdirSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { execFileSync } from 'node:child_process';
const target = process.argv[2];
const bundle = resolve('target', target, 'release/bundle');
const find = (dir, suffix) => readdirSync(dir).find(name => name.endsWith(suffix));
let resources;
if (process.platform === 'win32') {
  const exe = join(bundle, 'nsis', find(join(bundle, 'nsis'), '.exe'));
  execFileSync(exe, ['/S'], { stdio: 'inherit' });
  resources = join(process.env.LOCALAPPDATA, 'Dandy Harness');
} else if (process.platform === 'darwin') {
  const app = join(bundle, 'macos', find(join(bundle, 'macos'), '.app'));
  execFileSync('codesign', ['--verify', '--deep', '--strict', app], { stdio: 'inherit' });
  resources = join(app, 'Contents/Resources');
} else {
  const destination = resolve('target/installed-deb');
  mkdirSync(destination, { recursive: true });
  execFileSync('dpkg-deb', ['--extract', join(bundle, 'deb', find(join(bundle, 'deb'), '.deb')), destination]);
  const walk = (dir, depth = 0) => {
    if (existsSync(join(dir, 'engine/node_modules/@deepseek-ai/dsh/lib/bin.js'))) return dir;
    if (depth > 5) return;
    for (const entry of readdirSync(dir, { withFileTypes: true })) {
      if (entry.isDirectory()) { const result = walk(join(dir, entry.name), depth + 1); if (result) return result; }
    }
  };
  resources = walk(destination);
}
if (!resources) throw new Error('Installed resources not found');
const node = join(resources, 'runtime', process.platform === 'win32' ? 'node.exe' : 'node');
execFileSync(node, [resolve('smoke-engine.mjs'), join(resources, 'engine')], { stdio: 'inherit', timeout: 90000 });
console.log(`Installed ${process.platform}/${process.arch} runtime verified`);
