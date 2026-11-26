/* Port one PE32 app through the JS SDK with explicit input/target options. */
'use strict';
const { Client } = require('../../js');
const [direction, pe, project] = process.argv.slice(2);
if (!direction || !pe || !project) {
  console.error('usage: node app.js <direction> <pe> <project>'); process.exit(2);
}
try {
  const report = new Client().build(pe, project, {
    input_os: 'windows', input_arch: 'x86', target_os: 'host', target_arch: 'host',
    out_mode: 'full', skip_stages: 'verify'
  });
  console.log(`PORT_OK direction=${direction} score=${report.score ?? 'null'}`);
} catch (error) {
  console.log(`PORT_DEFERRED direction=${direction} reason=${error.message}`);
}
