// Install local packed packages in an isolated, relocatable production tree.
import { readdirSync, mkdirSync, readFileSync, writeFileSync, copyFileSync, existsSync } from 'node:fs';
import { resolve, join, basename } from 'node:path';
import { execFileSync } from 'node:child_process';
import { pathToFileURL } from 'node:url';
const [sourceArg, outputArg] = process.argv.slice(2);
if (!sourceArg || !outputArg) throw new Error('Usage: node prepare-engine.mjs SOURCE OUTPUT');
const source = resolve(sourceArg), output = resolve(outputArg);
if (existsSync(join(output, 'package.json'))) throw new Error('Use a fresh staging directory');
const packages = new Map();
const supports = (values, target) => !values || (!values.includes(`!${target}`) &&
  (values.includes(target) || values.includes('any') || values.every(x => x.startsWith('!'))));
for (const dir of ['packed-vendor', 'packed-dsh']) {
  for (const file of readdirSync(join(import.meta.dirname, 'target', dir)).filter(x => x.endsWith('.tgz'))) {
    const path = join(import.meta.dirname, 'target', dir, file);
    const manifest = JSON.parse(execFileSync('tar', ['-xOf', path, 'package/package.json'], {encoding:'utf8'}));
    if (!supports(manifest.os, process.platform) || !supports(manifest.cpu, process.arch)) continue;
    packages.set(manifest.name, {manifest, path});
  }
}
const selected = new Map();
function visit(name) {
  if (selected.has(name)) return;
  const item = packages.get(name);
  if (!item) throw new Error(`Missing local package ${name}`);
  selected.set(name, item);
  for (const dep of Object.keys({...item.manifest.dependencies, ...item.manifest.peerDependencies, ...item.manifest.optionalDependencies})) {
    if (packages.has(dep)) visit(dep);
  }
}
visit('@deepseek-ai/dsh');
mkdirSync(join(output, 'packages'), {recursive:true});
const deps = {};
for (const [name, item] of selected) {
  const file = basename(item.path);
  copyFileSync(item.path, join(output, 'packages', file));
  deps[name] = `file:packages/${file}`;
}
writeFileSync(join(output, 'package.json'), JSON.stringify({name:'dandy-engine-runtime',private:true,type:'module',dependencies:deps},null,2));
// Preserve the repository's reviewed dependency patches; no user configuration is copied.
const yaml = await import(pathToFileURL(join(source, 'node_modules/js-yaml/index.js')).href);
const workspace = yaml.load(readFileSync(join(source, 'pnpm-workspace.yaml'),'utf8'));
const patches = {};
for (const [name, file] of Object.entries(workspace.patchedDependencies ?? {})) {
  mkdirSync(join(output, 'patches'), {recursive:true});
  copyFileSync(join(source,file),join(output,'patches',basename(file)));
  patches[name] = `patches/${basename(file)}`;
}
writeFileSync(join(output,'pnpm-workspace.yaml'), yaml.dump({nodeLinker:'hoisted',autoInstallPeers:true,overrides:deps,patchedDependencies:patches,allowUnusedPatches:true,
  allowBuilds: {'node-pty':true,koffi:true,[`@deepseek-ai/dsh-subprocess-local@${deps['@deepseek-ai/dsh-subprocess-local']}`]:true,'@google/genai':false,protobufjs:false,'node-addon-require-builtin':false, 'fs-ext':true}}));
console.log(`Prepared ${selected.size} local packages in ${output}`);
