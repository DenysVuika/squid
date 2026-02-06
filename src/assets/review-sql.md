## SQL Code Review: Issues Only

**INSTRUCTIONS:**
Analyze the provided SQL code and ONLY report critical issues requiring fixes. Ignore correct code or style preferences. Use this structure:

### [Category: Issue Type]
- **Problem**: [Specific issue]
- **Fix**: [Concise action]
- **Why**: [1-sentence justification]

---

**FOCUS AREAS (Report issues ONLY in these categories):**

1. **Performance**
   - Missing indexes on frequently queried columns
   - Full table scans (SELECT * without WHERE)
   - N+1 query problems
   - Inefficient joins

2. **Security**
   - SQL injection vulnerabilities
   - Unsafe dynamic SQL
   - Overly permissive grants
   - Sensitive data exposure

3. **Correctness**
   - Missing transaction management
   - Potential NULL issues
   - Race conditions
   - Data integrity violations

4. **Best Practices**
   - Non-portable SQL (database-specific syntax)
   - Missing constraints
   - Inefficient data types
   - Poor schema design

---

**RULES:**
- No praise (e.g., "Good use of joins")
- No generic advice (e.g., "Consider indexing")
- Prioritize security > performance > correctness
- Group by category
- Be machine-like: short, direct, scannable

---

**EXAMPLE OUTPUT:**

### Performance
- **Problem**: SELECT * FROM users without WHERE clause
- **Fix**: Add WHERE clause with indexed column
- **Why**: Causes full table scan

### Security
- **Problem**: Concatenated query with user input
- **Fix**: Use parameterized queries
- **Why**: Prevents SQL injection
