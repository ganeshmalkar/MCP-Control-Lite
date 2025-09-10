# TaskMaster-ClickUp Sync Workflow (Hierarchical Structure)

## 🎯 Core Principle
**TaskMaster is the source of truth for task structure and dependencies. ClickUp is the workspace for progress tracking, notes, comments, and collaboration.**

## 📋 ClickUp Project Structure
- **List URL**: https://app.clickup.com/14168111/v/li/901102981630
- **List Name**: "General app, nerdy ideas to code up"
- **Project Structure**: Hierarchical with parent project task

## 🏗️ Hierarchical Organization

### Parent Project Task
- **Name**: "MCP Control Lite - Complete Project"
- **ClickUp ID**: 868faf3f5
- **Purpose**: Master project container for all development work

### TaskMaster Tasks → ClickUp Subtasks
| TaskMaster ID | ClickUp Subtask | ClickUp ID | Status |
|---------------|-----------------|------------|---------|
| TM-1 | [TM-1] Define Core Data Models | 868faf3hz | ✅ Synced |
| TM-2 | [TM-2] Implement File System Operations | 868faf3kj | ✅ Synced |
| TM-3 | [TM-3] Build Application Detection System | 868faf3p7 | ✅ Synced |
| TM-4 | [TM-4] Develop Core Configuration Engine | 868faf3r4 | ✅ Synced |
| TM-5 | [TM-5] Create Application Adapters | 868faf3tm | ✅ Synced |
| TM-6 | [TM-6] Implement MCP Server Management | 868faf3ux | ✅ Synced |
| TM-7 | [TM-7] Build GUI Application with Tauri | 868faf3w8 | ✅ Synced |
| TM-8 | [TM-8] Implement CLI Interface | 868faf3x8 | ✅ Synced |
| TM-9 | [TM-9] Implement Backup and Restore Functionality | 868faf3y2 | ✅ Synced |
| TM-10 | [TM-10] Implement System Integration and Auto-Updates | 868faf3yr | ✅ Synced |

### Future TaskMaster Subtasks → ClickUp Sub-subtasks
When TaskMaster tasks are expanded into subtasks, they become sub-subtasks in ClickUp:
- **TM-1.1** → Sub-subtask under [TM-1]
- **TM-1.2** → Sub-subtask under [TM-1]
- etc.

## 🚫 STRICT RULES - NEVER VIOLATE

### ❌ DO NOT:
1. **Never create files in the project directory for:**
   - Progress notes
   - Task summaries
   - Completion logs
   - Work journals
   - Implementation notes
   
2. **Never modify TaskMaster tasks without syncing to ClickUp**

3. **Never use local files for tracking work progress**

### ✅ ALWAYS DO:
1. **Use ClickUp for ALL progress tracking:**
   - Comments for implementation notes
   - Status updates for progress
   - Time tracking for work sessions
   - Attachments for screenshots/diagrams
   
2. **Sync TaskMaster changes to ClickUp immediately**

3. **Reference TaskMaster ID in all ClickUp activities**

## 🔄 Status Mapping

| TaskMaster Status | ClickUp Status | Action Required |
|-------------------|----------------|-----------------|
| pending | new | Ready to start |
| in-progress | in progress | Currently working |
| review | in review | Code review needed |
| done | completed | Task finished |
| blocked | blocked | Waiting on dependency |
| deferred | delayed | Postponed |
| cancelled | (delete task) | No longer needed |

## 📝 Workflow Commands

### When Starting Work on a Task:
```bash
# 1. Update TaskMaster status
set_task_status --id=1 --status=in-progress --projectRoot=/Users/peterkrzyzek/Development/mcp-control-lite

# 2. Update ClickUp status (use corresponding ClickUp task ID)
# Update via ClickUp interface or API
```

### When Adding Subtasks:
```bash
# 1. Expand task in TaskMaster
expand_task --id=1 --projectRoot=/Users/peterkrzyzek/Development/mcp-control-lite

# 2. Create corresponding subtasks in ClickUp with [TM-1.1], [TM-1.2] naming
```

### When Completing a Task:
```bash
# 1. Update TaskMaster
set_task_status --id=1 --status=done --projectRoot=/Users/peterkrzyzek/Development/mcp-control-lite

# 2. Add completion comment in ClickUp
# 3. Update ClickUp status to "completed"
```

## 💬 ClickUp Comment Templates

### Starting Work:
```
🚀 **Starting Task TM-X**
- Dependencies verified: [list dependencies]
- Approach: [brief implementation approach]
- Estimated time: [time estimate]
```

### Progress Update:
```
📈 **Progress Update - TM-X**
- Completed: [what's done]
- Current: [what you're working on]
- Next: [what's coming next]
- Blockers: [any issues]
```

### Completion:
```
✅ **Task TM-X Complete**
- Implementation: [brief summary]
- Files changed: [list key files]
- Testing: [testing approach]
- Ready for: [next dependent task]
```

## 🔍 Monitoring & Sync

### Daily Sync Check:
1. Compare TaskMaster task count with ClickUp
2. Verify status alignment
3. Check for orphaned tasks

### Weekly Review:
1. Update task priorities based on progress
2. Sync any new subtasks
3. Archive completed tasks

## 🛠 Emergency Procedures

### If Tasks Get Out of Sync:
1. **DO NOT** create duplicate tasks
2. Use TaskMaster as source of truth
3. Update ClickUp to match TaskMaster
4. Document the discrepancy in ClickUp comments

### If ClickUp Task is Accidentally Deleted:
1. Recreate from TaskMaster data
2. Use same [TM-X] naming convention
3. Add comment explaining the recreation

## 📊 Success Metrics

- ✅ All TaskMaster tasks have corresponding ClickUp tasks
- ✅ All progress is tracked in ClickUp comments
- ✅ No local files contain progress notes
- ✅ Status sync is maintained
- ✅ Dependencies are clear in both systems

---

**Remember: TaskMaster manages WHAT to do, ClickUp tracks HOW it's being done.**
