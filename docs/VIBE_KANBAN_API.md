# Vibe Kanban MCP API Guide

## Overview

Vibe Kanban provides a **local MCP (Model Context Protocol) server** that allows you to programmatically create and manage tasks without starting them immediately. This is perfect for setting up your task backlog before agents begin work.

---

## MCP Server Basics

### What is MCP?

MCP (Model Context Protocol) is a protocol that allows external tools and agents to interact with Vibe Kanban programmatically. The server runs **locally only** - it's not accessible over the internet.

### Starting the MCP Server

The MCP server starts automatically when you run Vibe Kanban:

```bash
npx vibe-kanban
```

The server will be available on your local machine. You can connect to it using an MCP client.

---

## Available MCP Tools

### 1. `list_projects`

Get all projects managed by Vibe Kanban.

**Parameters**: None

**Returns**: Array of project objects with `project_id`, name, repository path, etc.

**Example**:
```json
{
  "tool": "list_projects",
  "parameters": {}
}
```

**Use case**: Find the `project_id` for your moteOS project before creating tasks.

---

### 2. `list_tasks`

Get tasks for a specific project.

**Parameters**:
- `project_id` (required): The project ID
- `status` (optional): Filter by status ("todo", "in_progress", "in_review", "done")

**Example**:
```json
{
  "tool": "list_tasks",
  "parameters": {
    "project_id": "proj-abc123",
    "status": "todo"
  }
}
```

**Use case**: Check existing tasks, see what's in the backlog.

---

### 3. `create_task` ⭐

**Create a task without starting it.**

**Parameters**:
- `project_id` (required): The project ID
- `title` (required): Task title
- `description` (optional): Task description
- `tags` (optional): Array of tag strings
- `priority` (optional): Priority level

**Returns**: Task object with `task_id`

**Example**:
```json
{
  "tool": "create_task",
  "parameters": {
    "project_id": "proj-abc123",
    "title": "[1] UEFI boot entry point",
    "description": "Implement UEFI entry point using uefi-rs\n- Acquire framebuffer via Graphics Output Protocol\n- Get memory map and exit boot services",
    "tags": ["workstream-1", "priority-p0", "foundation", "blocking"],
    "priority": "P0"
  }
}
```

**Important**: This creates the task in **"To Do"** status. It does **NOT** start any agent work. The task will sit in the backlog until you explicitly start it.

---

### 4. `start_task_attempt`

Start working on a task with a coding agent.

**Parameters**:
- `task_id` (required): The task ID from `create_task`
- `executor` (required): Agent name (e.g., "claude-code", "cursor-cli")
- `base_branch` (required): Git branch to base work on (usually "main")
- `variant` (optional): Agent variant/profile

**Example**:
```json
{
  "tool": "start_task_attempt",
  "parameters": {
    "task_id": "task-xyz789",
    "executor": "claude-code",
    "base_branch": "main"
  }
}
```

**Use case**: After creating tasks, start them when ready. This moves the task to "In Progress" and begins agent work.

---

## Workflow: Create Tasks Without Starting

### Step 1: Get Project ID

```json
{
  "tool": "list_projects",
  "parameters": {}
}
```

Find the `project_id` for your moteOS project in the response.

### Step 2: Create Tasks

For each task you want to create:

```json
{
  "tool": "create_task",
  "parameters": {
    "project_id": "proj-abc123",
    "title": "[1] Project scaffolding",
    "description": "Create workspace Cargo.toml...",
    "tags": ["workstream-1", "priority-p0", "foundation"]
  }
}
```

**Note**: Tasks are created in "To Do" status. No agent work starts.

### Step 3: Start Tasks Later

When you're ready to begin work on a task:

```json
{
  "tool": "start_task_attempt",
  "parameters": {
    "task_id": "task-xyz789",
    "executor": "claude-code",
    "base_branch": "main"
  }
}
```

This moves the task to "In Progress" and begins agent execution.

---

## Using the Task Creation Script

We've provided a script with all tasks pre-defined:

```bash
# View the tasks
node tools/create-vk-tasks.js

# The script outputs:
# 1. List of all tasks
# 2. JSON format for MCP API calls
```

The script contains **all tasks from the 8 workstreams** with:
- Proper titles and descriptions
- Workstream tags
- Priority levels
- Dependency notes

---

## MCP Client Integration

### Option 1: Direct MCP Client

If you have an MCP client library, you can connect directly:

```javascript
import { MCPClient } from '@modelcontextprotocol/client';

const client = new MCPClient({
  serverUrl: 'http://localhost:58507/mcp' // VK MCP endpoint
});

// List projects
const projects = await client.call('list_projects', {});

// Create task
const task = await client.call('create_task', {
  project_id: projects[0].id,
  title: "[1] UEFI boot entry point",
  description: "...",
  tags: ["workstream-1", "priority-p0"]
});
```

### Option 2: Via Cursor/Claude

If you're using Cursor or Claude with MCP support, you can ask:

> "Use the Vibe Kanban MCP server to create a task titled '[1] UEFI boot entry point' in the moteOS project"

The agent will use the MCP tools automatically.

### Option 3: Manual via UI

You can also create tasks manually in the Vibe Kanban UI:
1. Click "Create Task" (not "Create & Start")
2. Fill in title, description, tags
3. Task goes to "To Do" column
4. Start it later when ready

---

## Task Naming Convention

We use this convention for moteOS tasks:

```
[Workstream-Number] Task-Name
```

Examples:
- `[1] UEFI boot entry point`
- `[2] virtio-net driver`
- `[3] Dark theme implementation`
- `[4] OpenAI client with streaming`

---

## Task Tags

Standard tags for moteOS:

- **Workstream**: `workstream-1` through `workstream-8`
- **Priority**: `priority-p0`, `priority-p1`
- **Type**: `foundation`, `feature`, `polish`
- **Dependency**: `blocking`, `parallel`

---

## Example: Bulk Task Creation

Here's a complete example of creating multiple tasks:

```javascript
// 1. Get project ID
const projects = await mcpClient.call('list_projects', {});
const moteOSProject = projects.find(p => p.name === 'moteOS');

// 2. Create all Workstream 1 tasks
const bootTasks = [
  {
    title: "[1] Project scaffolding",
    description: "Create workspace Cargo.toml...",
    tags: ["workstream-1", "priority-p0", "foundation"]
  },
  {
    title: "[1] UEFI boot entry point",
    description: "Implement UEFI entry point...",
    tags: ["workstream-1", "priority-p0", "foundation"]
  },
  // ... more tasks
];

for (const task of bootTasks) {
  await mcpClient.call('create_task', {
    project_id: moteOSProject.id,
    ...task
  });
  console.log(`Created: ${task.title}`);
}
```

---

## Key Differences

### Create Task vs Create & Start

| Action | MCP Tool | Result |
|--------|----------|--------|
| **Create Task** | `create_task` | Task in "To Do", no agent work |
| **Create & Start** | `create_task` + `start_task_attempt` | Task in "In Progress", agent working |

### Task Status Flow

```
To Do → In Progress → In Review → Done
  ↑         ↑              ↑         ↑
create   start_task    PR created  PR merged
task     _attempt
```

---

## Limitations

1. **Local Only**: MCP server only accessible on the same machine
2. **Requires VK Running**: Vibe Kanban must be running for MCP server to be available
3. **No Remote Access**: Can't access from CI/CD or remote agents without tunneling

---

## Troubleshooting

### "MCP server not found"

- Make sure Vibe Kanban is running: `npx vibe-kanban`
- Check that the MCP server started (check VK logs)
- Verify you're connecting to the correct local endpoint

### "Project not found"

- Use `list_projects` to get the correct `project_id`
- Make sure you've created the project in VK UI first
- Project must be created from the git repository

### "Task created but not visible"

- Refresh the Vibe Kanban UI
- Check the "To Do" column (tasks start there)
- Verify `project_id` was correct

---

## Next Steps

1. **Start Vibe Kanban**: `npx vibe-kanban`
2. **Create Project**: In VK UI, create project from moteOS repo
3. **Get Project ID**: Use `list_projects` MCP tool
4. **Create Tasks**: Use `create_task` for all tasks from `tools/create-vk-tasks.js`
5. **Start When Ready**: Use `start_task_attempt` to begin work

---

**Last Updated**: January 2026
