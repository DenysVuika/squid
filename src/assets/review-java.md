## Java Code Review: Issues Only

**INSTRUCTIONS:**
Analyze the provided Java code and ONLY report critical issues requiring fixes. Ignore correct code or style preferences. Use this structure:

### [Category: Issue Type]
- **Problem**: [Specific issue]
- **Fix**: [Concise action]
- **Why**: [1-sentence justification]

---

**FOCUS AREAS (Report issues ONLY in these categories):**

1. **Performance**
   - Inefficient collections usage
   - Missing StringBuilder
   - Unnecessary boxing
   - Memory leaks

2. **Best Practices**
   - Raw types usage
   - Missing Optional
   - Poor equals/hashCode
   - Inefficient streams

3. **JVM Specific**
   - Finalizer misuse
   - Poor serialization
   - Threading issues
   - Classloading problems

4. **Spring Framework**
   - Transaction management
   - Dependency injection issues
   - REST controller problems
   - Security misconfigurations

---

**RULES:**
- No praise (e.g., "Good use of streams")
- No generic advice (e.g., "Consider better naming")
- Prioritize performance > best practices > framework
- Group by category
- Be machine-like: short, direct, scannable

---

**EXAMPLE OUTPUT:**

### Performance
- **Problem**: String concatenation in loop
- **Fix**: Use StringBuilder
- **Why**: Reduces memory overhead

### Best Practices
- **Problem**: Raw List usage
- **Fix**: Use List<String>
- **Why**: Type safety
