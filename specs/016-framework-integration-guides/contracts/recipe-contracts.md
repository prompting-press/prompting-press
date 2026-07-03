# Recipe Contracts

These are the behavioral contracts each tested sample program must satisfy. They are
NOT library API — they are the guarantees of the user-side helper functions shown in
the docs. Each contract has a corresponding in-program assertion in its sample.

The library's only exposed contract here is the **unchanged** composition output:
`resolve()` → ordered `[{role, text}]`, `role ∈ {system, user, assistant}`. No new
library surface is introduced (FR-006).

## C1 — `to_langchain(messages)` (Python & TypeScript)

**Input**: composition result `[{role, text}]`.
**Output**: `[{role, content}]` where `content = text`, `role` passed through unchanged, order preserved.
**Guarantees**:
- Length and order identical to input.
- `role` value unchanged (`system`/`user`/`assistant` accepted by LangChain directly).
- No re-templating: literal `{...}` in `text` survives verbatim (contract exists *because* `ChatPromptTemplate` shorthand would break this — sample asserts a brace-containing text round-trips).
**Assertion in sample**: build a composition incl. a message whose text contains `{"k": "v"}`; assert mapped output's `content` equals the source `text` byte-for-byte and that a chat model's message-coercion accepts it (using a fake/stub model or `convert_to_messages`, no network).

## C2 — `to_strands(messages)` (Python & TypeScript)

**Input**: composition result `[{role, text}]`.
**Output**: `{ system, convo }` where
- `system` = system-role texts joined in original order with `\n\n`; `None`/`undefined` when there is no system message.
- `convo` = non-system messages mapped to `{role, content: [{text}]}`, order preserved, `role ∈ {user, assistant}`.
**Guarantees**:
- Every system-role message is removed from `convo` and folded into `system`.
- Non-system order preserved; each `text` wrapped as exactly one `{text}` block.
- No `guardContent`/`toolResult`/other blocks emitted (FR-009).
**Assertion in sample**: build a composition with `[system, system, user, assistant, user]` (TWO system messages, so the `\n\n`-join and ordering are actually exercised — G1); assert `system == "<s1>\n\n<s2>"`, `convo` has 3 entries all `user|assistant`, each `content` is `[{text: ...}]`; assert `Agent(system_prompt=system, messages=convo)` constructs without error (no `.run()`/no network). The non-leading-system-message flattening edge is additionally called out in prose (documented limitation).

## C3 — CrewAI field assignment (Python)

**Input**: individually rendered strings (each the `render(...).text` field — the RENDERED output string; NOT `Prompt.body`, which is the raw un-rendered default-variant template).
**Output**: an `Agent` and `Task` constructed with those strings in `role`/`goal`/`backstory` and `description`/`expected_output`.
**Guarantees**:
- No message array involved.
- No `crew.kickoff(inputs=...)` re-interpolation of already-rendered variables (FR-008).
**Assertion in sample**: construct `Agent(...)` + `Task(...)` from rendered strings; assert the field values equal the rendered strings (constructor only; no `kickoff()`, no network).

## Cross-cutting contract — no network, no keys

Every sample MUST construct framework objects and assert on shape only. No sample may
call a model, require an API key, or hit the network — the `docs:test-samples` gate runs
offline. (R5.)

## Cross-cutting contract — no shipped-package dependency

Framework SDK imports appear only in `docs/site/samples/{python,typescript}/examples/**`
and their project manifests. A grep/inspection of `packages/**` and `crates/**` manifests
shows zero framework deps before and after (SC-004 / FR-007).
