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
export declare class Client {
  readonly cli: string;
  constructor(cli?: string | null);
  inspect(pe: string): Record<string, unknown>;
  recompileToPath(pe: string, project: string): string;
  recompile(pe: string, project: string): Report;
}
