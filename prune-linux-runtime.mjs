import { existsSync, readFileSync, readdirSync, realpathSync, rmSync } from 'node:fs';
import { join, relative, isAbsolute, resolve } from 'node:path';
import { pathToFileURL } from 'node:url';

// The Linux installers use the official glibc Node runtime. Other prebuilds
// cannot run there and confuse linuxdeploy's recursive ELF dependency scan.
export function pruneLinuxRuntime(directory, arch = process.arch) {
  const root = realpathSync(directory);
  if (JSON.parse(readFileSync(join(root, 'package.json'), 'utf8')).name !== 'dandy-engine-runtime') {
    throw new Error('Refusing to prune a directory that is not an isolated release runtime');
  }
  if (!['x64', 'arm64'].includes(arch)) throw new Error(`Unsupported architecture: ${arch}`);
  const removals = [];
  for (const name of ['node-pty/prebuilds', '@deepseek-ai/libreoffice-kit/frame-forwarding/prebuilds']) {
    const dir = join(root, 'node_modules', name);
    if (!existsSync(dir)) continue;
    for (const entry of readdirSync(dir, { withFileTypes: true })) {
      if (entry.name !== `linux-${arch}` && entry.isDirectory()) removals.push(join(dir, entry.name));
    }
  }
  const musl = join(root, 'node_modules', `@deepseek-ai/node-addon-system-linux-${arch}`, 'bin/musl');
  if (existsSync(musl)) removals.push(musl);
  // Validate all resolved targets before removing anything, including junctions.
  for (const path of removals) {
    const rel = relative(root, realpathSync(path));
    if (!rel || rel.startsWith('..') || isAbsolute(rel)) throw new Error('Runtime path escapes staging');
  }
  for (const path of removals) rmSync(path, { recursive: true });
  return removals.map(path => relative(root, path));
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) {
  if (process.platform !== 'linux') throw new Error('Linux release staging only');
  if (!process.argv[2]) throw new Error('Usage: node prune-linux-runtime.mjs RUNTIME');
  console.log('Removed incompatible release variants:', pruneLinuxRuntime(process.argv[2]));
}
