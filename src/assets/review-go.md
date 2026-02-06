## Go Code Review: Issues Only

**INSTRUCTIONS:**
Analyze the provided Go code and ONLY report critical issues requiring fixes. Ignore correct code or style preferences. Use this structure:

### [Category: Issue Type]
- **Problem**: [Specific issue]
- **Fix**: [Concise action]
- **Why**: [1-sentence justification]

---

**FOCUS AREAS (Report issues ONLY in these categories):**

1. **Concurrency**
   - Missing mutexes for shared resources
   - Inefficient channel usage
   - Potential race conditions
   - Goroutine leaks

2. **Performance**
   - Unnecessary allocations
   - Inefficient slices/maps
   - Missing preallocation
   - Blocking operations

3. **Error Handling**
   - Ignored errors
   - Overly generic errors
   - Missing context in errors
   - Panic misuse

4. **Best Practices**
   - Improper interface usage
   - Missing context package
   - Inefficient string handling
   - Poor dependency management

---

**RULES:**
- No praise (e.g., "Good use of channels")
- No generic advice (e.g., "Consider better naming")
- Prioritize concurrency > performance > errors
- Group by category
- Be machine-like: short, direct, scannable

---

**EXAMPLE OUTPUT:**

### Concurrency
- **Problem**: Shared variable without mutex
- **Fix**: Add sync.Mutex protection
- **Why**: Prevents race conditions

### Performance
- **Problem**: String concatenation in loop
- **Fix**: Use strings.Builder
- **Why**: Reduces allocations
