# Tech Stack Alignment: Tauri (Rust + Web Frontend)

## 🎯 **Confirmed Tech Stack**

Per the original project plan, MCP Control Lite uses:

### **Backend: Rust**
- **Location**: `src-tauri/src/`
- **Purpose**: Core business logic, file operations, system integration
- **Benefits**: Native performance, small binary size, memory safety

### **Frontend: Web Technologies**
- **Location**: `src/` (HTML/CSS/JavaScript)
- **Purpose**: User interface for both GUI and web components
- **Framework**: Vanilla JS or lightweight framework (React/Vue optional)

### **Integration: Tauri Framework**
- **Purpose**: Bridge between Rust backend and web frontend
- **Features**: Native macOS app, CLI generation, auto-updater, system tray

## 📁 **Project Structure**

```
mcp-control-lite/
├── src-tauri/              # Rust backend
│   ├── src/
│   │   ├── models/         # Data models (Rust structs)
│   │   │   ├── mod.rs
│   │   │   ├── server.rs   # MCPServerConfig
│   │   │   ├── app.rs      # ApplicationProfile
│   │   │   ├── prefs.rs    # UserPreferences
│   │   │   └── registry.rs # ServerRegistry
│   │   ├── services/       # Business logic
│   │   │   ├── mod.rs
│   │   │   ├── config.rs   # Configuration management
│   │   │   ├── fs.rs       # File system operations
│   │   │   └── detection.rs # App detection
│   │   ├── utils/          # Utilities
│   │   ├── commands/       # Tauri commands (IPC)
│   │   └── main.rs         # Entry point
│   ├── Cargo.toml          # Rust dependencies
│   └── tauri.conf.json     # Tauri configuration
├── src/                    # Web frontend
│   ├── index.html
│   ├── main.js
│   ├── styles.css
│   └── components/         # UI components
├── package.json            # Frontend dependencies
└── .taskmaster/            # TaskMaster files
```

## 🔧 **Development Setup**

### **Prerequisites**
```bash
# 1. Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 2. Install Node.js (for frontend tooling)
brew install node

# 3. Install Tauri CLI
cargo install tauri-cli
```

### **Project Initialization**
```bash
# Initialize Tauri project (if not done)
cargo tauri init

# Install frontend dependencies
npm install

# Development mode
cargo tauri dev

# Build for production
cargo tauri build
```

## 📋 **Task Implementation Approach**

### **Task 1: Define Core Data Models**
- **Language**: Rust structs with serde
- **Location**: `src-tauri/src/models/`
- **Serialization**: JSON via serde_json
- **State Management**: Tauri's managed state

### **Task 2: File System Operations**
- **Language**: Rust
- **Location**: `src-tauri/src/services/fs.rs`
- **Libraries**: `std::fs`, `tokio::fs` for async operations

### **Task 3: Application Detection**
- **Language**: Rust
- **Location**: `src-tauri/src/services/detection.rs`
- **Platform**: macOS-specific detection logic

### **Tasks 7-8: GUI and CLI**
- **GUI**: Tauri app with web frontend
- **CLI**: Rust binary (same codebase, different entry point)
- **Shared Logic**: All business logic in Rust backend

## 🔄 **TaskMaster-ClickUp Sync Alignment**

All progress tracking remains the same:
- **TaskMaster**: Source of truth for task structure
- **ClickUp**: Progress tracking, notes, comments
- **Implementation**: Now uses Rust instead of TypeScript
- **Workflow**: Unchanged (still hierarchical project structure)

## 🚀 **Benefits of Tauri Stack**

1. **Single Codebase**: Both GUI and CLI from same Rust backend
2. **Native Performance**: Rust backend with native macOS integration
3. **Small Bundle**: ~10-15MB vs ~100MB+ for Electron
4. **Security**: Rust's memory safety + Tauri's security model
5. **Auto-updater**: Built-in update mechanism
6. **Cross-platform**: Easy expansion to Windows/Linux later

## 📝 **Updated Development Workflow**

1. **Backend Development**: Write Rust code in `src-tauri/src/`
2. **Frontend Development**: Web UI in `src/`
3. **IPC Communication**: Tauri commands bridge Rust ↔ Frontend
4. **Testing**: Rust unit tests + integration tests
5. **Building**: `cargo tauri build` creates native app + CLI

---

**All documentation and task details now aligned with Tauri (Rust + Web Frontend) tech stack.**
