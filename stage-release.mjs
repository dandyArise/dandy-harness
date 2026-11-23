import { mkdirSync, copyFileSync, chmodSync, writeFileSync } from 'node:fs';
import { resolve, join } from 'node:path';
const engine = resolve(process.argv[2]);
mkdirSync('runtime', { recursive: true });
const binary = join('runtime', process.platform === 'win32' ? 'node.exe' : 'node');
copyFileSync(process.execPath, binary);
if (process.platform !== 'win32') chmodSync(binary, 0o755);
const license = await fetch(`https://raw.githubusercontent.com/nodejs/node/${process.version}/LICENSE`);
if (!license.ok) throw new Error(`Node license HTTP ${license.status}`);
writeFileSync('runtime/LICENSE', await license.text());
const bundle = { resources: { [`${engine}/`]: 'engine/', 'runtime/': 'runtime/' },
  windows: { nsis: { compression: 'zlib' } },
  macOS: { signingIdentity: '-' } };
writeFileSync('target/release-resources.json', JSON.stringify({ bundle }, null, 2));
