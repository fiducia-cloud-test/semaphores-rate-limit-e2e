#!/usr/bin/env node
import assert from 'node:assert/strict';
import { readFileSync, readdirSync, statSync } from 'node:fs';
import { join, resolve } from 'node:path';

const root = resolve('fixtures/ores-cli-rate-limit');
const manifest = JSON.parse(readFileSync(join(root, 'expected.json'), 'utf8'));
const cases = new Map(manifest.cases.map((item) => [item.name, item]));
const expectedNames = [
  'redis-bound',
  'legacy-local',
  'ci-only',
  'production-reference-only',
];
assert.equal(manifest.schemaVersion, 1);
assert.equal(manifest.consumer, 'ORESoftware/ores-cli');
assert.equal(manifest.audit, 'runtime-config-consumption');
assert.deepEqual([...cases.keys()].sort(), [...expectedNames].sort());
for (const item of manifest.cases) {
  assert.ok(item.expectedClass.length > 0);
  assert.match(item.expectedCode, /^runtime-config-/);
  const config = readFileSync(join(root, item.name, '.ores-rl.toml'), 'utf8').toLowerCase();
  assert.match(config, /redis/);
  assert.ok(config.includes('strict') || config.includes('fail_closed'));
}

const loaderTokens = [
  'include_str!',
  'fs::read',
  'read_to_string',
  'toml::from_str',
  'load_repo_root',
  'load_optional',
  'admit_server_stack',
  'admit_runtime',
  'with_optional_sidecar_file',
];
const hasLoader = (text) => loaderTokens.some((token) => text.includes(token));

const bound = readFileSync(join(root, 'redis-bound/src/limiter.rs'), 'utf8');
assert.ok(bound.includes('.ores-rl.toml'));
assert.ok(hasLoader(bound));
assert.ok(bound.includes('REDIS_URL'));
assert.ok(bound.includes('ORES_RL_HMAC_KEY'));
assert.ok(bound.includes('ores_rl'));

const legacy = readFileSync(join(root, 'legacy-local/src/limiter.rs'), 'utf8');
assert.ok(legacy.includes('Mutex<HashMap'));
assert.ok(legacy.includes('ZED_RATE_LIMIT_'));
assert.ok(!legacy.includes('REDIS_URL'));
assert.ok(!legacy.includes('ORES_RL_HMAC_KEY'));
assert.ok(!hasLoader(legacy));

const ciRoot = join(root, 'ci-only');
assert.ok(!readdirSync(ciRoot).includes('src'));
const ciScript = readFileSync(join(ciRoot, 'scripts/validate-config.sh'), 'utf8');
assert.ok(ciScript.includes('.ores-rl.toml'));

const reference = readFileSync(join(root, 'production-reference-only/src/main.rs'), 'utf8');
assert.ok(reference.includes('.ores-rl.toml'));
assert.ok(!hasLoader(reference));

const forbidden = ['ghp_', 'github_pat_', 'lin_api_', 'BEGIN PRIVATE KEY'];
function walk(path) {
  for (const name of readdirSync(path)) {
    const child = join(path, name);
    if (statSync(child).isDirectory()) {
      walk(child);
      continue;
    }
    const text = readFileSync(child, 'utf8');
    for (const marker of forbidden) {
      assert.ok(!text.includes(marker), `credential marker ${marker} in ${child}`);
    }
  }
}
walk(root);
console.log(`validated ${manifest.cases.length} ores-cli rate-limit evidence fixtures`);
