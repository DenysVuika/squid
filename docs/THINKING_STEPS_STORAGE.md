# Plan: Store Exact Order of Thinking Steps

## Overview

Currently, reasoning and tools are stored separately in the database:
- `messages.reasoning` - single TEXT field with all reasoning concatenated
- `messages.tools` - JSON array of tool invocations

**Problem**: We lose the exact order of when reasoning and tool steps occurred. This makes it impossible to accurately reconstruct the chain of thought (e.g., reasoning → tool → reasoning → tool → answer).

## Proposed Solution

Create a new table to store thinking steps with their exact order.

### Database Schema Changes

#### New Table: `thinking_steps`

```sql
CREATE TABLE thinking_steps (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    message_id INTEGER NOT NULL,
    step_order INTEGER NOT NULL,
    step_type TEXT NOT NULL, -- 'reasoning' or 'tool'
    content TEXT, -- for reasoning steps
    tool_name TEXT, -- for tool steps
    tool_arguments TEXT, -- JSON for tool steps
    tool_result TEXT, -- for tool steps
    tool_error TEXT, -- for tool steps
    created_at INTEGER NOT NULL,
    FOREIGN KEY (message_id) REFERENCES messages(id) ON DELETE CASCADE
);

CREATE INDEX idx_thinking_steps_message_id ON thinking_steps(message_id);
CREATE INDEX idx_thinking_steps_order ON thinking_steps(message_id, step_order);
```

### Implementation Steps

#### 1. Create Migration File

Create `migrations/009_thinking_steps.sql`:

```sql
-- Thinking steps tracking
-- Version: 009
-- Description: Adds thinking_steps table to store the exact order of reasoning and tool steps

CREATE TABLE thinking_steps (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    message_id INTEGER NOT NULL,
    step_order INTEGER NOT NULL,
    step_type TEXT NOT NULL CHECK(step_type IN ('reasoning', 'tool')),
    content TEXT,
    tool_name TEXT,
    tool_arguments TEXT,
    tool_result TEXT,
    tool_error TEXT,
    created_at INTEGER NOT NULL,
    FOREIGN KEY (message_id) REFERENCES messages(id) ON DELETE CASCADE
);

CREATE INDEX idx_thinking_steps_message_id ON thinking_steps(message_id);
CREATE INDEX idx_thinking_steps_order ON thinking_steps(message_id, step_order);
```

#### 2. Update Rust Types (`src/session.rs`)

Add new struct:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThinkingStep {
    pub step_type: String, // "reasoning" or "tool"
    pub step_order: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>, // for reasoning
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_arguments: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_result: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_error: Option<String>,
}
```

Update `ChatMessage`:

```rust
pub struct ChatMessage {
    pub role: String,
    pub content: String,
    pub sources: Vec<Source>,
    pub timestamp: i64,
    pub reasoning: Option<String>, // Keep for backward compatibility
    pub tools: Option<Vec<ToolInvocation>>, // Keep for backward compatibility
    pub thinking_steps: Option<Vec<ThinkingStep>>, // NEW
}
```

#### 3. Update Database Layer (`src/db.rs`)

##### Saving Thinking Steps

Add method to save thinking steps:

```rust
pub fn save_thinking_steps(
    &self,
    message_id: i64,
    steps: &[ThinkingStep],
) -> SqliteResult<()> {
    let conn = self.conn.lock().unwrap();
    
    for step in steps {
        let tool_args_json = step.tool_arguments.as_ref()
            .map(|args| serde_json::to_string(args).unwrap_or_default());
        
        conn.execute(
            "INSERT INTO thinking_steps (message_id, step_order, step_type, content, tool_name, tool_arguments, tool_result, tool_error, created_at) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                message_id,
                step.step_order,
                step.step_type,
                step.content,
                step.tool_name,
                tool_args_json,
                step.tool_result,
                step.tool_error,
                chrono::Utc::now().timestamp(),
            ],
        )?;
    }
    
    Ok(())
}
```

##### Loading Thinking Steps

Update `get_session` to load thinking steps:

```rust
// After loading messages, load thinking steps for each message
let mut steps_stmt = conn.prepare(
    "SELECT step_order, step_type, content, tool_name, tool_arguments, tool_result, tool_error
     FROM thinking_steps
     WHERE message_id = ?1
     ORDER BY step_order ASC"
)?;

let thinking_steps = steps_stmt.query_map(params![message_id], |row| {
    let tool_args_json: Option<String> = row.get(4)?;
    let tool_arguments = tool_args_json.and_then(|json| serde_json::from_str(&json).ok());
    
    Ok(ThinkingStep {
        step_order: row.get(0)?,
        step_type: row.get(1)?,
        content: row.get(2)?,
        tool_name: row.get(3)?,
        tool_arguments,
        tool_result: row.get(5)?,
        tool_error: row.get(6)?,
    })
})?.collect::<SqliteResult<Vec<ThinkingStep>>>()?;

// Add to message
message.thinking_steps = if thinking_steps.is_empty() {
    None
} else {
    Some(thinking_steps)
};
```

#### 4. Update API Layer (`src/api.rs`)

When saving assistant messages, build thinking steps array:

```rust
// After parsing <think> tags and collecting tool invocations
let mut thinking_steps = Vec::new();
let mut step_order = 0;

// Add reasoning blocks as steps
for reasoning_block in reasoning_parts.iter() {
    thinking_steps.push(ThinkingStep {
        step_type: "reasoning".to_string(),
        step_order,
        content: Some(reasoning_block.clone()),
        tool_name: None,
        tool_arguments: None,
        tool_result: None,
        tool_error: None,
    });
    step_order += 1;
}

// Add tool invocations as steps
for tool in collected_tool_invocations.iter() {
    thinking_steps.push(ThinkingStep {
        step_type: "tool".to_string(),
        step_order,
        content: None,
        tool_name: Some(tool.name.clone()),
        tool_arguments: Some(tool.arguments.clone()),
        tool_result: tool.result.clone(),
        tool_error: tool.error.clone(),
    });
    step_order += 1;
}

// Save thinking steps
if !thinking_steps.is_empty() {
    session_manager.save_thinking_steps(message_id, &thinking_steps)?;
}
```

#### 5. Update Frontend (`web/src/stores/chat-store.ts`)

The frontend already has the `ThinkingStep` type and uses it. When loading sessions, it will automatically use the `thinking_steps` field if available, falling back to the old behavior for backward compatibility.

### Backward Compatibility

- Keep `reasoning` and `tools` fields in the `messages` table
- Old messages without `thinking_steps` will continue to work
- Frontend falls back to building steps from `reasoning` + `tools` if `thinking_steps` is not available
- Migration is non-breaking

### Benefits

1. ✅ **Exact order preservation** - Know exactly when each reasoning and tool step occurred
2. ✅ **Better chain of thought** - Display the true flow of AI's thinking process
3. ✅ **Multiple reasoning blocks** - Support reasoning before, between, and after tools
4. ✅ **Backward compatible** - Old data continues to work
5. ✅ **Future-proof** - Easy to add new step types (e.g., "observation", "reflection")

### Testing Plan

1. Create new migration and run it
2. Test with new messages that have mixed reasoning/tool steps
3. Test loading old messages (should fall back gracefully)
4. Verify chain of thought displays correctly in both cases
5. Test session reload shows proper order

## Current Status

- ✅ Frontend supports thinking steps structure
- ✅ Frontend displays chain of thought
- ⏳ Database migration needed
- ⏳ Backend storage implementation needed
- ⏳ Backend loading implementation needed

## Notes

- Consider adding a `thinking_steps` column to the `messages` table as JSON for simpler queries (alternative to separate table)
- The separate table approach is more normalized and easier to query/modify individual steps
- Step order starts at 0 and increments for each step in the message
