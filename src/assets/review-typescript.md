## TypeScript/JavaScript Code Review: Issues Only

**INSTRUCTIONS:**
Analyze the provided TypeScript/JavaScript code and ONLY report critical issues requiring fixes. Ignore correct code or style preferences. Use this structure:

### [Category: Issue Type]
- **Problem**: [Specific issue, e.g. Using `any` type without justification]
- **Fix**: [Concise action, e.g. Define proper interface for this data]
- **Why**: [1-sentence justification, e.g. Loses all TypeScript type safety benefits]

---

**FOCUS AREAS (Report issues ONLY in these categories):**

1. **Type Safety**
   - `any` type usage without justification
   - Missing return types in function signatures
   - Improper null checks (missing optional chaining/nullish coalescing)
   - Implicit any (untyped function parameters)

2. **Code Quality**
   - Inconsistent naming (e.g., mixing camelCase/PascalCase)
   - Unhandled promise rejections (missing `.catch()` or `try/catch`)
   - Callback hell vs proper async/await pattern
   - Inefficient array operations (e.g., nested `.map()`)

3. **Modern JS/TS**
   - `var` usage in ES6+ code
   - Missing destructuring for object/array access
   - Regular functions when arrow functions preferable
   - Unnecessary class usage where functions would suffice

4. **Security**
   - Unsanitized `innerHTML` assignments
   - Potential XSS vulnerabilities (unsafe string interpolation)
   - Use of `eval()` or `Function` constructor
   - Missing input validation

5. **Performance**
   - Unnecessary React/Vue re-renders
   - Memory leaks (uncleared event listeners/subscriptions)
   - Large bundle size contributions (e.g., importing entire libraries)
   - Inefficient DOM manipulations

6. **Testing & Maintainability**
   - Untestable functions (hardcoded dependencies)
   - Violations of Single Responsibility Principle
   - Duplicated code blocks
   - Missing JSDoc/TSDoc for public APIs

---

**RULES:**
- No praise (e.g., "Good use of interfaces")
- No generic advice (e.g., "Consider using arrow functions")
- Prioritize critical issues (security > formatting)
- Group by category (e.g., all type safety issues together)
- Be machine-like: short, direct, scannable

---

**EXAMPLE OUTPUT:**

### Type Safety
- **Problem**: `function process(data: any)` parameter type
- **Fix**: Define interface `interface ProcessData { ... }`
- **Why**: Loses all TypeScript type checking benefits

### Security
- **Problem**: `element.innerHTML = userInput` without sanitization
- **Fix**: Use `textContent` or DOM sanitization library
- **Why**: XSS vulnerability risk

### Performance
- **Problem**: Large library import (`import { ... } from 'lodash'`)
- **Fix**: Import specific functions or use native alternatives
- **Why**: Increases bundle size unnecessarily
