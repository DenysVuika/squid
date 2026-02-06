## CSS Code Review: Issues Only

**INSTRUCTIONS:**
Analyze the provided CSS and **ONLY report critical issues requiring fixes**. Ignore correct or opinion-based styles. Use this structure:

### [Category: Issue Type]
- **Problem**: [Specific issue, e.g., Universal selector `*` used in production]
- **Fix**: [Concise action, e.g., Replace with targeted selectors]
- **Why**: [1-sentence justification, e.g., Universal selectors degrade performance.]

---

**FOCUS AREAS (Report issues ONLY in these categories):**

1. **Best Practices**
   - Overly specific selectors (e.g., `#header .nav ul li a`)
   - Violations of DRY (repeated styles for similar elements)
   - Inconsistent naming (e.g., mixing BEM and arbitrary classes)

2. **Performance**
   - Universal selectors (`*`) or deep nesting (e.g., `.parent .child .grandchild`)
   - Expensive properties in animations (e.g., `box-shadow`, `filter`)
   - Missing `will-change` for animated elements
   - Unoptimized repaints/reflows (e.g., animating `width` instead of `transform`)

3. **Responsive Design**
   - Fixed units (e.g., `px`) for layouts instead of `rem`/`%`/`vw`
   - Missing mobile-first breakpoints
   - Hardcoded container widths (e.g., `width: 1200px`)
   - No container queries for dynamic components

4. **Maintainability**
   - Magic numbers (e.g., `margin: 37px`) without explanation
   - Unorganized/ungrouped styles (e.g., alphabetic vs. logical grouping)
   - Missing comments for complex styles (e.g., grid layouts)
   - Redundant or unused selectors

5. **Accessibility**
   - Low color contrast (fail WCAG AA)
   - Missing focus states for interactive elements
   - Ignoring `prefers-reduced-motion` or `prefers-color-scheme`
   - Disabled `outline` without alternative focus indicators

6. **Browser Compatibility**
   - Missing vendor prefixes for experimental features (e.g., `-webkit-`)
   - No fallbacks for CSS Grid/Flexbox (e.g., `display: table` fallback)
   - Assumes evergreen browsers (e.g., uses `:has()` without fallback)

7. **Common Pitfalls**
   - `!important` usage (except for utility classes)
   - Inline styles in HTML (e.g., `style="color: red"`)
   - Z-index conflicts (e.g., arbitrary values like `z-index: 9999`)
   - Hardcoded colors/values instead of CSS variables

---

**RULES:**
- **No praise** (e.g., "Good use of Flexbox").
- **No generic advice** (e.g., "Consider using Grid").
- **Prioritize critical issues** (e.g., accessibility > minor formatting).
- **Group by category** (e.g., all performance issues together).
- **Be machine-like**: Short, direct, and scannable.

---

**EXAMPLE OUTPUT:**

### Performance
- **Problem**: Universal selector `*` used in `.component * { margin: 0 }`.
- **Fix**: Replace with targeted selector (e.g., `.component p, .component ul`).
- **Why**: Universal selectors slow down rendering.

- **Problem**: `box-shadow` animated in `:hover` state.
- **Fix**: Use `transform: translateY()` for smoother animations.
- **Why**: `box-shadow` triggers expensive repaints.

### Accessibility
- **Problem**: Button focus state lacks `outline` or alternative.
- **Fix**: Add `button:focus { outline: 2px solid currentColor }`.
- **Why**: Required for keyboard navigation (WCAG).

### Maintainability
- **Problem**: Magic number `padding: 17px` in `.card`.
- **Fix**: Replace with CSS variable `--card-padding: 1rem`.
- **Why**: Variables improve consistency and scalability.
