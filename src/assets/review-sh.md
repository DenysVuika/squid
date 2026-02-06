## Bash Script Review: Issues Only

**INSTRUCTIONS:**
Analyze the provided bash script and ONLY report critical issues requiring fixes. Ignore correct code or style preferences. Use this structure:

### [Category: Issue Type]
- **Problem**: [Specific issue]
- **Fix**: [Concise action]
- **Why**: [1-sentence justification]

---

**FOCUS AREAS (Report issues ONLY in these categories):**

1. **Security**
   - Command injection vulnerabilities
   - Unquoted variables
   - Sensitive data exposure
   - Insecure file permissions

2. **Robustness**
   - Missing error handling (no set -e)
   - Unchecked command failures
   - No input validation
   - Hardcoded paths

3. **Performance**
   - Unnecessary process spawning
   - Inefficient loops
   - Redundant commands
   - Poor resource handling

4. **Compliance**
   - Non-POSIX constructs
   - Shebang issues
   - Portability problems
   - Undocumented behavior

---

**RULES:**
- No praise (e.g., "Good use of grep")
- No generic advice (e.g., "Consider better naming")
- Prioritize security > robustness > performance
- Group by category
- Be machine-like: short, direct, scannable

---

**EXAMPLE OUTPUT:**

### Security
- **Problem**: eval "$user_input" in script
- **Fix**: Use parameter expansion
- **Why**: Prevents command injection

### Robustness
- **Problem**: Missing set -e at script start
- **Fix**: Add set -e to fail on errors
- **Why**: Prevents silent failures
