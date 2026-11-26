/*
 * app.js - JavaScript example for the GameRecomp SDK.
 *
 * Usage: node app.js <game-recomp-cli> <pe32-x86.exe> <existing-project-dir>
 *
 * The CLI path is for this harness only; production callers use `new Client()`
 * and pass only the game path and build options.
 */

'use strict';

const fs = require('node:fs');
const { Client } = require('../../../js');

function buildOptions() {
  return {
    input_os: 'windows',
    input_arch: 'x86',
    target_os: 'host',
    target_arch: 'host',
    out_mode: 'full',
  };
}

function main(cliPath, pePath, projectDir) {
  const client = new Client(cliPath);
  const reportPath = client.buildToPath(pePath, projectDir, buildOptions());
  const report = JSON.parse(fs.readFileSync(reportPath, 'utf8'));
  const stages = Array.isArray(report.stages) ? report.stages.length : 0;

  console.log(
    `APP_OK sdk=javascript score=${report.score ?? 'null'}` +
    ` stages=${stages} report=${reportPath}`
  );
  return 0;
}

if (process.argv.length !== 5) {
  console.error('usage: node app.js <cli> <pe> <project>');
  process.exit(2);
}

try {
  process.exit(main(...process.argv.slice(2)));
} catch (error) {
  console.error(`error: ${error.message}`);
  process.exit(1);
}
