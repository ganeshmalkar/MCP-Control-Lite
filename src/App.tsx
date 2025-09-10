import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { 
  Server, 
  Settings as SettingsIcon, 
  Monitor, 
  FileText,
  RefreshCw,
  Edit,
  Search
} from 'lucide-react';
import { Application, ViewType } from './types';
import Settings from './components/Settings';
import Logs from './components/Logs';
import ServerDetail from './components/ServerDetail';
import Discover from './components/Discover';

function App() {
  const [currentView, setCurrentView] = useState<ViewType>('servers');
  const [consolidatedServers, setConsolidatedServers] = useState<any[]>([]);
  const [applications, setApplications] = useState<Application[]>([]);
  const [loading, setLoading] = useState(true);
  const [systemStatus, setSystemStatus] = useState<any>(null);
  const [error, setError] = useState<string | null>(null);
  const [selectedServer, setSelectedServer] = useState<{
    id: string;
    name: string;
    application: string;
  } | null>(null);

  useEffect(() => {
    loadData();
    initializeTheme();
    
    // Listen for tray events
    const setupListeners = async () => {
      const unlisten1 = await listen('refresh-data', () => {
        loadData();
      });
      
      const unlisten2 = await listen('navigate-to', (event: any) => {
        setCurrentView(event.payload);
      });
      
      const unlisten3 = await listen('toggle-all-servers', async () => {
        // Toggle all servers in the first consolidated server (demo)
        if (consolidatedServers.length > 0) {
          const server = consolidatedServers[0];
          const allEnabled = server.applications.every((app: any) => app.enabled);
          await handleToggleAllApps(server.name, !allEnabled);
        }
      });
      
      return [unlisten1, unlisten2, unlisten3];
    };
    
    let unlisteners: any[] = [];
    setupListeners().then(listeners => {
      unlisteners = listeners;
    });
    
    const interval = setInterval(loadData, 10000); // Refresh every 10 seconds
    
    return () => {
      clearInterval(interval);
      unlisteners.forEach(unlisten => unlisten());
    };
  }, [consolidatedServers]);

  const initializeTheme = async () => {
    try {
      const savedSettings = await invoke<any>('get_settings');
      applyTheme(savedSettings.theme || 'system');
    } catch (error) {
      // Default to system theme if settings can't be loaded
      applyTheme('system');
    }
  };

  const applyTheme = (theme: string) => {
    const root = document.documentElement;
    if (theme === 'dark') {
      root.classList.add('dark-theme');
      root.classList.remove('light-theme');
    } else if (theme === 'light') {
      root.classList.add('light-theme');
      root.classList.remove('dark-theme');
    } else {
      // System theme
      root.classList.remove('dark-theme', 'light-theme');
      const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
      if (prefersDark) {
        root.classList.add('dark-theme');
      } else {
        root.classList.add('light-theme');
      }
    }
  };

  const loadData = async () => {
    try {
      setError(null);
      const [serversData, appsData, statusData, settingsData] = await Promise.all([
        invoke<any[]>('get_servers'),
        invoke<any[]>('get_applications'),
        invoke<any>('get_system_status'),
        invoke<any>('get_settings')
      ]);
      
      // Sort servers alphabetically by name, then by application for consistency
      const sortedServers = serversData.sort((a, b) => {
        const nameCompare = a.name.localeCompare(b.name);
        if (nameCompare !== 0) return nameCompare;
        return a.application.localeCompare(b.application);
      });

      // Get enabled apps from settings
      const enabledAppsSettings = settingsData.enabledApps || {};

      // Filter servers by enabled applications
      const filteredServers = sortedServers.filter(server => 
        enabledAppsSettings[server.application] !== false
      );

      // Consolidate servers by name
      const consolidated = filteredServers.reduce((acc: any[], server: any) => {
        const existing = acc.find(s => s.name === server.name);
        if (existing) {
          existing.applications.push({
            name: server.application,
            enabled: server.enabled,
            command: server.command,
            args: server.args
          });
        } else {
          acc.push({
            name: server.name,
            applications: [{
              name: server.application,
              enabled: server.enabled,
              command: server.command,
              args: server.args
            }]
          });
        }
        return acc;
      }, []);
      
      setConsolidatedServers(consolidated);
      setApplications(appsData.sort((a, b) => a.name.localeCompare(b.name)));
      setSystemStatus(statusData);
    } catch (error) {
      console.error('Failed to load data:', error);
      setError(`Failed to load data: ${error}`);
    } finally {
      setLoading(false);
    }
  };

  const handleToggleServer = async (serverName: string, application: string, enabled: boolean) => {
    try {
      await invoke('toggle_server', { 
        serverName, 
        application, 
        enabled: !enabled 
      });
      await loadData(); // Refresh data
    } catch (error) {
      console.error('Failed to toggle server:', error);
      setError(`Failed to toggle server: ${error}`);
    }
  };

  const renderSidebar = () => (
    <div className="sidebar">
      <div className="sidebar-header">
        <h1>MCP Control</h1>
      </div>
      
      <div 
        className={`nav-item ${currentView === 'servers' ? 'active' : ''}`}
        onClick={() => setCurrentView('servers')}
      >
        <Server size={18} />
        Servers
      </div>
      
      <div 
        className={`nav-item ${currentView === 'applications' ? 'active' : ''}`}
        onClick={() => setCurrentView('applications')}
      >
        <Monitor size={18} />
        Applications
      </div>
      
      <div 
        className={`nav-item ${currentView === 'discover' ? 'active' : ''}`}
        onClick={() => setCurrentView('discover')}
      >
        <Search size={18} />
        Discover
      </div>
      
      <div 
        className={`nav-item ${currentView === 'settings' ? 'active' : ''}`}
        onClick={() => setCurrentView('settings')}
      >
        <SettingsIcon size={18} />
        Settings
      </div>
      
      <div 
        className={`nav-item ${currentView === 'logs' ? 'active' : ''}`}
        onClick={() => setCurrentView('logs')}
      >
        <FileText size={18} />
        Logs
      </div>
    </div>
  );

  const handleToggleAllApps = async (serverName: string, enabled: boolean) => {
    try {
      const server = consolidatedServers.find(s => s.name === serverName);
      if (server) {
        await Promise.all(
          server.applications.map((app: any) =>
            invoke('toggle_server', { 
              serverName, 
              application: app.name, 
              enabled 
            })
          )
        );
        await loadData();
      }
    } catch (error) {
      console.error('Failed to toggle all apps:', error);
      setError(`Failed to toggle server: ${error}`);
    }
  };

  const renderServersView = () => (
    <div>
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '20px' }}>
        <h2>MCP Servers</h2>
        <button className="btn btn-primary" onClick={loadData}>
          <RefreshCw size={16} style={{ marginRight: '8px' }} />
          Refresh
        </button>
      </div>
      
      {systemStatus && (
        <div style={{ 
          background: 'var(--bg-secondary)', 
          padding: '16px', 
          borderRadius: '8px', 
          marginBottom: '20px',
          display: 'flex',
          gap: '20px',
          border: '1px solid var(--border-color)'
        }}>
          <div>
            <strong>{systemStatus.totalServers}</strong> Total Servers
          </div>
          <div>
            <strong style={{ color: '#27ae60' }}>{systemStatus.enabledServers}</strong> Enabled
          </div>
          <div>
            <strong>{systemStatus.detectedApps}</strong> Apps Detected
          </div>
        </div>
      )}
      
      <div className="server-list">
        {consolidatedServers.map((server, index) => {
          const allEnabled = server.applications.every((app: any) => app.enabled);
          const someEnabled = server.applications.some((app: any) => app.enabled);
          
          return (
            <div key={`${server.name}-${index}`} className="server-item" style={{ flexDirection: 'column', alignItems: 'stretch' }}>
              {/* Server Header */}
              <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '12px' }}>
                <div className="server-info">
                  <h3>{server.name}</h3>
                  <p>{server.applications.length} application{server.applications.length !== 1 ? 's' : ''}</p>
                </div>
                
                <div style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
                  <div className={`status-indicator ${someEnabled ? 'status-enabled' : 'status-disabled'}`}></div>
                  
                  <button
                    onClick={() => setSelectedServer({
                      id: `${server.name}-consolidated`,
                      name: server.name,
                      application: server.applications[0]?.name || ''
                    })}
                    style={{
                      background: 'none',
                      border: 'none',
                      cursor: 'pointer',
                      color: 'var(--text-secondary)',
                      padding: '4px'
                    }}
                    title="Configure server"
                  >
                    <Edit size={16} />
                  </button>
                  
                  <div 
                    className={`toggle-switch ${allEnabled ? 'enabled' : ''}`}
                    onClick={() => handleToggleAllApps(server.name, !allEnabled)}
                    title="Toggle all applications"
                  ></div>
                </div>
              </div>
              
              {/* Application List */}
              <div style={{ display: 'grid', gap: '8px', paddingLeft: '16px' }}>
                {server.applications.map((app: any, appIndex: number) => (
                  <div key={appIndex} style={{ 
                    display: 'flex', 
                    justifyContent: 'space-between', 
                    alignItems: 'center',
                    padding: '8px 12px',
                    background: 'var(--bg-primary)',
                    borderRadius: '4px',
                    border: '1px solid var(--border-color)'
                  }}>
                    <div style={{ fontSize: '14px', color: 'var(--text-primary)' }}>
                      {app.name}
                    </div>
                    
                    <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                      <div className={`status-indicator ${app.enabled ? 'status-enabled' : 'status-disabled'}`} style={{ width: '6px', height: '6px' }}></div>
                      
                      <div 
                        className={`toggle-switch ${app.enabled ? 'enabled' : ''}`}
                        onClick={() => handleToggleServer(server.name, app.name, app.enabled)}
                        style={{ transform: 'scale(0.8)' }}
                      ></div>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          );
        })}
      </div>
      
      {consolidatedServers.length === 0 && !loading && (
        <div style={{ textAlign: 'center', padding: '40px', color: 'var(--text-secondary)' }}>
          No MCP servers found. Install some servers using the CLI or check your application configurations.
        </div>
      )}
    </div>
  );

  const handleSyncApplication = async (appName: string) => {
    try {
      await invoke('sync_application', { appName });
      setError(null);
      await loadData(); // Refresh after sync
    } catch (error) {
      console.error('Failed to sync application:', error);
      setError(`Failed to sync ${appName}: ${error}`);
    }
  };

  const renderApplicationsView = () => (
    <div>
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '20px' }}>
        <h2>Applications</h2>
        <button className="btn btn-primary" onClick={loadData}>
          <RefreshCw size={16} style={{ marginRight: '8px' }} />
          Refresh
        </button>
      </div>
      
      <div className="server-list" style={{ marginTop: '20px' }}>
        {applications.map((app, index) => (
          <div key={index} className="server-item" style={{ flexDirection: 'column', alignItems: 'stretch' }}>
            {/* App Header */}
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '12px' }}>
              <div className="server-info">
                <h3>{app.name}</h3>
                <p>
                  {app.detected ? `✅ Detected • ${app.serverCount} servers` : '❌ Not found'}
                  {app.configPath && <><br /><small style={{ color: 'var(--text-secondary)' }}>{app.configPath}</small></>}
                </p>
              </div>
              
              <div style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
                <div className={`status-indicator ${app.detected ? 'status-enabled' : 'status-disabled'}`}></div>
                
                {app.detected && (
                  <button
                    onClick={() => handleSyncApplication(app.name)}
                    className="btn btn-secondary"
                    style={{ fontSize: '12px', padding: '6px 12px' }}
                  >
                    <RefreshCw size={14} style={{ marginRight: '6px' }} />
                    Sync
                  </button>
                )}
              </div>
            </div>
            
            {/* Sync Status Details */}
            {app.detected && (
              <div style={{ 
                padding: '12px',
                background: 'var(--bg-primary)',
                borderRadius: '6px',
                border: '1px solid var(--border-color)'
              }}>
                <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr 1fr', gap: '16px', fontSize: '12px' }}>
                  <div>
                    <strong style={{ color: 'var(--text-primary)' }}>Last Sync:</strong>
                    <div style={{ color: 'var(--text-secondary)' }}>
                      {app.lastSync || 'Never'}
                    </div>
                  </div>
                  <div>
                    <strong style={{ color: 'var(--text-primary)' }}>Status:</strong>
                    <div style={{ color: app.syncStatus === 'synced' ? '#27ae60' : '#f39c12' }}>
                      {app.syncStatus || 'Unknown'}
                    </div>
                  </div>
                  <div>
                    <strong style={{ color: 'var(--text-primary)' }}>Servers:</strong>
                    <div style={{ color: 'var(--text-secondary)' }}>
                      {app.serverCount} configured
                    </div>
                  </div>
                </div>
                
                {app.syncStatus === 'conflict' && (
                  <div style={{ 
                    marginTop: '8px', 
                    padding: '8px', 
                    background: '#fff3cd', 
                    color: '#856404',
                    borderRadius: '4px',
                    fontSize: '12px'
                  }}>
                    ⚠️ Configuration conflict detected. Manual resolution required.
                  </div>
                )}
              </div>
            )}
          </div>
        ))}
      </div>
      
      {applications.length === 0 && !loading && (
        <div style={{ textAlign: 'center', padding: '40px', color: 'var(--text-secondary)' }}>
          No applications detected.
        </div>
      )}
    </div>
  );

  const renderDiscoverView = () => <Discover />;

  const renderSettingsView = () => <Settings onSettingsSaved={loadData} />;

  const renderLogsView = () => <Logs />;

  const renderContent = () => {
    if (error) {
      return (
        <div style={{ padding: '20px', background: '#ffebee', color: '#c62828', borderRadius: '8px' }}>
          <h3>Error</h3>
          <p>{error}</p>
          <button className="btn btn-primary" onClick={loadData}>Retry</button>
        </div>
      );
    }

    if (loading) {
      return (
        <div className="loading">
          <RefreshCw size={24} style={{ animation: 'spin 1s linear infinite' }} />
          <p style={{ marginTop: '10px' }}>Loading...</p>
        </div>
      );
    }

    switch (currentView) {
      case 'servers':
        return renderServersView();
      case 'applications':
        return renderApplicationsView();
      case 'discover':
        return renderDiscoverView();
      case 'settings':
        return renderSettingsView();
      case 'logs':
        return renderLogsView();
      default:
        return renderServersView();
    }
  };

  return (
    <div className="app">
      {renderSidebar()}
      <div className="main-content">
        {renderContent()}
      </div>
      
      {selectedServer && (
        <ServerDetail
          serverId={selectedServer.id}
          serverName={selectedServer.name}
          application={selectedServer.application}
          onClose={() => setSelectedServer(null)}
          onSave={() => {
            setSelectedServer(null);
            loadData(); // Refresh data after save
          }}
        />
      )}
    </div>
  );
}

export default App;
