## CSS Code Review Instructions

Analyze the following CSS code and provide constructive feedback focusing on:

**CSS Best Practices:**
- Use of modern CSS features (Grid, Flexbox, Custom Properties)
- Specificity management and avoiding overly specific selectors
- Proper use of CSS inheritance and cascade
- DRY principles (Don't Repeat Yourself)
- Consistent naming conventions (BEM, SMACSS, or other methodologies)

**Performance:**
- Efficient selectors (avoiding universal selectors, deep nesting)
- Minimize repaints and reflows
- Use of `will-change` and transform properties
- Avoiding expensive properties (box-shadow on animations, etc.)
- Critical CSS and loading strategies

**Responsive Design:**
- Mobile-first approach
- Proper breakpoint usage
- Flexible units (rem, em, %, vh/vw) vs fixed pixels
- Container queries where appropriate
- Fluid typography

**Maintainability:**
- Logical organization and grouping
- Meaningful class names
- Avoiding magic numbers
- Use of CSS custom properties (variables)
- Comments for complex or non-obvious styles

**Accessibility:**
- Sufficient color contrast
- Focus states for interactive elements
- Respecting user preferences (prefers-reduced-motion, prefers-color-scheme)
- Readable font sizes
- Proper use of `outline` for focus indicators

**Browser Compatibility:**
- Vendor prefixes where needed
- Fallbacks for newer features
- Progressive enhancement approach

**Common Issues:**
- Unused or redundant styles
- `!important` usage (should be minimal)
- Inline styles in production code
- Z-index conflicts
- Hardcoded values that should be variables

Provide specific, actionable suggestions for improvement with modern CSS alternatives.