# Setting Up Vibe Kanban MCP Server in Cursor

## Quick Setup

The Vibe Kanban MCP server has been configured in your Cursor settings.

**Location**: `~/.cursor/mcp.json`

**Configuration**:
```json
{
  "mcpServers": {
    "vibe_kanban": {
      "command": "npx",
      "args": ["-y", "vibe-kanban", "--mcp"]
    }
  }
}
```

## Next Steps

1. **Restart Cursor** - Close and reopen Cursor to load the MCP server configuration

2. **Verify MCP Server** - After restarting:
   - Open Cursor Settings
   - Look for "MCP Servers" section
   - You should see `vibe_kanban` listed

3. **Test the Connection** - In Cursor chat, try asking:
   ```
   List all projects in Vibe Kanban
   ```
   Or:
   ```
   Use the Vibe Kanban MCP to list projects
   ```

## Using the MCP Server

Once configured, you can use Vibe Kanban tools directly in Cursor:

### Available Tools

- **`list_projects`** - Get all Vibe Kanban projects
- **`list_tasks`** - Get tasks for a project
- **`create_task`** - Create a new task (without starting it)
- **`start_task_attempt`** - Start working on a task

### Example Usage in Cursor

You can ask Cursor to:

1. **Create tasks**:
   ```
   Create a task in Vibe Kanban for "[1] UEFI boot entry point" 
   in the moteOS project
   ```

2. **List tasks**:
   ```
   Show me all tasks in the moteOS project
   ```

3. **Bulk create tasks**:
   ```
   Create all tasks from tools/create-vk-tasks.js in Vibe Kanban
   ```

## Troubleshooting

### MCP Server Not Showing

1. **Check the config file**:
   ```bash
   cat ~/.cursor/mcp.json
   ```

2. **Verify JSON syntax** - Make sure the JSON is valid

3. **Restart Cursor** - Fully quit and restart Cursor

4. **Check Cursor logs** - Look for MCP-related errors

### "Command not found" Errors

If you see errors about `npx` or `vibe-kanban`:

1. **Verify Node.js is installed**:
   ```bash
   node --version
   ```

2. **Test Vibe Kanban directly**:
   ```bash
   npx vibe-kanban --mcp
   ```

3. **Use full path** (if needed):
   ```json
   {
     "mcpServers": {
       "vibe_kanban": {
         "command": "/usr/local/bin/npx",
         "args": ["-y", "vibe-kanban", "--mcp"]
       }
     }
   }
   ```

### Server Not Responding

1. **Check if Vibe Kanban is running**:
   - The MCP server should start automatically when Cursor connects
   - You can also run `npx vibe-kanban` separately

2. **Check permissions**:
   - Make sure Cursor has necessary permissions
   - On macOS, check System Settings → Privacy & Security

3. **Check ports**:
   - Vibe Kanban MCP server runs locally
   - No firewall rules should block it

## Manual Configuration

If you need to edit the config manually:

1. **Open the config file**:
   ```bash
   open ~/.cursor/mcp.json
   # or
   code ~/.cursor/mcp.json
   ```

2. **Add the Vibe Kanban server**:
   ```json
   {
     "mcpServers": {
       "vibe_kanban": {
         "command": "npx",
         "args": ["-y", "vibe-kanban", "--mcp"]
       }
     }
   }
   ```

3. **Save and restart Cursor**

## Advanced Configuration

### Multiple MCP Servers

You can add multiple MCP servers:

```json
{
  "mcpServers": {
    "vibe_kanban": {
      "command": "npx",
      "args": ["-y", "vibe-kanban", "--mcp"]
    },
    "another_server": {
      "command": "node",
      "args": ["/path/to/server.js"]
    }
  }
}
```

### Custom Arguments

You can customize the Vibe Kanban launch:

```json
{
  "mcpServers": {
    "vibe_kanban": {
      "command": "npx",
      "args": ["-y", "vibe-kanban", "--mcp", "--port", "8080"]
    }
  }
}
```

## Verification

After setup, verify it's working:

1. **In Cursor chat**, ask:
   ```
   What MCP servers are available?
   ```

2. **Or try a direct command**:
   ```
   Use Vibe Kanban MCP to list all projects
   ```

3. **Check Cursor's MCP panel** (if available in your version)

---

**Last Updated**: January 2026
