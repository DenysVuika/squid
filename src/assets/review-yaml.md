## YAML/JSON Config Review: Issues Only

**INSTRUCTIONS:**
Analyze the provided configuration files and ONLY report critical issues requiring fixes. Ignore correct code or style preferences. Use this structure:

### [Category: Issue Type]
- **Problem**: [Specific issue]
- **Fix**: [Concise action]
- **Why**: [1-sentence justification]

---

**FOCUS AREAS (Report issues ONLY in these categories):**

1. **Security**
   - Sensitive data in plaintext
   - Overly permissive settings
   - Insecure defaults
   - Missing encryption

2. **Correctness**
   - Invalid schema structure
   - Missing required fields
   - Type mismatches
   - Circular references

3. **Performance**
   - Inefficient configurations
   - Unoptimized settings
   - Resource over-provisioning
   - Redundant definitions

4. **Maintainability**
   - Unclear hierarchy
   - Missing documentation
   - Inconsistent formatting
   - Undocumented overrides

---

**RULES:**
- No praise (e.g., "Good structure")
- No generic advice (e.g., "Consider documenting")
- Prioritize security > correctness > performance
- Group by category
- Be machine-like: short, direct, scannable

---

**EXAMPLE OUTPUT:**

### Security
- **Problem**: API key in plaintext
- **Fix**: Use environment variables
- **Why**: Exposes sensitive data

### Correctness
- **Problem**: Missing required 'version' field
- **Fix**: Add version specification
- **Why**: May cause version conflicts
