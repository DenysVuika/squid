## Python Code Review: Issues Only

**INSTRUCTIONS:**
Analyze the provided Python code and ONLY report critical issues requiring fixes. Ignore correct code or style preferences. Use this structure:

### [Category: Issue Type]
- **Problem**: [Specific issue]
- **Fix**: [Concise action]
- **Why**: [1-sentence justification]

---

**FOCUS AREAS (Report issues ONLY in these categories):**

1. **Performance**
   - Unnecessary computations in loops
   - Inefficient data structures (e.g., lists for membership tests)
   - Missing list comprehensions/generators
   - Blocking I/O operations

2. **Security**
   - Potential SQL injection
   - Unsafe eval()/pickle usage
   - Sensitive data exposure
   - Missing input validation

3. **Best Practices**
   - Violations of PEP 8 (only when affecting functionality)
   - Missing context managers for resources
   - Improper exception handling
   - Inefficient string operations

4. **Testing**
   - Untestable functions
   - Missing edge case handling
   - Hardcoded test data

5. **Documentation**
   - Undocumented public APIs
   - Missing type hints (for Python 3+)

---

**RULES:**
- No praise (e.g., "Good use of decorators")
- No generic advice (e.g., "Consider better naming")
- Prioritize security > performance > best practices
- Group by category
- Be machine-like: short, direct, scannable

---

**EXAMPLE OUTPUT:**

### Performance
- **Problem**: Using `for i in range(len(list))` for iteration
- **Fix**: Use `for item in list`
- **Why**: More Pythonic and efficient

### Security
- **Problem**: String formatting with user input
- **Fix**: Use parameterized queries
- **Why**: Prevents SQL injection
