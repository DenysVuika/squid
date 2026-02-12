# Token Tracking and Cost Calculation

This document explains how Squid tracks token usage and calculates costs for AI model interactions.

## Overview

Squid automatically tracks token usage for each chat session and displays:
- **Real-time token counts** during streaming responses
- **Per-session accumulation** across multiple conversation turns
- **Cost estimates** in USD based on model pricing
- **Breakdown by token type** (input, output, reasoning, cache)

## How It Works

### 1. Token Capture from LLM API

When you send a message, Squid captures token usage from the OpenAI-compatible API response:

```rust
// From src/api.rs - Streaming response handler
if let Some(usage) = &response.usage {
    debug!(
        "Token usage - Prompt: {}, Completion: {}, Total: {}",
        usage.prompt_tokens, usage.completion_tokens, usage.total_tokens
    );

    yield Ok(StreamEvent::Usage {
        input_tokens: usage.prompt_tokens as i64,
        output_tokens: usage.completion_tokens as i64,
        reasoning_tokens: 0, // Not provided by standard OpenAI API
        cache_tokens: 0,     // Not provided by standard OpenAI API
    });
}
```

### 2. Database Storage

Token usage is persisted in the SQLite database per session:

```sql
-- Database schema (migrations/004_token_tracking.sql)
ALTER TABLE sessions ADD COLUMN model_id TEXT;
ALTER TABLE sessions ADD COLUMN total_tokens INTEGER DEFAULT 0;
ALTER TABLE sessions ADD COLUMN input_tokens INTEGER DEFAULT 0;
ALTER TABLE sessions ADD COLUMN output_tokens INTEGER DEFAULT 0;
ALTER TABLE sessions ADD COLUMN reasoning_tokens INTEGER DEFAULT 0;
ALTER TABLE sessions ADD COLUMN cache_tokens INTEGER DEFAULT 0;
ALTER TABLE sessions ADD COLUMN cost_usd REAL DEFAULT 0.0;
```

### 3. Session-Level Accumulation

Tokens accumulate across multiple conversation turns in the same session:

```rust
// From src/session.rs
pub fn add_tokens(&mut self, input: i64, output: i64, reasoning: i64, cache: i64) {
    self.token_usage.input_tokens += input;
    self.token_usage.output_tokens += output;
    self.token_usage.reasoning_tokens += reasoning;
    self.token_usage.cache_tokens += cache;
    self.token_usage.total_tokens =
        self.token_usage.input_tokens +
        self.token_usage.output_tokens +
        self.token_usage.reasoning_tokens +
        self.token_usage.cache_tokens;
}
```

### 4. Model Tracking

The first model used in a session is tracked and persists:

```rust
pub fn set_model(&mut self, model_id: String) {
    if self.model_id.is_none() {
        self.model_id = Some(model_id);
    }
}
```

## Cost Calculation

### TokenLens Library

Squid uses the **[tokenlens](https://www.npmjs.com/package/tokenlens)** library (v1.3.1) for cost calculation in the web UI.

TokenLens is a community-maintained library that:
- Provides a comprehensive database of AI model specifications and pricing
- Sources pricing data from **[models.dev](https://github.com/anomalyco/models.dev)** - an open-source database of AI models
- Supports 270+ contributors maintaining accurate pricing information
- Updates regularly to reflect provider pricing changes

### Pricing Data Source

The pricing data comes from **models.dev**, which aggregates official pricing from:
- **OpenAI** - https://openai.com/pricing
- **Anthropic** - https://anthropic.com/pricing
- **Google** - https://ai.google.dev/pricing
- **Mistral** - https://mistral.ai/technology/#pricing
- **Cohere** - https://cohere.com/pricing
- **Deepseek** - https://platform.deepseek.com/api-docs/pricing
- And many more providers

Each model's pricing is stored as **cost per million tokens**:

```toml
# Example from models.dev
[cost]
input = 3.00                # $3.00 per million input tokens
output = 15.00              # $15.00 per million output tokens
reasoning = 15.00           # $15.00 per million reasoning tokens (for models with CoT)
cache_read = 0.30           # $0.30 per million cached read tokens
cache_write = 3.75          # $3.75 per million cached write tokens
```

### Cost Calculation in UI

The Context component calculates costs using tokenlens:

```typescript
// From web/src/components/ai-elements/context.tsx
import { getUsage } from "tokenlens";

const costUSD = modelId
  ? getUsage({
      modelId,
      usage: {
        input: usage?.inputTokens ?? 0,
        output: usage?.outputTokens ?? 0,
      },
    }).costUSD?.totalUSD
  : undefined;
```

### Example Pricing (as of 2024)

Common model pricing per million tokens:

| Model | Input | Output |
|-------|--------|--------|
| GPT-4o | $2.50 | $10.00 |
| GPT-4o-mini | $0.15 | $0.60 |
| Claude 3.5 Sonnet | $3.00 | $15.00 |
| Claude 3.5 Haiku | $0.80 | $4.00 |
| Gemini 1.5 Pro | $1.25 | $5.00 |
| Llama 3.1 405B | $2.70 | $2.70 |
| Deepseek-V3 | $0.27 | $1.10 |

**Note:** Pricing changes frequently. TokenLens is regularly updated to reflect current rates.

### Local Models (LM Studio, Ollama)

For **local models**, cost calculation may not be accurate because:
- Local models don't have API pricing (they're free to run)
- Model IDs may not match the tokenlens registry
- TokenLens returns `undefined` for unknown models

In these cases, the UI shows token counts but may display `$0.00` for cost.

## UI Display

### Context Component

The token usage is displayed using the **Context component** from `@ai-sdk/react`:

```tsx
<Context
  maxTokens={128000}
  modelId={sessionModelId || model}
  usage={{
    inputTokens: tokenUsage.input_tokens,
    outputTokens: tokenUsage.output_tokens,
    reasoningTokens: tokenUsage.reasoning_tokens,
  }}
  usedTokens={tokenUsage.total_tokens}
>
  <ContextTrigger />
  <ContextContent>
    <ContextContentHeader />
    <ContextContentBody>
      <ContextInputUsage />
      <ContextOutputUsage />
      <ContextReasoningUsage />
      <ContextCacheUsage />
    </ContextContentBody>
    <ContextContentFooter />
  </ContextContent>
</Context>
```

### What You See

**Compact View (always visible):**
- Circular progress indicator showing % of context used
- Token percentage (e.g., "2.5%")

**Hover Card (on hover):**
- Progress bar with exact numbers (e.g., "3.2K / 128K")
- **Input tokens** with individual cost
- **Output tokens** with individual cost
- **Reasoning tokens** (for models that support it) with cost
- **Cache tokens** (if prompt caching is used) with cost
- **Total cost** in USD at the bottom

## API Response Structure

Token usage is included in all session-related API responses:

```json
{
  "session_id": "abc-123",
  "title": "Explain async/await",
  "model_id": "gpt-4o",
  "token_usage": {
    "total_tokens": 3200,
    "input_tokens": 2800,
    "output_tokens": 400,
    "reasoning_tokens": 0,
    "cache_tokens": 0
  },
  "cost_usd": 0.011,
  "messages": [...],
  "created_at": 1707756000,
  "updated_at": 1707756120
}
```

## Accuracy and Limitations

### Accuracy
- **High accuracy** for OpenAI, Anthropic, Google, and other major providers
- Pricing updated regularly by the models.dev community
- Calculations use exact token counts from the API

### Limitations
1. **Pricing changes:** Providers update pricing frequently; tokenlens may lag
2. **Local models:** Cost calculation may not work for local/custom models
3. **Reasoning tokens:** Standard OpenAI API doesn't report reasoning tokens separately
4. **Cache tokens:** Only supported by providers with prompt caching (Anthropic)
5. **Model aliases:** Some model IDs may not resolve correctly

### Cost is Estimated
The displayed cost is an **estimate** based on:
- Community-maintained pricing data
- Token counts from the API
- Standard pricing tiers (not volume discounts)

**Always check your provider's billing dashboard for actual charges.**

## Updating Pricing Data

TokenLens automatically updates pricing data from models.dev. To get the latest:

```bash
# Update tokenlens package
cd web
npm update tokenlens
```

For the most current pricing, visit:
- **models.dev:** https://models.dev
- **Provider pricing pages** (links above)

## Future Enhancements

Planned improvements (see PLAN.md Phase 2.2):
- **Token usage analytics** - Charts and trends over time
- **Cost tracking dashboard** - Historical cost analysis
- **Budget alerts** - Warnings when approaching limits
- **Custom pricing** - Override pricing for local models
- **Volume discounts** - Support for tiered pricing

## Related Files

- `src/session.rs` - Token usage tracking logic
- `src/api.rs` - Token capture from streaming API
- `src/db.rs` - Database persistence
- `web/src/components/ai-elements/context.tsx` - UI component
- `web/src/lib/chat-api.ts` - Frontend API types
- `migrations/004_token_tracking.sql` - Database schema

## References

- **TokenLens:** https://www.npmjs.com/package/tokenlens
- **models.dev:** https://github.com/anomalyco/models.dev
- **OpenAI Pricing:** https://openai.com/pricing
- **Anthropic Pricing:** https://anthropic.com/pricing