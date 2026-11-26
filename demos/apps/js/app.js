// Working JavaScript app: recompile and print a structured report card.
const { Client } = require('../../../js');
const [cli, pe, project] = process.argv.slice(2);
if (!cli || !pe || !project) {
  console.error('usage: node app.js <cli> <pe> <project>');
  process.exit(2);
}
try {
  const client = new Client(cli);
  const report = client.recompile(pe, project);
  const stages = Array.isArray(report.stages) ? report.stages.length : 0;
  console.log(`APP_OK sdk=javascript score=${report.score ?? 'null'} stages=${stages} report=${client.recompileToPath(pe, project)}`);
} catch (e) { console.error(`error: ${e.message}`); process.exit(1); }
