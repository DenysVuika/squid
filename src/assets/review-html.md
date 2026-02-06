## HTML Code Review: Issues Only

**INSTRUCTIONS:**
Analyze the provided HTML and **ONLY report critical issues requiring fixes**. Ignore what’s correct. Use this structure:

### [Category: Issue Type]
- **Problem**: [Specific issue, e.g., Missing `alt` text for logo image in `<header>`]
- **Fix**: [Concise action, e.g., Add `alt="Company Logo"`]
- **Why**: [1-sentence justification, e.g., Screen readers require text alternatives for accessibility.]

---

**FOCUS AREAS (Report issues ONLY in these categories):**

1. **Semantic HTML**
   - Misused/non-semantic elements (e.g., `<div>` instead of `<section>`)
   - Broken heading hierarchy (e.g., `h1` → `h3` without `h2`)

2. **Accessibility**
   - Missing `alt`/`aria-*` attributes
   - Unlabeled form inputs
   - Low color contrast (fail WCAG AA)
   - Keyboard navigation blockers

3. **Validation Errors**
   - Unclosed tags, invalid attributes, or deprecated HTML

4. **Performance**
   - Render-blocking scripts/styles (missing `async`/`defer`)
   - Unoptimized images (large files, missing `width`/`height`)

5. **Security**
   - Unescaped user-generated content
   - Unsafe iframes (missing `sandbox`)

6. **SEO**
   - Missing meta tags (`description`, `viewport`, OpenGraph)

7. **Maintainability**
   - Inconsistent class naming (e.g., `btn-primary` vs. `primaryBtn`)
   - Poorly organized sections (e.g., mixed HTML/CSS/JS)
   - Missing comments for complex components
   - Non-reusable code (e.g., duplicated markup)

---

**RULES:**
- No praise (e.g., "Good use of...").
- No explanations beyond the fix (assume I know HTML basics).
- Prioritize critical issues (e.g., accessibility > minor formatting).
- Group by category (e.g., all semantic issues together).
- Be machine-like: Short, direct, and scannable.

---

**EXAMPLE OUTPUT:**

### Semantic HTML
- **Problem**: <div class="header"> used instead of <header>
- **Fix**: Replace with <header role="banner">
- **Why**: Semantic HTML improves SEO and screen reader navigation.

- **Problem**: Heading hierarchy jumps from <h1> to <h3>
- **Fix**: Insert <h2> before <h3>
- **Why**: Screen readers rely on logical heading order.

### Accessibility
- **Problem**: Image <img src="logo.png> lacks alt text.
- **Fix**: Add alt="Company Logo".
- **Why**: Required for WCAG compliance.
