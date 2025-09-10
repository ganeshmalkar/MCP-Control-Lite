import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Search, Download, Star, ExternalLink, Package, RefreshCw } from 'lucide-react';

interface MCPServerPackage {
  name: string;
  description: string;
  version: string;
  author: string;
  keywords: string[];
  repository?: string;
  downloads?: number;
  rating?: number;
  installed: boolean;
}

export default function Discover() {
  const [searchTerm, setSearchTerm] = useState('');
  const [packages, setPackages] = useState<MCPServerPackage[]>([]);
  const [loading, setLoading] = useState(false);
  const [installing, setInstalling] = useState<Set<string>>(new Set());
  const [filter, setFilter] = useState<'all' | 'popular' | 'recent'>('all');
  const [source, setSource] = useState<'npm' | 'github' | 'local'>('npm');

  useEffect(() => {
    loadPopularPackages();
  }, [source]); // Reload when source changes

  const loadPopularPackages = async () => {
    setLoading(true);
    try {
      const popularPackages = await invoke<MCPServerPackage[]>('search_mcp_packages', { 
        query: '', 
        filter: 'popular',
        source 
      });
      setPackages(popularPackages);
    } catch (error) {
      console.error('Failed to load packages:', error);
      setPackages([]);
    } finally {
      setLoading(false);
    }
  };

  const searchPackages = async () => {
    setLoading(true);
    try {
      const results = await invoke<MCPServerPackage[]>('search_mcp_packages', { 
        query: searchTerm.trim(),
        filter,
        source 
      });
      setPackages(results);
    } catch (error) {
      console.error('Search failed:', error);
      setPackages([]);
    } finally {
      setLoading(false);
    }
  };

  const installPackage = async (packageName: string) => {
    setInstalling(prev => new Set(prev).add(packageName));
    try {
      await invoke('install_mcp_package', { packageName });
      // Update package status
      setPackages(prev => prev.map(pkg => 
        pkg.name === packageName ? { ...pkg, installed: true } : pkg
      ));
    } catch (error) {
      console.error('Installation failed:', error);
    } finally {
      setInstalling(prev => {
        const newSet = new Set(prev);
        newSet.delete(packageName);
        return newSet;
      });
    }
  };

  const handleKeyPress = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter') {
      searchPackages();
    }
  };

  return (
    <div>
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '20px' }}>
        <h2>Discover MCP Servers</h2>
        <button className="btn btn-primary" onClick={loadPopularPackages}>
          <RefreshCw size={16} style={{ marginRight: '8px' }} />
          Refresh
        </button>
      </div>

      {/* Search and Filters */}
      <div style={{ 
        background: 'var(--bg-secondary)', 
        padding: '16px', 
        borderRadius: '8px', 
        marginBottom: '20px',
        border: '1px solid var(--border-color)'
      }}>
        <div style={{ display: 'flex', gap: '12px', marginBottom: '12px' }}>
          <div style={{ flex: 1, position: 'relative' }}>
            <Search size={16} style={{ 
              position: 'absolute', 
              left: '12px', 
              top: '50%', 
              transform: 'translateY(-50%)', 
              color: 'var(--text-secondary)' 
            }} />
            <input
              type="text"
              placeholder="Search MCP servers..."
              value={searchTerm}
              onChange={(e) => setSearchTerm(e.target.value)}
              onKeyPress={handleKeyPress}
              style={{
                width: '100%',
                padding: '8px 8px 8px 40px',
                border: '1px solid var(--border-color)',
                borderRadius: '4px',
                background: 'var(--bg-primary)',
                color: 'var(--text-primary)'
              }}
            />
          </div>
          <button className="btn btn-primary" onClick={searchPackages}>
            Search
          </button>
        </div>

        <div style={{ display: 'flex', gap: '8px', marginBottom: '12px' }}>
          <div style={{ display: 'flex', gap: '8px' }}>
            <label style={{ color: 'var(--text-primary)', fontSize: '14px', display: 'flex', alignItems: 'center' }}>
              Source:
            </label>
            {['npm', 'github', 'local'].map(sourceOption => (
              <button
                key={sourceOption}
                onClick={() => setSource(sourceOption as any)}
                style={{
                  padding: '4px 8px',
                  border: '1px solid var(--border-color)',
                  borderRadius: '4px',
                  background: source === sourceOption ? '#3498db' : 'var(--bg-primary)',
                  color: source === sourceOption ? 'white' : 'var(--text-primary)',
                  cursor: 'pointer',
                  textTransform: 'capitalize',
                  fontSize: '12px'
                }}
              >
                {sourceOption}
              </button>
            ))}
          </div>
        </div>

        <div style={{ display: 'flex', gap: '8px' }}>
          <label style={{ color: 'var(--text-primary)', fontSize: '14px', display: 'flex', alignItems: 'center' }}>
            Filter:
          </label>
          {['all', 'popular', 'recent'].map(filterOption => (
            <button
              key={filterOption}
              onClick={() => setFilter(filterOption as any)}
              style={{
                padding: '6px 12px',
                border: '1px solid var(--border-color)',
                borderRadius: '4px',
                background: filter === filterOption ? '#3498db' : 'var(--bg-primary)',
                color: filter === filterOption ? 'white' : 'var(--text-primary)',
                cursor: 'pointer',
                textTransform: 'capitalize'
              }}
            >
              {filterOption}
            </button>
          ))}
        </div>
      </div>

      {/* Package List */}
      {loading ? (
        <div className="loading">
          <RefreshCw size={24} style={{ animation: 'spin 1s linear infinite' }} />
          <p style={{ marginTop: '10px' }}>Searching packages...</p>
        </div>
      ) : (
        <div style={{ display: 'grid', gap: '12px' }}>
          {packages.map((pkg) => (
            <div key={pkg.name} style={{
              background: 'var(--bg-secondary)',
              border: '1px solid var(--border-color)',
              borderRadius: '8px',
              padding: '16px'
            }}>
              <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start' }}>
                <div style={{ flex: 1 }}>
                  <div style={{ display: 'flex', alignItems: 'center', gap: '8px', marginBottom: '8px' }}>
                    <Package size={18} style={{ color: 'var(--text-secondary)' }} />
                    <h3 style={{ margin: 0, color: 'var(--text-primary)' }}>{pkg.name}</h3>
                    <span style={{ 
                      fontSize: '12px', 
                      color: 'var(--text-secondary)',
                      background: 'var(--bg-primary)',
                      padding: '2px 6px',
                      borderRadius: '3px'
                    }}>
                      v{pkg.version}
                    </span>
                    {pkg.installed && (
                      <span style={{
                        fontSize: '12px',
                        color: '#27ae60',
                        background: '#e8f5e8',
                        padding: '2px 6px',
                        borderRadius: '3px'
                      }}>
                        Installed
                      </span>
                    )}
                  </div>
                  
                  <p style={{ 
                    margin: '0 0 8px 0', 
                    color: 'var(--text-secondary)',
                    fontSize: '14px'
                  }}>
                    {pkg.description}
                  </p>
                  
                  <div style={{ display: 'flex', alignItems: 'center', gap: '16px', fontSize: '12px', color: 'var(--text-secondary)' }}>
                    <span>by {pkg.author}</span>
                    {pkg.downloads && <span>{pkg.downloads.toLocaleString()} downloads</span>}
                    {pkg.rating && (
                      <div style={{ display: 'flex', alignItems: 'center', gap: '4px' }}>
                        <Star size={12} style={{ color: '#f39c12' }} />
                        {pkg.rating}
                      </div>
                    )}
                  </div>
                  
                  {pkg.keywords.length > 0 && (
                    <div style={{ marginTop: '8px', display: 'flex', gap: '4px', flexWrap: 'wrap' }}>
                      {pkg.keywords.slice(0, 3).map(keyword => (
                        <span key={keyword} style={{
                          fontSize: '11px',
                          background: 'var(--bg-primary)',
                          color: 'var(--text-secondary)',
                          padding: '2px 6px',
                          borderRadius: '3px',
                          border: '1px solid var(--border-color)'
                        }}>
                          {keyword}
                        </span>
                      ))}
                    </div>
                  )}
                </div>
                
                <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                  {pkg.repository && (
                    <button
                      onClick={() => window.open(pkg.repository, '_blank')}
                      style={{
                        background: 'none',
                        border: '1px solid var(--border-color)',
                        borderRadius: '4px',
                        padding: '6px',
                        cursor: 'pointer',
                        color: 'var(--text-secondary)'
                      }}
                      title="View repository"
                    >
                      <ExternalLink size={14} />
                    </button>
                  )}
                  
                  {!pkg.installed ? (
                    <button
                      onClick={() => installPackage(pkg.name)}
                      disabled={installing.has(pkg.name)}
                      className="btn btn-primary"
                      style={{ fontSize: '12px', padding: '6px 12px' }}
                    >
                      {installing.has(pkg.name) ? (
                        <>
                          <RefreshCw size={12} style={{ marginRight: '6px', animation: 'spin 1s linear infinite' }} />
                          Installing...
                        </>
                      ) : (
                        <>
                          <Download size={12} style={{ marginRight: '6px' }} />
                          Install
                        </>
                      )}
                    </button>
                  ) : (
                    <span style={{
                      fontSize: '12px',
                      color: '#27ae60',
                      padding: '6px 12px',
                      background: '#e8f5e8',
                      borderRadius: '4px'
                    }}>
                      ✓ Installed
                    </span>
                  )}
                </div>
              </div>
            </div>
          ))}
        </div>
      )}

      {packages.length === 0 && !loading && (
        <div style={{ textAlign: 'center', padding: '40px', color: 'var(--text-secondary)' }}>
          {searchTerm ? 'No packages found for your search.' : 'No packages available.'}
        </div>
      )}
    </div>
  );
}
