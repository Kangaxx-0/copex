# Spool Spec — Outstanding Questions for Dev Planning

## Document Status

- Status: Open questions collected during spec deep dive
- Date: 2026-04-06
- Parent spec: [2026-04-06-spool-refined-spec.md](./2026-04-06-spool-refined-spec.md)

---

## Connectivity and Adapters

### Q1: Fabric API surface for v1

The spec lists REST and MCP as candidate operation transports and flags this as an explicit blocker (section 11.7). For dev planning we need to know:

- Which Fabric operations does v1 actually need? (report metadata, semantic model metadata, measure definitions, DAX execution, warehouse SQL execution, artifact resolution by GUID/URL)
- Which of those are available through Fabric REST APIs today?
- Are there operations where MCP is the only viable path?

This determines whether `spool-fabric` is a pure REST client, an MCP client, or a hybrid.

### Q2: DAX query execution transport

The spec says Spool runs diagnostic DAX queries. What is the concrete execution path?

- XMLA endpoint?
- Fabric REST API?
- Some other Fabric query surface?

This affects the data-access adapter design and auth requirements.

### Q3: Warehouse SQL execution transport

Same question for read-only T-SQL against Fabric Warehouse:

- Direct SQL endpoint (TDS)?
- Fabric REST API?
- Something else?

### Q4: Auth flow details

The spec says GitHub OAuth / device-style product login plus Entra/Fabric access auth (section 11.8). Questions:

- Is product login auth actually needed for v1, or can we defer it and just do Fabric access auth?
- For Fabric access auth, is this a standard Entra ID OAuth2 device code flow?
- Does the user need separate credentials for DAX execution vs. warehouse access vs. report metadata, or is one Fabric token sufficient?

---

## Knowledge Model

### Q5: Tier 1 bundle format

The spec says `spool-index` generates Tier 1 schema knowledge from TMDL. What is the output format?

- JSON files following a defined schema?
- Markdown files like Tier 2?
- Something else?

We need this to define the bundle loading contract even if `spool-index` itself is deferred.

### Q6: Knowledge template reference repo access

Section 17.4 references `/Users/gaxx/Work/agent-console.feature-structured-knowledge-base/knowledge` as the template for Tier 2 authored knowledge. Is this accessible? We need to review it before designing the Tier 2 loading and validation contracts.

### Q7: LOB selection mechanism

The spec says "explicit LOB selection at session start" (section 7.7). How?

- CLI flag (`spool --lob finance`)?
- Config file default?
- Interactive selection from discovered bundles?
- All of the above?

---

## Session, Compaction, and Resume

### Q8: Local persistence format

The spec says full raw session history is persisted (section 12.1) but section 16 lists "exact local persistence file formats" as an open question. For dev planning:

- Is SQLite acceptable, or do we prefer flat files (JSON/MessagePack)?
- Where does session state live on disk? (`~/.spool/sessions/`?)

### Q9: Compaction trigger

The spec says Spool uses local structured compaction (section 12.3) but doesn't specify when compaction fires. Options:

- Context window pressure (like codex-rs)?
- After each task phase transition?
- Manual trigger?
- Some combination?

### Q10: Multi-task sessions

The spec says "a session may contain multiple task contracts over time, but only one is active by default in v1" (section 5.1). Can the user explicitly start a new task within the same session, or does a new task mean a new session?

---

## UX Model

### Q11: TUI framework

The spec references `spool-tui` as a crate. Is the intent to use ratatui (like codex-rs) or a different TUI framework?

### Q12: Advanced view interaction

The spec describes advanced view as exposing structured transcript, role activity, evidence ledger detail, and intermediate artifacts (section 13.3). Is this:

- A toggle within the main TUI (like a panel or tab)?
- A separate command (`spool inspect <session-id>`)?
- Both?

### Q13: Result rendering

The canonical task result is internal. What does the user actually see when a task completes?

- Natural language summary in chat?
- Formatted structured output (table/card)?
- Exportable artifact (JSON/Markdown file)?
- Some combination?

---

## LLM Provider

### Q14: Default provider and model for v1

The spec lists Azure OpenAI and Anthropic as v1 providers (section 11.5). Which is the primary target?

- Is there a preferred default model (e.g., GPT-4o, Claude Sonnet)?
- Does the evaluator subagent use the same model as the main conversation, or a different one?

### Q15: Structured output vs. tool calling

The spec says the LLM provider interface should support both structured output and tool calling (section 11.5). For the harness:

- Does the generator use tool calling to execute Fabric operations (inspect, query)?
- Does the evaluator use structured output for its response (the five outcome classes)?
- Or is this left to implementation discretion?

---

## Recipes

### Q16: Shipping recipes for v1

The spec includes one example recipe sketch (`report_number_mismatch`). How many recipes should v1 ship with, and for which investigation classes? This affects:

- How much Tier 2 content we need to author
- Whether the recipe engine needs to be fully general or can be simpler for a small initial set

---

## Cross-Cutting

### Q17: Error handling philosophy

The spec covers investigation-level failure states well (blocked, inconclusive) but doesn't say much about system-level errors:

- Fabric API timeouts or failures mid-investigation
- LLM provider errors
- Malformed bundle data at runtime

Should these surface as `blocked` task results, or is there a separate error UX?

### Q18: Telemetry and observability

No mention in the spec. Is telemetry in scope for v1, or deferred?

### Q19: Testing strategy at the spec level

The spec is testable — schemas, state rules, confidence caps, checkpoint triggers are all concrete. But is there a preferred testing philosophy?

- Unit tests for core contracts and state machines
- Integration tests with fixture bundles and mock Fabric responses
- End-to-end tests with real Fabric (probably not v1)
- Evaluator prompt regression tests

---

## Already Resolved During Review

For reference, these questions were raised and resolved during the spec deep dive:

| Question | Resolution |
|----------|-----------|
| Standalone binary or hosted? | Standalone `spool` CLI |
| Fabric workspace selection | Configuration, not interactive |
| Planner/generator/evaluator topology | Single conversation for planner+generator, evaluator as isolated subagent |
| `spool-index` sequencing | Parallel track, fixture bundles for core dev |
| Tier 2 format | Markdown |
