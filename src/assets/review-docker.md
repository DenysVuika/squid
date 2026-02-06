## Docker/Kubernetes Review: Issues Only

**INSTRUCTIONS:**
Analyze the provided container configuration and ONLY report critical issues requiring fixes. Ignore correct code or style preferences. Use this structure:

### [Category: Issue Type]
- **Problem**: [Specific issue]
- **Fix**: [Concise action]
- **Why**: [1-sentence justification]

---

**FOCUS AREAS (Report issues ONLY in these categories):**

1. **Security**
   - Running as root
   - No resource limits
   - Overly permissive mounts
   - Insecure network config

2. **Performance**
   - Missing resource requests/limits
   - Inefficient image layers
   - Unoptimized volumes
   - Poor health checks

3. **Reliability**
   - Missing liveness/readiness probes
   - No restart policies
   - Unhandled dependencies
   - Poor logging config

4. **Best Practices**
   - Multi-process containers
   - Large image sizes
   - Missing health checks
   - Improper orchestration

---

**RULES:**
- No praise (e.g., "Good use of volumes")
- No generic advice (e.g., "Consider better naming")
- Prioritize security > reliability > performance
- Group by category
- Be machine-like: short, direct, scannable

---

**EXAMPLE OUTPUT:**

### Security
- **Problem**: Container runs as root
- **Fix**: Use USER directive
- **Why**: Security risk

### Performance
- **Problem**: Missing CPU/memory limits
- **Fix**: Add resource constraints
- **Why**: May cause resource exhaustion
