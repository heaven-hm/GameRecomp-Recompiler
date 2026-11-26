const { Client } = require('../index.js');
const [cli, pe, project] = process.argv.slice(2);
if (!cli || !pe || !project) {
  console.error('usage: node js/examples/basic.js <cli> <pe32-x86.exe> <project-dir>');
  process.exit(2);
}
try {
  const client = new Client(cli);
  const report = client.recompile(pe, project);
  console.log(JSON.stringify({ score: report.score ?? null, report: client.recompileToPath(pe, project) }, null, 2));
} catch (e) {
  console.error(`${e.name}: ${e.message}`);
  process.exit(1);
}
