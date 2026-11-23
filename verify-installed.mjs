// Test the runtime extracted from the distributable, not the source checkout.
import { readdirSync, existsSync, mkdirSync, mkdtempSync, rmdirSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import { execFileSync } from 'node:child_process';
const target = process.argv[2];
const bundle = resolve('target', target, 'release/bundle');
const find = (dir, suffix) => {
  const name = readdirSync(dir).find(name => name.endsWith(suffix));
  if (!name) throw new Error(`Missing ${suffix} in ${dir}`);
  return name;
};
const verifyRuntime = resources => {
  if (!resources) throw new Error('Installed resources not found');
  const node = join(resources, 'runtime', process.platform === 'win32' ? 'node.exe' : 'node');
  execFileSync(node, [resolve('smoke-engine.mjs'), join(resources, 'engine')], { stdio: 'inherit', timeout: 90000 });
};
const walk = (dir, depth = 0) => {
  if (existsSync(join(dir, 'engine/node_modules/@deepseek-ai/dsh/lib/bin.js'))) return dir;
  if (depth > 5) return;
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    if (entry.isDirectory()) { const result = walk(join(dir, entry.name), depth + 1); if (result) return result; }
  }
};
let resources;
if (process.platform === 'win32') {
  const exe = join(bundle, 'nsis', find(join(bundle, 'nsis'), '.exe'));
  execFileSync(exe, ['/S'], { stdio: 'inherit' });
  resources = join(process.env.LOCALAPPDATA, 'Dandy Harness');
} else if (process.platform === 'darwin') {
  const mount = mkdtempSync(join(tmpdir(), 'dandy-dmg-'));
  const dmg = join(bundle, 'dmg', find(join(bundle, 'dmg'), '.dmg'));
  let mounted = false;
  try {
    execFileSync('hdiutil', ['attach', '-readonly', '-nobrowse', '-mountpoint', mount, dmg], { stdio: 'inherit', timeout: 120000 });
    mounted = true;
    const app = join(mount, find(mount, '.app'));
    execFileSync('codesign', ['--verify', '--deep', '--strict', app], { stdio: 'inherit' });
    verifyRuntime(join(app, 'Contents/Resources'));
  } finally {
    if (mounted) execFileSync('hdiutil', ['detach', mount], { stdio: 'inherit', timeout: 120000 });
    rmdirSync(mount);
  }
} else {
  const destination = resolve('target/installed-deb');
  mkdirSync(destination, { recursive: true });
  execFileSync('dpkg-deb', ['--extract', join(bundle, 'deb', find(join(bundle, 'deb'), '.deb')), destination]);
  resources = walk(destination);
  const imageDir = mkdtempSync(resolve('target/installed-appimage-'));
  const image = join(bundle, 'appimage', find(join(bundle, 'appimage'), '.AppImage'));
  execFileSync(image, ['--appimage-extract'], { cwd: imageDir, stdio: 'ignore', timeout: 120000 });
  verifyRuntime(walk(join(imageDir, 'squashfs-root')));
}
if (process.platform !== 'darwin') verifyRuntime(resources);
console.log(`Installed ${process.platform}/${process.arch} runtime verified`);
