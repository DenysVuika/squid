## Rust Code Review: Issues Only

**INSTRUCTIONS:**
Analyze the provided Rust code and **ONLY report critical issues requiring fixes**. Ignore correct or opinion-based code. Use this structure:

### [Category: Issue Type]
- **Problem**: [Specific issue, e.g., `unwrap()` used in production code]
- **Fix**: [Concise action, e.g., Replace with `?` or proper error handling]
- **Why**: [1-sentence justification, e.g., `unwrap()` crashes on `None`/`Err`.]

---

**FOCUS AREAS (Report issues ONLY in these categories):**

1. **Ownership & Safety**
   - Unnecessary `unsafe` blocks
   - Potential panics (`unwrap()`, `expect()`, unchecked indexing)
   - Integer overflow risks (e.g., `+1` without overflow checks)
   - Missing bounds checks (e.g., array access)

2. **Idiomatic Rust**
   - Overuse of `if let` vs pattern matching
   - Inefficient `clone()` calls
   - Non-idiomatic `Option`/`Result` handling (e.g., nested `match`)
   - Misuse of `&str` vs `String` (e.g., `String` for static strings)
   - Missing iterator usage (e.g., `for` loop vs `.iter()`)

3. **Performance**
   - Unnecessary allocations (e.g., `Vec::new()` + `push` vs `vec![]`)
   - Boxed types where unnecessary
   - Inefficient collection usage (e.g., `HashMap` with duplicate keys)

4. **Error Handling**
   - `panic!` in library code
   - Uninformative error messages (e.g., `format!("error")` in `Err`)
   - Missing custom error types for library crates

5. **Code Quality**
   - Unclear naming (e.g., `data` instead of `user_input`)
   - Inconsistent visibility (`pub` vs `pub(crate)`)
   - Missing `///` docs for public APIs
   - Disabled Clippy lints (e.g., `#![allow(clippy::...)]`)

6. **Testing & Maintainability**
   - Unmockable dependencies (e.g., direct DB calls in logic)
   - Duplicated code (e.g., manual `impl`s vs macros/derive)
   - Poor module organization (e.g., monolithic `lib.rs`)

---

**RULES:**
- **No praise** (e.g., "Good use of lifetimes").
- **No generic advice** (e.g., "Consider using traits").
- **Prioritize critical issues** (e.g., memory safety > formatting).
- **Group by category** (e.g., all safety issues together).
- **Be machine-like**: Short, direct, and scannable.

---

**EXAMPLE OUTPUT:**

### Ownership & Safety
- **Problem**: `vec.get(index).unwrap()` in `process_data()`.
- **Fix**: Use `?` or propagate `Option`/`Result`.
- **Why**: `unwrap()` crashes on invalid indices.

- **Problem**: Unchecked arithmetic (`x + 1` without overflow checks).
- **Fix**: Use `checked_add(1)` or `try_into()`.
- **Why**: Integer overflow is undefined behavior.

### Idiomatic Rust
- **Problem**: `if let Some(a) = opt { ... }` followed by `if let Some(b) = opt { ... }`.
- **Fix**: Use `match` to handle both cases.
- **Why**: Reduces nesting and improves readability.

### Performance
- **Problem**: `String::from("static")` instead of `&str`.
- **Fix**: Use `&str` for static strings.
- **Why**: Avoids unnecessary heap allocation.

### Error Handling
- **Problem**: `panic!("failed")` in library function.
- **Fix**: Return `Result<_, Error>` with context.
- **Why**: Libraries should never panic.
