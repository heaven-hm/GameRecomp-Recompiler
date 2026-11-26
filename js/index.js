'use strict';
const fs = require('node:fs');
const path = require('node:path');
const { spawnSync } = require('node:child_process');

class RecompileError extends Error {
  constructor(message, details = {}) {
    super(message);
    this.name = 'RecompileError';
    Object.assign(this, details);
  }
}

function ensureFile(value, what) {
  if (!value || typeof value !== 'string') {
    throw new RecompileError(`${what} must be a non-empty path`, { what });
  }
  let st;
  try { st = fs.statSync(value); } catch (e) {
    throw new RecompileError(`${what} does not exist: ${value}`, { what, path: value, cause: e.message });
  }
  if (!st.isFile()) {
    throw new RecompileError(`${what} is not a regular file: ${value}`, { what, path: value });
  }
}

class Client {
  constructor(cli) {
    const resolved = cli || process.env.GAMERECOMP_CLI || 'game-recomp-cli';
    ensureFile(resolved, 'CLI');
    this.cli = path.resolve(resolved);
  }

  _run(args) {
    const proc = spawnSync(this.cli, args, { encoding: 'utf8', shell: false });
    if (proc.error) {
      throw new RecompileError('failed to spawn CLI', { cli: this.cli, cause: proc.error.message });
    }
    if (proc.status !== 0) {
      throw new RecompileError(`CLI exited ${proc.status}`, {
        cli: this.cli, exitCode: proc.status, output: String(proc.stdout || '') + String(proc.stderr || '')
      });
    }
    return String(proc.stdout || '');
  }

  inspect(pe) {
    ensureFile(pe, 'PE32 input');
    const out = this._run(['inspect', path.resolve(pe), '--json']);
    try { return JSON.parse(out); }
    catch (e) { throw new RecompileError('CLI produced invalid JSON', { cli: this.cli, cause: e.message }); }
  }

  buildToPath(pe, project, options = {}) {
    ensureFile(pe, 'PE32 input');
    if (!project || typeof project !== 'string') {
      throw new RecompileError('project directory must be a non-empty path', { what: 'project' });
    }
    const dir = path.resolve(project);
    const args = ['run', path.resolve(pe), '--project-dir', dir, '--no-table', '--no-progress'];
    const flags = [['--input-os', options.input_os], ['--input-arch', options.input_arch],
      ['--target-os', options.target_os], ['--target-arch', options.target_arch],
      ['--out-mode', options.out_mode], ['--skip-stages', options.skip_stages]];
    for (const [flag, value] of flags) if (value) args.push(flag, String(value));
    this._run(args);
    const report = path.join(dir, 'reports', 'report.json');
    if (!fs.existsSync(report)) {
      throw new RecompileError('pipeline produced no report.json', { cli: this.cli, report });
    }
    return report;
  }

  recompileToPath(pe, project) {
    return this.buildToPath(pe, project);
  }

  build(pe, project, options = {}) {
    return JSON.parse(require('node:fs').readFileSync(this.buildToPath(pe, project, options), 'utf8'));
  }

  recompile(pe, project) {
    const report = this.recompileToPath(pe, project);
    try { return JSON.parse(fs.readFileSync(report, 'utf8')); }
    catch (e) { throw new RecompileError('report.json is not valid JSON', { report, cause: e.message }); }
  }
}

module.exports = { Client, RecompileError };
