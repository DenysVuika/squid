## Code Review: Issues Only

**INSTRUCTIONS:**
Analyze the provided code and ONLY report critical issues requiring fixes. Ignore correct code or style preferences. Use this structure:

### [Language-Specific: Issue Type]
- **Problem**: [Specific issue]
- **Fix**: [Concise action]
- **Why**: [1-sentence justification]

---

**FOCUS AREAS (Report issues ONLY in these categories):**

1. **Critical Issues**
   - Security vulnerabilities (injection, XSS, data exposure)
   - Memory/resource leaks
   - Crashes or undefined behavior
   - Missing error handling

2. **Quality Problems**
   - Duplicated code violating DRY
   - Complexity making logic unmaintainable
   - Undocumented public APIs
   - Unclear naming causing confusion

3. **Performance Bottlenecks**
   - Inefficient algorithms
   - Unnecessary computations
   - Blocking operations

4. **Testing Problems**
   - Untestable functions
   - Missing edge case handling
   - Unmockable dependencies

---

**RULES:**
- No praise (e.g., "Good use of patterns")
- No generic advice (e.g., "Consider better naming")
- Prioritize security > performance > quality
- Group by category
- Be machine-like: short, direct, scannable

---

**EXAMPLE OUTPUT:**

### Security
- **Problem**: Hardcoded password in config file
- **Fix**: Use environment variables
- **Why**: Security risk in version control

### Performance
- **Problem**: Nested loops with O(n²) complexity
- **Fix**: Use hash map for O(1) lookups
- **Why**: Will fail on large datasets
