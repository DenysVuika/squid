# Example Project: Documentation Bot

This example shows how to use squid with RAG for a real-world project.

## Scenario

You're building a documentation bot for your team. Your project has:
- API documentation
- Setup guides  
- Code examples
- Troubleshooting tips

Instead of searching manually, team members can ask questions and get instant answers.

## Project Structure

```
my-project/
├── documents/           # RAG document folder
│   ├── api-guide.md
│   ├── setup.md
│   ├── examples.md
│   └── troubleshooting.md
├── src/
├── tests/
├── squid.config.json   # Squid configuration
└── .squidignore        # Files to exclude
```

## Setup

### 1. Initialize Squid

```bash
cd my-project
squid init
```

Follow the prompts to configure your API endpoint and model.

### 2. Organize Documentation

```bash
mkdir documents
cp docs/*.md documents/
```

### 3. Index Documents

```bash
squid rag init
```

Output:
```
🦑 Scanning documents directory...
Found 4 documents (52 KB)
Embedding documents... ████████████ 4/4
✓ Indexed 4 documents, 89 chunks
```

## Usage Examples

### Example 1: API Questions

**Query:** "How do I authenticate API requests?"

**Result:** Squid searches `api-guide.md`, finds the authentication section, and provides a complete answer with code examples.

### Example 2: Setup Issues

**Query:** "I'm getting a connection refused error"

**Result:** Squid searches `troubleshooting.md`, finds the relevant section, and suggests solutions.

### Example 3: Code Examples

**Query:** "Show me how to paginate results"

**Result:** Squid finds pagination examples in `examples.md` and provides working code.

## Configuration

`squid.config.json`:

```json
{
  "api_url": "http://127.0.0.1:1234/v1",
  "api_model": "qwen2.5-coder-7b-instruct",
  "context_window": 32768,
  "rag": {
    "enabled": true,
    "chunk_size": 512,
    "top_k": 5
  }
}
```

## Team Workflow

### For Documentation Writers

1. Write docs in Markdown
2. Save to `documents/` folder
3. Run `squid rag init` to index
4. Test with sample queries

### For Team Members

1. Start squid: `squid serve`
2. Open Web UI: `http://localhost:8080`
3. Enable RAG mode
4. Ask questions naturally

## Advanced Features

### Automatic Updates

When running `squid serve`, file watching is enabled:
- Add new docs → automatically indexed
- Edit existing docs → automatically re-indexed
- Delete docs → automatically removed from index

### Custom Prompts

Create `.squid-prompts/` for custom system prompts:

```bash
mkdir .squid-prompts
echo "You are a helpful API documentation assistant" > .squid-prompts/api-helper.md
```

Use with:
```bash
squid ask "How do I..." --prompt .squid-prompts/api-helper.md
```

### Multiple Workspaces

Different projects, different documents:

```bash
# Project A
cd project-a
squid serve  # Uses ./documents/ from project-a

# Project B  
cd project-b
squid serve  # Uses ./documents/ from project-b
```

## Measuring Success

Track usage with squid's built-in stats:

```bash
squid rag stats
```

Output:
```
📊 RAG Statistics:
- Documents: 4
- Chunks: 89
- Avg chunks/doc: 22
- Storage: 2.3 MB
- Last indexed: 2024-01-15 10:30:00
```

## Tips & Tricks

### Organizing Docs

✅ **Good structure:**
```
documents/
├── 01-getting-started.md
├── 02-api-reference.md
├── 03-examples/
│   ├── auth.md
│   └── pagination.md
└── 99-troubleshooting.md
```

❌ **Poor structure:**
```
documents/
├── stuff.md
├── notes.md
└── misc.md
```

### Writing RAG-Friendly Docs

- **Use clear headings**: Makes chunking effective
- **Include keywords**: Improves search results
- **Add examples**: LLM can reference them directly
- **Stay concise**: 200-500 words per section

### Querying Tips

Instead of: *"authentication"*  
Try: *"How do I add authentication to API requests?"*

Instead of: *"error"*  
Try: *"Why am I getting a connection refused error?"*

## Results

After setting up RAG:
- ✅ Team finds answers **instantly**
- ✅ Fewer repetitive questions
- ✅ Documentation **actually used**
- ✅ New members onboard **faster**

## Next Steps

1. Add your project's documentation
2. Run `squid rag init`
3. Share with your team
4. Iterate based on usage

Happy documenting! 🚀
