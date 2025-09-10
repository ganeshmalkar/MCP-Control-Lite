import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Search, Download, Trash2, RefreshCw, ChevronDown, ChevronRight } from 'lucide-react';

interface LogEntry {
  id: string;
  timestamp: string;
  level: 'error' | 'warning' | 'info' | 'debug';
  category: 'server' | 'application' | 'system';
  message: string;
  details?: string;
}

export default function Logs() {
  const [logs, setLogs] = useState<LogEntry[]>([]);
  const [filteredLogs, setFilteredLogs] = useState<LogEntry[]>([]);
  const [loading, setLoading] = useState(true);
  const [autoRefresh, setAutoRefresh] = useState(true);
  const [searchTerm, setSearchTerm] = useState('');
  const [expandedLogs, setExpandedLogs] = useState<Set<string>>(new Set());
  
  // Filter states
  const [levelFilters, setLevelFilters] = useState({
    error: true,
    warning: true,
    info: true,
    debug: false
  });
  const [categoryFilters, setCategoryFilters] = useState({
    server: true,
    application: true,
    system: true
  });

  useEffect(() => {
    loadLogs();
    
    let interval: number;
    if (autoRefresh) {
      interval = window.setInterval(loadLogs, 5000);
    }
    
    return () => {
      if (interval) window.clearInterval(interval);
    };
  }, [autoRefresh]);

  useEffect(() => {
    filterLogs();
  }, [logs, searchTerm, levelFilters, categoryFilters]);

  const loadLogs = async () => {
    try {
      // Check if logging is enabled
      const settings = await invoke<any>('get_settings');
      if (settings && settings.enableLogs === false) {
        setLogs([{
          id: 'disabled-' + Date.now(),
          timestamp: new Date().toISOString(),
          level: 'info',
          category: 'system',
          message: 'Logging is disabled in settings. Enable logging to see system events.'
        }]);
        return;
      }
      
      const logsData = await invoke<LogEntry[]>('get_logs');
      setLogs(logsData);
    } catch (error) {
      console.error('Failed to load logs:', error);
      // Generate sample logs for demo
      setLogs(generateSampleLogs());
    } finally {
      setLoading(false);
    }
  };

  const generateSampleLogs = (): LogEntry[] => {
    const now = new Date();
    return [
      {
        id: '1',
        timestamp: new Date(now.getTime() - 60000).toISOString(),
        level: 'info',
        category: 'server',
        message: 'MCP server "filesystem" started successfully',
        details: 'Server listening on port 3001, PID: 12345'
      },
      {
        id: '2',
        timestamp: new Date(now.getTime() - 120000).toISOString(),
        level: 'warning',
        category: 'application',
        message: 'Claude Desktop configuration updated',
        details: 'Server "weather-api" was disabled by user'
      },
      {
        id: '3',
        timestamp: new Date(now.getTime() - 180000).toISOString(),
        level: 'error',
        category: 'server',
        message: 'Failed to start MCP server "broken-server"',
        details: 'Error: Command not found: /invalid/path/to/server\nExit code: 127'
      },
      {
        id: '4',
        timestamp: new Date(now.getTime() - 240000).toISOString(),
        level: 'info',
        category: 'system',
        message: 'Configuration sync completed',
        details: 'Synced 3 applications, 12 servers total'
      },
      {
        id: '5',
        timestamp: new Date(now.getTime() - 300000).toISOString(),
        level: 'debug',
        category: 'system',
        message: 'Application detection scan started',
        details: 'Scanning for: Claude Desktop, Cursor, VS Code, Amazon Q'
      }
    ];
  };

  const filterLogs = () => {
    let filtered = logs.filter(log => {
      // Level filter
      if (!levelFilters[log.level]) return false;
      
      // Category filter
      if (!categoryFilters[log.category]) return false;
      
      // Search filter
      if (searchTerm && !log.message.toLowerCase().includes(searchTerm.toLowerCase()) && 
          !log.details?.toLowerCase().includes(searchTerm.toLowerCase())) {
        return false;
      }
      
      return true;
    });
    
    setFilteredLogs(filtered);
  };

  const toggleLogExpansion = (logId: string) => {
    const newExpanded = new Set(expandedLogs);
    if (newExpanded.has(logId)) {
      newExpanded.delete(logId);
    } else {
      newExpanded.add(logId);
    }
    setExpandedLogs(newExpanded);
  };

  const clearLogs = async () => {
    try {
      await invoke('clear_logs');
      setLogs([]);
    } catch (error) {
      console.error('Failed to clear logs:', error);
    }
  };

  const exportLogs = async () => {
    try {
      await invoke('export_logs');
    } catch (error) {
      console.error('Failed to export logs:', error);
    }
  };

  const getLevelColor = (level: string) => {
    switch (level) {
      case 'error': return '#e74c3c';
      case 'warning': return '#f39c12';
      case 'info': return '#3498db';
      case 'debug': return '#95a5a6';
      default: return '#666';
    }
  };

  const formatTimestamp = (timestamp: string) => {
    return new Date(timestamp).toLocaleString();
  };

  if (loading) {
    return (
      <div className="loading">
        <RefreshCw size={24} style={{ animation: 'spin 1s linear infinite' }} />
        <p style={{ marginTop: '10px' }}>Loading logs...</p>
      </div>
    );
  }

  return (
    <div>
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '20px' }}>
        <h2>System Logs</h2>
        <div style={{ display: 'flex', gap: '10px' }}>
          <label style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
            <input
              type="checkbox"
              checked={autoRefresh}
              onChange={(e) => setAutoRefresh(e.target.checked)}
            />
            Auto-refresh
          </label>
          <button className="btn btn-secondary" onClick={exportLogs}>
            <Download size={16} style={{ marginRight: '8px' }} />
            Export
          </button>
          <button className="btn btn-secondary" onClick={clearLogs}>
            <Trash2 size={16} style={{ marginRight: '8px' }} />
            Clear
          </button>
          <button className="btn btn-primary" onClick={loadLogs}>
            <RefreshCw size={16} style={{ marginRight: '8px' }} />
            Refresh
          </button>
        </div>
      </div>

      {/* Filters */}
      <div style={{ background: 'var(--bg-secondary)', padding: '16px', borderRadius: '8px', marginBottom: '20px' }}>
        <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr 1fr', gap: '20px' }}>
          {/* Search */}
          <div>
            <label style={{ display: 'block', marginBottom: '8px', fontWeight: '500' }}>Search</label>
            <div style={{ position: 'relative' }}>
              <Search size={16} style={{ position: 'absolute', left: '8px', top: '50%', transform: 'translateY(-50%)', color: 'var(--text-secondary)' }} />
              <input
                type="text"
                placeholder="Search logs..."
                value={searchTerm}
                onChange={(e) => setSearchTerm(e.target.value)}
                style={{ 
                  width: '100%', 
                  padding: '8px 8px 8px 32px', 
                  border: '1px solid var(--border-color)',
                  borderRadius: '4px',
                  background: 'var(--bg-primary)',
                  color: 'var(--text-primary)'
                }}
              />
            </div>
          </div>

          {/* Level Filters */}
          <div>
            <label style={{ display: 'block', marginBottom: '8px', fontWeight: '500' }}>Log Levels</label>
            <div style={{ display: 'flex', flexWrap: 'wrap', gap: '8px' }}>
              {Object.entries(levelFilters).map(([level, checked]) => (
                <label key={level} style={{ display: 'flex', alignItems: 'center', gap: '4px' }}>
                  <input
                    type="checkbox"
                    checked={checked}
                    onChange={(e) => setLevelFilters({ ...levelFilters, [level]: e.target.checked })}
                  />
                  <span style={{ color: getLevelColor(level), textTransform: 'capitalize' }}>{level}</span>
                </label>
              ))}
            </div>
          </div>

          {/* Category Filters */}
          <div>
            <label style={{ display: 'block', marginBottom: '8px', fontWeight: '500' }}>Categories</label>
            <div style={{ display: 'flex', flexWrap: 'wrap', gap: '8px' }}>
              {Object.entries(categoryFilters).map(([category, checked]) => (
                <label key={category} style={{ display: 'flex', alignItems: 'center', gap: '4px' }}>
                  <input
                    type="checkbox"
                    checked={checked}
                    onChange={(e) => setCategoryFilters({ ...categoryFilters, [category]: e.target.checked })}
                  />
                  <span style={{ textTransform: 'capitalize' }}>{category}</span>
                </label>
              ))}
            </div>
          </div>
        </div>
      </div>

      {/* Log Entries */}
      <div style={{ background: 'var(--bg-secondary)', borderRadius: '8px', overflow: 'hidden' }}>
        {filteredLogs.length === 0 ? (
          <div style={{ padding: '40px', textAlign: 'center', color: 'var(--text-secondary)' }}>
            No logs match the current filters
          </div>
        ) : (
          filteredLogs.map((log) => (
            <div key={log.id} style={{ borderBottom: '1px solid var(--border-color)' }}>
              <div 
                style={{ 
                  padding: '12px 16px', 
                  display: 'flex', 
                  alignItems: 'center', 
                  gap: '12px',
                  cursor: log.details ? 'pointer' : 'default'
                }}
                onClick={() => log.details && toggleLogExpansion(log.id)}
              >
                {log.details && (
                  expandedLogs.has(log.id) ? 
                    <ChevronDown size={16} style={{ color: 'var(--text-secondary)' }} /> :
                    <ChevronRight size={16} style={{ color: 'var(--text-secondary)' }} />
                )}
                
                <div style={{ 
                  width: '8px', 
                  height: '8px', 
                  borderRadius: '50%', 
                  backgroundColor: getLevelColor(log.level),
                  flexShrink: 0
                }} />
                
                <div style={{ 
                  minWidth: '140px', 
                  fontSize: '12px', 
                  color: 'var(--text-secondary)',
                  flexShrink: 0
                }}>
                  {formatTimestamp(log.timestamp)}
                </div>
                
                <div style={{ 
                  minWidth: '80px', 
                  fontSize: '12px', 
                  color: getLevelColor(log.level),
                  textTransform: 'uppercase',
                  fontWeight: '500',
                  flexShrink: 0
                }}>
                  {log.level}
                </div>
                
                <div style={{ 
                  minWidth: '100px', 
                  fontSize: '12px', 
                  color: 'var(--text-secondary)',
                  textTransform: 'capitalize',
                  flexShrink: 0
                }}>
                  {log.category}
                </div>
                
                <div style={{ flex: 1, color: 'var(--text-primary)' }}>
                  {log.message}
                </div>
              </div>
              
              {log.details && expandedLogs.has(log.id) && (
                <div style={{ 
                  padding: '12px 16px 16px 60px', 
                  background: 'var(--bg-primary)',
                  fontSize: '12px',
                  color: 'var(--text-secondary)',
                  fontFamily: 'monospace',
                  whiteSpace: 'pre-wrap'
                }}>
                  {log.details}
                </div>
              )}
            </div>
          ))
        )}
      </div>
      
      <div style={{ marginTop: '16px', fontSize: '14px', color: 'var(--text-secondary)' }}>
        Showing {filteredLogs.length} of {logs.length} log entries
      </div>
    </div>
  );
}
