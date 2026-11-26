export interface Report {
  schema?: number;
  score?: number;
  [key: string]: unknown;
}
export declare class RecompileError extends Error {
  readonly cli?: string;
  readonly exitCode?: number;
  readonly report?: string;
  readonly output?: string;
  readonly cause?: string;
  constructor(message: string, details?: Record<string, unknown>);
}
export interface BuildOptions {
  input_os?: string;
  input_arch?: string;
  target_os?: string;
  target_arch?: string;
  out_mode?: string;
  skip_stages?: string;
}
export declare class Client {
  readonly cli: string;
  constructor(cli?: string | null);
  inspect(pe: string): Record<string, unknown>;
  recompileToPath(pe: string, project: string): string;
  buildToPath(pe: string, project: string, options?: BuildOptions): string;
  build(pe: string, project: string, options?: BuildOptions): Report;
  recompile(pe: string, project: string): Report;
}
