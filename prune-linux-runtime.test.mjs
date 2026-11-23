import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, mkdirSync, writeFileSync, existsSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { pruneLinuxRuntime } from './prune-linux-runtime.mjs';

test('Linux staging preserves glibc, static sandbox and matching PTY only', () => {
  const root = mkdtempSync(join(tmpdir(), 'dandy-prune-'));
  try {
    writeFileSync(join(root, 'package.json'), JSON.stringify({ name: 'dandy-engine-runtime' }));
    const files = [
      'node-pty/prebuilds/linux-x64/pty.node', 'node-pty/prebuilds/linux-arm64/pty.node',
      'node-pty/prebuilds/darwin-x64/pty.node',
      '@deepseek-ai/node-addon-system-linux-x64/bin/glibc/system.node',
      '@deepseek-ai/node-addon-system-linux-x64/bin/musl/system.node',
      '@deepseek-ai/node-addon-system-linux-x64/bin/landlock-run',
    ];
    for (const file of files) {
      const path = join(root, 'node_modules', file);
      mkdirSync(join(path, '..'), { recursive: true });
      writeFileSync(path, 'fixture');
    }
    assert.equal(pruneLinuxRuntime(root, 'x64').length, 3);
    for (const i of [0, 3, 5]) assert.ok(existsSync(join(root, 'node_modules', files[i])));
    for (const i of [1, 2, 4]) assert.equal(existsSync(join(root, 'node_modules', files[i])), false);
    assert.deepEqual(pruneLinuxRuntime(root, 'x64'), []);
    writeFileSync(join(root, 'package.json'), JSON.stringify({ name: 'user-project' }));
    assert.throws(() => pruneLinuxRuntime(root), /not an isolated/);
  } finally { rmSync(root, { recursive: true }); }
});
