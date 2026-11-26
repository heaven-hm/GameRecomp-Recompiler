# JavaScript / TypeScript SDK

The `js/` package is a dependency-free CommonJS module with TypeScript
declarations (`index.d.ts`). It uses `node:child_process` with `shell: false`
and the same fail-closed contract as every other binding.

```js
const { Client, RecompileError } = require('./js');
const client = new Client(); // or GAMERECOMP_CLI
const report = client.recompile('game.exe', 'out');
console.log(report.score);
```

TypeScript consumers get typed `Client`, `Report` and `RecompileError` from
`index.d.ts`. `inspect()` runs `<cli> inspect <pe> --json`;
`recompileToPath()` returns `<project>/reports/report.json`; `recompile()`
parses that report. Missing CLI, missing PE, failed spawn, non-zero exit,
invalid JSON and missing report all throw `RecompileError`.

```sh
cd js && npm test                                  # 7 node:test cases
node js/examples/basic.js <cli> <pe> <project-dir>
```

The full five-binding demo and its screenshot proof are in
[`demos/`](../demos/README.md) and
[`proof/screenshots/`](../proof/screenshots/).
