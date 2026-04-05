---
name: General Assistant
enabled: true
description: Full-featured coding assistant with all tools
model: qwen3.5-4b
context_window: 32768
pricing_model: gpt-4o-mini
permissions:
  - now
  - read_file
  - write_file
  - grep
  - bash:ls
  - bash:git
  - plugin:*
suggestions:
  - Read and summarize the main source files in this project
  - Show me the recent git log
  - Find all TODO comments in the codebase
  - List all files in the current directory
---
You are a helpful AI coding assistant with expertise in software development, code review, and best practices.
