'use strict';
const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const { Client, RecompileError } = require('../index.js');

function fixture(mode = 'ok') {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'gr-sdk-js-'));
  const pe = path.join(dir, 'game.exe');
  fs.writeFileSync(pe, 'MZ synthetic');
  const cli = path.join(dir, 'fake-cli');
  const body = mode === 'fail' ? 'exit 7\n' : mode === 'no-report' ? 'exit 0\n'
    : mode === 'bad-json' ? 'printf not-json\n'
    : `case "$1" in
  inspect) printf '{"machine":"i386"}\\n' ;;
  run)
    out="";
    while [ "$#" -gt 0 ]; do
      if [ "$1" = "--project-dir" ]; then out="$2"; fi
      shift;
    done
    mkdir -p "$out/reports"
    printf '{"schema":1,"score":42}\\n' > "$out/reports/report.json"
    ;;
esac
exit 0
`;
  fs.writeFileSync(cli, `#!/bin/sh\n${body}`);
  fs.chmodSync(cli, 0o755);
  return { dir, pe, cli, project: path.join(dir, 'out') };
}

test('missing CLI is refused', () => {
  assert.throws(() => new Client('/nonexistent/cli'), RecompileError);
});

test('missing PE is refused', () => {
  const { cli } = fixture();
  assert.throws(() => new Client(cli).inspect('/nonexistent/game.exe'), RecompileError);
});

test('inspect parses CLI JSON', () => {
  const { cli, pe } = fixture();
  assert.deepEqual(new Client(cli).inspect(pe), { machine: 'i386' });
});

test('recompile returns parsed report and path', () => {
  const { cli, pe, project } = fixture();
  const client = new Client(cli);
  const report = client.recompile(pe, project);
  assert.equal(report.score, 42);
  assert.equal(client.recompileToPath(pe, project), path.join(project, 'reports', 'report.json'));
});

test('non-zero CLI exit is an error', () => {
  const { cli, pe, project } = fixture('fail');
  assert.throws(() => new Client(cli).recompile(pe, project), (e) => e instanceof RecompileError && e.exitCode === 7);
});

test('missing report is an error', () => {
  const { cli, pe, project } = fixture('no-report');
  assert.throws(() => new Client(cli).recompile(pe, project), /no report\.json/);
});

test('invalid JSON is an error', () => {
  const { cli, pe } = fixture('bad-json');
  assert.throws(() => new Client(cli).inspect(pe), /invalid JSON/);
});
