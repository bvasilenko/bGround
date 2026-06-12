#!/usr/bin/env node
const { spawnSync } = require('node:child_process');
const path = require('node:path');
const os = require('node:os');
const fs = require('node:fs');
const candidates = [
  path.join(os.homedir(), '.cargo', 'bin', 'bground'),
  '/usr/local/bin/bground',
  'bground',
];
let exe = candidates.find((p) => p === 'bground' || (fs.existsSync(p) && fs.statSync(p).isFile())) || 'bground';
const result = spawnSync(exe, process.argv.slice(2), { stdio: 'inherit' });
process.exit(result.status ?? 1);
