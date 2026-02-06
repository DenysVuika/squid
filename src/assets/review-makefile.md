## Makefile Review: Issues Only

**INSTRUCTIONS:**
Analyze the provided Makefile and ONLY report critical issues requiring fixes. Ignore correct code or style preferences. Use this structure:

### [Category: Issue Type]
- **Problem**: [Specific issue]
- **Fix**: [Concise action]
- **Why**: [1-sentence justification]

---

**FOCUS AREAS (Report issues ONLY in these categories):**

1. **Correctness**
   - Missing dependencies
   - Incorrect phony targets
   - Variable misuse
   - Command errors

2. **Portability**
   - Non-POSIX commands
   - Hardcoded paths
   - Platform-specific syntax
   - Shell incompatibilities

3. **Performance**
   - Unnecessary rebuilds
   - Inefficient rules
   - Redundant commands
   - Poor parallelization

4. **Security**
   - Unsafe shell commands
   - Improper permissions
   - Sensitive data exposure
   - Missing .PHONY targets

---

**RULES:**
- No praise (e.g., "Good use of variables")
- No generic advice (e.g., "Consider better naming")
- Prioritize correctness > portability > performance
- Group by category
- Be machine-like: short, direct, scannable

---

**EXAMPLE OUTPUT:**

### Correctness
- **Problem**: Missing .PHONY declaration
- **Fix**: Add .PHONY: target
- **Why**: Prevents file collision issues

### Portability
- **Problem**: Uses bash-specific syntax
- **Fix**: Use POSIX shell commands
- **Why**: Ensures cross-platform compatibility
