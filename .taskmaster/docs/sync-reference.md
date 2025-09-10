# Quick Reference: TaskMaster-ClickUp Sync (Hierarchical)

## 🎯 GOLDEN RULE
**NEVER write progress notes, summaries, or completion logs to project files. ALL tracking happens in ClickUp subtasks.**

## 📍 ClickUp Project Structure
**Parent Project**: https://app.clickup.com/t/868faf3f5 (MCP Control Lite - Complete Project)
**List URL**: https://app.clickup.com/14168111/v/li/901102981630

## 🏗️ Hierarchy
```
MCP Control Lite Project (868faf3f5)
├── [TM-1] Define Core Data Models (868faf3hz)
├── [TM-2] File System Operations (868faf3kj)
├── [TM-3] Application Detection (868faf3p7)
├── [TM-4] Core Configuration Engine (868faf3r4)
├── [TM-5] Application Adapters (868faf3tm)
├── [TM-6] MCP Server Management (868faf3ux)
├── [TM-7] GUI with Tauri (868faf3w8)
├── [TM-8] CLI Interface (868faf3x8)
├── [TM-9] Backup & Restore (868faf3y2)
└── [TM-10] System Integration (868faf3yr)
```

## 🔢 TaskMaster → ClickUp Mapping
```
TaskMaster Tasks = ClickUp Subtasks (under parent project)
TaskMaster Subtasks = ClickUp Sub-subtasks (under TM subtasks)

TM-1 → 868faf3hz (subtask of 868faf3f5)
TM-2 → 868faf3kj (subtask of 868faf3f5)
TM-3 → 868faf3p7 (subtask of 868faf3f5)
TM-4 → 868faf3r4 (subtask of 868faf3f5)
TM-5 → 868faf3tm (subtask of 868faf3f5)
TM-6 → 868faf3ux (subtask of 868faf3f5)
TM-7 → 868faf3w8 (subtask of 868faf3f5)
TM-8 → 868faf3x8 (subtask of 868faf3f5)
TM-9 → 868faf3y2 (subtask of 868faf3f5)
TM-10 → 868faf3yr (subtask of 868faf3f5)
```

## ⚡ Quick Commands

### Start Working on Task:
```bash
# Update TaskMaster status
set_task_status --id=1 --status=in-progress --projectRoot=/Users/peterkrzyzek/Development/mcp-control-lite

# Then update ClickUp SUBTASK (not parent) to "in progress"
# Add comment to SUBTASK with approach/plan
```

### Complete Task:
```bash
# Update TaskMaster status  
set_task_status --id=1 --status=done --projectRoot=/Users/peterkrzyzek/Development/mcp-control-lite

# Then update ClickUp SUBTASK to "completed"
# Add completion comment to SUBTASK with summary
```

### Add Subtasks:
```bash
# Expand in TaskMaster first
expand_task --id=1 --projectRoot=/Users/peterkrzyzek/Development/mcp-control-lite

# Then create sub-subtasks in ClickUp under [TM-1] with [TM-1.1] format
```

## 🦀 **Tech Stack: Tauri (Rust + Web Frontend)**

### Development Setup:
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Tauri CLI
cargo install tauri-cli

# Initialize project (if needed)
cargo tauri init

# Development mode
cargo tauri dev
```

### Project Structure:
```
src-tauri/src/models/    # Rust data models
src-tauri/src/services/  # Business logic
src/                     # Web frontend
```

## 🚨 CRITICAL REMINDERS

1. **NO local files for progress tracking**
2. **ALL notes go in ClickUp SUBTASK comments (not parent project)**
3. **TaskMaster = structure, ClickUp subtasks = progress**
4. **Always work within the project hierarchy**
5. **Never create standalone tasks - everything under parent project**
6. **Use [TM-X] naming for subtasks, [TM-X.Y] for sub-subtasks**

## 🎯 Where to Track What

| Activity | Location |
|----------|----------|
| Overall project status | Parent project (868faf3f5) |
| Task progress notes | Individual subtasks ([TM-X]) |
| Implementation details | Subtask comments |
| Code snippets | Subtask attachments |
| Time tracking | Individual subtasks |
| Completion summaries | Subtask comments |

---
*Last Updated: 2025-08-23 - Hierarchical Structure*
