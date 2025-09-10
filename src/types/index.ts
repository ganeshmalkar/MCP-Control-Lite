export interface MCPServer {
  name: string;
  enabled: boolean;
  command?: string;
  args?: string[];
  env?: Record<string, string>;
  application: string;
}

export interface Application {
  name: string;
  detected: boolean;
  configPath?: string;
  serverCount: number;
  lastSync?: string;
  syncStatus?: 'synced' | 'pending' | 'conflict' | 'error';
}

export interface ServerStatus {
  name: string;
  status: 'running' | 'stopped' | 'error' | 'unknown';
  pid?: number;
}

export type ViewType = 'servers' | 'applications' | 'discover' | 'settings' | 'logs';
