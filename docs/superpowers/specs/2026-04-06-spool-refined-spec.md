# Spool Refined Spec

## Document Status

- Status: Draft for review
- Date: 2026-04-06
- Type: Main product spec with implementation-facing examples
- Product: Spool

## 1. Product Definition

Spool is a Rust terminal analytics agent for Microsoft Fabric and Power BI. It is not a coding agent. Its purpose is to help users investigate report, semantic model, DAX, and warehouse-backed analytics issues through a chat-first CLI, then produce a structured, auditable result with evidence, validation, confidence, and recommended actions.

Spool is:

- a Fabric-first analytics investigation agent
- a semantic-model and report debugger
- a proposal-first assistant for BI work

Spool is not:

- a coding assistant
- a general dashboard authoring tool
- an admin/governance console
- a data correction engine

## 2. V1 Boundaries

### 2.1 In Scope

- chat-first terminal UX
- plan mode
- resumable sessions
- platform-neutral core contracts
- report investigation
- semantic model investigation
- DAX query execution and interpretation
- warehouse validation through read-only T-SQL
- structured evidence capture
- canonical structured task results for rendering, persistence, and audit
- curated LOB knowledge loading
- named reusable investigation recipes
- local context compaction

### 2.2 Out Of Scope

- direct warehouse data updates
- autonomous Fabric-side remediation
- multiple concrete platform adapters in v1 runtime
- cross-LOB investigation inside one task contract
- cross-workspace investigation inside one task contract
- raw chain-of-thought exposure
- provider compact API dependency

### 2.3 Execution Stance

V1 is proposal-first. Spool may inspect, validate, compare, classify, and propose. It does not apply Fabric-side changes directly.

Spool optimizes for earned conclusions over helpful-sounding answers. When evidence is insufficient, the product should prefer explicit uncertainty, downgrade, or continuation options rather than polished overcommitment.

### 2.4 Platform Strategy

Spool is Fabric-first in v1 but should not be Fabric-locked by architecture.

The architectural rule is:

- core task semantics must be platform-neutral from day one
- v1 may ship with Fabric as the only concrete platform adapter
- future data platforms should enter through adapter packages, not through core rewrites

This means Spool should generalize the contract layer early, while avoiding speculative lowest-common-denominator abstractions in user-facing behavior or adapter implementations.

V1 therefore targets:

- platform-neutral contracts for task execution, evidence, result states, contradiction handling, and capabilities
- Fabric-specific implementations for report inspection, semantic-model inspection, DAX validation, warehouse validation, and artifact resolution

The goal is to make future extension to other data stores or analytics platforms a bounded adapter effort rather than a harness redesign.

## 3. Core Interaction Model

### 3.1 User Vocabulary

The user should normally speak in business-artifact terms:

- report
- page
- visual
- semantic model
- measure
- warehouse
- business definition

Users may also provide stronger identifiers:

- report URL
- workspace ID
- report ID
- model ID

### 3.2 Artifact Resolution Policy

Spool normalizes all references into an internal artifact reference model.

If multiple plausible matches exist, Spool presents candidates and asks the user to choose. It does not silently pick among multiple strong candidates in v1.

### 3.3 Canonical Artifact Identity Model

Every resolved object in Spool must have a canonical internal identity, not just a display label.

The identity model exists so that:

- evidence ledger entries can point to exact artifacts
- canonical task results can be machine-readable
- resume and compaction can restore the exact investigation target
- planner, generator, and evaluator can refer to the same object without ambiguity

Every artifact identity should contain at least:

- `artifact_id`
- `artifact_type`
- `workspace_id` when applicable
- `parent_artifact_id` when applicable
- `canonical_locator`
- `display_name`
- `resolution_basis`

`artifact_id` is Spool's canonical internal ID for the artifact within the task and persisted session state.

`canonical_locator` is the normalized platform locator, such as a Fabric GUID-based path or a model-scoped object path.

`resolution_basis` records how the identity was established, such as:

- explicit GUID from user input
- report URL parsed successfully
- exact API match
- unique name match within confirmed scope
- derived child locator from resolved parent artifact

### 3.4 Artifact Identity Shapes

Canonical identity shapes in v1:

- `report`
  - workspace-scoped
  - canonical locator should include `workspace_id` and `report_id`
- `page`
  - child of report
  - canonical locator should include `workspace_id`, `report_id`, and page key
- `visual`
  - child of page
  - canonical locator should include `workspace_id`, `report_id`, page key, and visual key
- `semantic_model`
  - workspace-scoped
  - canonical locator should include `workspace_id` and `model_id`
- `measure`
  - child of semantic model
  - canonical locator should include `workspace_id`, `model_id`, table name, and measure name
- `table`
  - child of semantic model
  - canonical locator should include `workspace_id`, `model_id`, and table name
- `column`
  - child of table
  - canonical locator should include `workspace_id`, `model_id`, table name, and column name
- `relationship`
  - child of semantic model
  - canonical locator should include `workspace_id`, `model_id`, and a deterministic relationship key
- `warehouse`
  - workspace-scoped
  - canonical locator should include `workspace_id` and `warehouse_id`
- `query_result`
  - execution-scoped evidence artifact
  - canonical locator should include source kind, source artifact, execution ID, and query fingerprint

Representative examples:

```json
[
  {
    "artifact_id": "art_report_exec_rev",
    "artifact_type": "report",
    "workspace_id": "ws_123",
    "parent_artifact_id": null,
    "canonical_locator": "fabric://workspace/ws_123/report/rpt_456",
    "display_name": "Executive Revenue Report",
    "resolution_basis": "report_url"
  },
  {
    "artifact_id": "art_page_summary",
    "artifact_type": "page",
    "workspace_id": "ws_123",
    "parent_artifact_id": "art_report_exec_rev",
    "canonical_locator": "fabric://workspace/ws_123/report/rpt_456/page/ReportSectionSummary",
    "display_name": "Summary",
    "resolution_basis": "derived_from_resolved_report"
  },
  {
    "artifact_id": "art_measure_qoq_revenue",
    "artifact_type": "measure",
    "workspace_id": "ws_123",
    "parent_artifact_id": "art_model_sales",
    "canonical_locator": "fabric://workspace/ws_123/model/mod_789/measure/Sales[QoQ Revenue]",
    "display_name": "QoQ Revenue",
    "resolution_basis": "exact_api_match"
  },
  {
    "artifact_id": "art_query_dax_001",
    "artifact_type": "query_result",
    "workspace_id": "ws_123",
    "parent_artifact_id": "art_measure_qoq_revenue",
    "canonical_locator": "spool://query-result/dax/exec_001/fingerprint_ab12",
    "display_name": "Diagnostic DAX result for QoQ Revenue",
    "resolution_basis": "runtime_execution"
  }
]
```

### 3.5 Resolution And Fallback Rules

Resolution should prefer the strongest identity source available:

1. explicit GUID-bearing user input
2. parsed report URL or artifact URL
3. exact platform/API resolution within confirmed workspace scope
4. unique scoped name resolution
5. derived child identity from an already resolved parent

If Spool cannot produce a sufficiently specific canonical identity, it must ask the user rather than silently continue with a weak match.

If a child artifact has no stable Fabric GUID, v1 may use a deterministic parent-scoped locator, such as:

- page key within report
- visual key within page
- `table_name + measure_name` within semantic model
- `table_name + column_name` within semantic model
- deterministic relationship key within semantic model

### 3.6 Query Result Handling

Spool treats DAX query results and warehouse query results as analytical evidence, not as default chat payloads to dump in full.

Default behavior:

- summarize results in natural language
- highlight key rows, aggregates, anomalies, or mismatches
- attach evidence references

If a query returns a large result set, Spool:

- summarizes by default
- shows only a bounded preview when useful
- allows explicit raw-output drill-down on request

## 4. Harness Model

### 4.1 Roles

Spool uses a planner, generator, evaluator model.

- planner
  - clarifies scope
  - resolves ambiguity
  - selects artifacts, recipes, and validation expectations
  - produces the task contract
- generator
  - executes investigation
  - collects evidence
  - runs DAX and warehouse validations
  - drafts findings and proposed actions
- evaluator
  - independently checks whether the conclusion is supported
  - reviews evidence quality and confidence calibration
  - classifies the result state

### 4.2 Evaluator Isolation

The evaluator operates from a bounded evaluator packet rather than unrestricted generator context.

A likely v1 implementation is a subagent callout with:

- dedicated evaluator prompt
- bounded evaluator packet
- separate model pass

### 4.3 Evaluator Loop Protocol

The evaluator loop must follow a formal contract in v1.

The evaluator may return only these outcome classes:

- `accept`
- `request_more_evidence`
- `downgrade`
- `blocked`
- `contradiction`

Meaning:

- `accept`
  - current evidence is sufficient for final classification
- `request_more_evidence`
  - current evidence is insufficient, and evaluator identifies one or more specific missing evidence targets
- `downgrade`
  - current claim is too strong for the evidence currently available
- `blocked`
  - the task cannot continue meaningfully because required evidence cannot be obtained or policy prevents progress
- `contradiction`
  - material conflict exists between evidence sources and must be surfaced before finalization

### 4.4 Generator Obligations In The Evaluator Loop

If the evaluator returns `request_more_evidence`, the generator must either:

- collect the requested evidence target, or
- return an explicit reason why the target cannot be collected

Allowed inability reasons include:

- missing access
- missing user clarification
- unavailable artifact
- out-of-scope request
- policy boundary

### 4.5 New Evidence Target Definition

A valid evaluator-requested evidence target must be specific and actionable.

Valid examples:

- inspect the report visual filter context
- run a DAX query scoped to the disputed measure
- run a warehouse validation query at the same date grain
- retrieve the measure definition for the backing semantic-model object

Invalid examples:

- investigate more
- get stronger evidence
- double check the model

### 4.6 Loop Exhaustion

The evaluator/generator loop must be bounded.

If the loop exhausts without reaching `accept`, the final result must:

- surface the evaluator concern
- record what additional evidence was requested
- record what was attempted
- explain why the task stopped

After loop exhaustion, the final state may be:

- `supported_hypothesis`
- `inconclusive`
- `blocked`

After loop exhaustion, the final state must not be `confirmed`.

If loop exhaustion ends with unresolved disagreement between the generator's leading conclusion and the evaluator's final classification, the normal user-facing answer must surface:

- the generator's leading conclusion
- the evaluator's objection, downgrade, or contradiction reason
- the remaining evidence gap or blocking condition

## 5. Task Contract Model

### 5.1 Primary Work Unit

The task contract is the primary work unit. A session may contain multiple task contracts over time, but only one is active by default in v1.

### 5.2 Required Fields

A task contract should define at least:

- interpreted user request
- selected LOB bundle
- selected workspace scope
- target artifacts
- selected recipe, if any
- explicit assumptions
- expected evidence classes
- validation floor
- clarification checkpoints
- approval checkpoints
- expected deliverable shape
- evaluator packet requirements

### 5.3 Canonical Task Contract Schema

Spool must maintain a canonical task-contract object for every active or completed task.

The task contract is the authoritative planning contract used by:

- planner output
- generator scope control
- evaluator packet expectations
- resume and compaction
- advanced view

The top-level task-contract object should contain at least:

- `task_id`
- `intent`
- `scope`
- `selected_recipe`
- `assumptions`
- `expected_evidence_classes`
- `validation_floor`
- `checkpoint_policy`
- `clarification_checkpoints`
- `approval_checkpoints`
- `expected_deliverable_shape`
- `evaluator_packet_requirements`

Minimum child object expectations:

- `scope`
  - `lob`
  - `workspace`
  - `artifacts`
- `scope.artifacts[]`
  - `type`
  - `ref`
- `checkpoint_policy`
  - `ask_on`

Optional but recommended fields:

- `task_status`
- `selected_recipe_selection_mode`
- `selected_recipe_rationale`
- `selected_recipe_user_preference`
- `created_at`
- `updated_at`

The canonical task contract is a stable internal contract for v1. It is not a loose planning note.

### 5.4 Example Task Contract

```json
{
  "task_id": "task_123",
  "scope": {
    "lob": "finance",
    "workspace": "Executive BI",
    "artifacts": [
      {"type": "report", "ref": "Executive Revenue Report"},
      {"type": "measure", "ref": "Sales Model.Revenue"}
    ]
  },
  "intent": "Find why the report revenue number does not match expected quarter totals.",
  "selected_recipe": "report_number_mismatch",
  "selected_recipe_selection_mode": "auto_select",
  "selected_recipe_rationale": "Strongest fit for a report-versus-model mismatch investigation with direct validation required.",
  "assumptions": [
    "The user is referring to the published report in Executive BI."
  ],
  "expected_evidence_classes": [
    "report_metadata",
    "measure_definition",
    "dax_query_result",
    "warehouse_query_result"
  ],
  "validation_floor": "direct_validation_required",
  "checkpoint_policy": {
    "ask_on": ["ambiguous", "scope_expanding", "expectation_shaping", "side_effecting"]
  }
}
```

### 5.5 Completion

A task is complete when Spool has produced a terminal structured result and has either:

- resolved the request, or
- clearly handed back the remaining human action with no further in-product investigation step still pending

Completion does not require the business fix to already be applied in Fabric.

If Spool is still waiting on:

- user clarification
- user input to an `AskUserQuestion`
- approval to continue
- a user decision that would allow the investigation to proceed

then the task is not complete yet, even if Spool can already summarize the current partial understanding.

“Clearly handed back the remaining human action” is terminal only when Spool has no further active investigation step to perform after the handoff. External follow-through may still remain, such as applying a business fix in Fabric, but unresolved in-product continuation must not be labeled complete.

### 5.6 Live Task Status And Pending Interaction

Spool distinguishes:

- terminal result state
- live task or session status

Result state answers “what did the investigation conclude?”

Live status answers “is the investigation still active, interrupted, or waiting on someone?”

At minimum, the live status model in v1 must be able to distinguish:

- active work in progress
- waiting on approval
- waiting on user input
- interrupted work
- completed work

Waiting on approval or waiting on user input is not a completed state. These are active blocked states that must survive compaction and resume.

## 6. AskUserQuestion And Checkpoint Policy

### 6.1 AskUserQuestion

Spool includes a first-class `AskUserQuestion` mechanism.

It is used for:

- artifact disambiguation
- scope clarification
- interpretation selection
- missing-information recovery
- approval checkpoints

Rules:

- one question at a time by default
- multiple-choice preferred
- free-text allowed when necessary
- Q&A is persisted into task history
- an unanswered question keeps the task active rather than completed

### 6.2 Checkpoint Policy

Spool interrupts only when the next step is:

- ambiguous
- scope-expanding
- expectation-shaping
- side-effecting

Spool does not interrupt merely because a validation step is computationally heavy or broad if it is already clearly in scope.

### 6.3 Checkpoint Classes

- information checkpoints
  - ambiguity, missing identifiers, unclear intent
- investigation checkpoints
  - steps that would expand beyond confirmed scope or rely on risky assumptions
- action checkpoints
  - side-effecting steps or anything beyond the v1 proposal-first boundary

### 6.4 Checkpoint Examples By Class

Representative checkpoint examples in v1:

- information checkpoints
  - ask when the user names a report but multiple strong matches exist in the confirmed workspace scope
  - ask when the user says a number is wrong but has not identified which page, visual, model object, or business metric is in dispute
- investigation checkpoints
  - ask when the next step would expand from the confirmed report into a different report, semantic model, or workspace
  - ask when the investigation would need to assume a business interpretation that is not supported by the selected LOB knowledge or explicit user instruction
- action checkpoints
  - ask before any side-effecting action in future versions
  - ask before shifting the task from diagnosis into proposing or drafting a remediation package when that expectation has not yet been set

Representative no-checkpoint examples in v1:

- do not ask before inspecting report or semantic-model metadata needed for the current confirmed task
- do not ask before running an in-scope DAX validation query
- do not ask before running an in-scope read-only warehouse validation query
- do not ask merely because a validation step is broad or computationally heavy when it remains clearly inside the confirmed task scope

### 6.5 Continuation Offer Policy For Insufficient Evidence

When evidence is insufficient for `confirmed`, but further investigation is still possible, Spool should not only emit an unresolved result and stop. It should offer a structured continuation path.

This policy applies when at least one of the following is true:

- the evaluator requests more evidence and the system is not continuing automatically
- loop exhaustion occurs but actionable next investigation steps still exist
- the current result is `supported_hypothesis` or `inconclusive` and the system can identify one or more realistic next evidence targets

In these situations, the normal answer should include:

- the current best explanation or leading conclusion
- what evidence was gathered so far
- what evidence is still missing
- why the current result cannot be `confirmed`
- a short menu of suggested next investigation options
- a free-text path so the user can choose a different next step

Suggested next-step options should normally be limited to one to three items and should include trade-off language when useful.

Representative option classes:

- gather missing artifact detail
- run broader or deeper validation
- expand scope with user confirmation
- stop and keep the current unresolved result

If no realistic next investigation step exists, Spool should not fabricate continuation options. In that case it should emit the unresolved result with explicit blocker or limitation details only.

## 7. Knowledge Model

### 7.1 Two-Tier Knowledge Architecture

Spool uses two distinct tiers:

- Tier 1: auto-generated schema knowledge from Fabric semantic model definitions
- Tier 2: hand-authored business knowledge curated by the team

Tier 1 provides structural understanding. Tier 2 provides business meaning, rules, patterns, and recipes.

### 7.2 Knowledge Bundle Composition

Each selected LOB bundle must follow a formal composition contract in v1.

The bundle is the unit that Spool loads into the session for that LOB. It is not an unstructured folder of notes.

Each bundle should contain:

- one bundle manifest
- one Tier 1 schema package
- one Tier 2 curated knowledge package
- bundle validation metadata

At the conceptual level:

```text
lob_bundle/
  manifest
  tier1/
    schema index artifacts
  tier2/
    contexts/
    metrics/
    rules/
    patterns/
    recipes/
  validation/
    bundle validation outputs
```

### 7.3 Bundle Manifest

Each LOB bundle manifest should define at least:

- `bundle_id`
- `lob_id`
- `version`
- `display_name`
- `default_workspace_scope` when applicable
- `tier1_schema_version`
- `tier2_bundle_version`
- `build_timestamp`
- `source_summary`
- `declared_artifact_classes`
- `declared_recipe_ids`

The manifest is the authoritative index for what the bundle contains.

### 7.4 Tier 1 Schema Package

Tier 1 is produced by `spool-index` from Fabric semantic model definitions.

The Tier 1 package should contain compact structural knowledge for at least:

- semantic models
- tables
- measures
- relationships
- key business-facing aliases captured from source metadata when available

Tier 1 is structural, not interpretive. It should not contain hand-authored business policy unless that policy is explicitly part of Tier 2.

### 7.5 Tier 2 Curated Knowledge Package

Tier 2 contains human-authored business knowledge for the selected LOB.

The Tier 2 package should use structured authored artifacts aligned with the existing generic knowledge-template approach.

Worked guidance and examples belong inside the Tier 2 artifacts that own them. v1 should not introduce a separate top-level examples package or a separate examples prompt section.

Supported Tier 2 artifact classes in v1:

- `context`
- `metric`
- `rule`
- `pattern`
- `recipe`

Minimum expectations by class:

- `context`
  - defines a business concept, relationships, and usage notes
- `metric`
  - defines business meaning, aliases, linked semantic-model measures where applicable, and may include metric-specific worked examples
- `rule`
  - defines decision logic or interpretation logic
- `pattern`
  - defines a reusable analytical approach, anti-patterns, and may include worked examples
- `recipe`
  - defines an investigation playbook tied to evidence classes, validation expectations, and worked examples

### 7.6 Bundle Naming And Reference Rules

Knowledge artifacts inside a bundle must use stable bundle-local IDs.

Reference rules:

- every Tier 2 artifact must have a unique ID within the bundle
- aliases may overlap only if disambiguation rules exist
- metric-to-measure references must resolve against Tier 1 or explicitly declare an unresolved state
- rules, patterns, and recipes may reference other bundle artifacts only through stable IDs

Spool must not rely only on display names to link knowledge artifacts.

### 7.7 Selected-LOB Loading Policy

The default v1 behavior is explicit LOB selection at session start.

In v1:

- Spool loads the selected LOB bundle only
- Spool does not load all LOB bundles
- the selected LOB’s Tier 1 and Tier 2 are loaded in full
- the prompt is composed from that selected LOB bundle

### 7.7.1 Knowledge Projection And Prompt Composition

Spool should compose the main agent prompt from a single selected-LOB knowledge projection rather than from multiple parallel knowledge sections.

Prompt-composition rules in v1:

- the selected LOB bundle remains the only curated knowledge source loaded into the session
- the main prompt may render knowledge from contexts, metrics, rules, patterns, and recipes into one composed knowledge block
- examples remain owned by their source artifacts inside the bundle and are rendered through that same composed knowledge block
- v1 may surface examples from metric-level, pattern-level, or recipe-level authored artifacts when useful
- Spool must not create a separate top-level examples prompt section that can drift from the bundle itself

### 7.8 Cold-Start Structural Awareness

Spool must remain useful when hand-authored business knowledge is absent or incomplete.

This is achieved through a schema indexer companion component, `spool-index`, which builds Tier 1 schema knowledge from Fabric semantic model definitions.

### 7.9 Knowledge Build Pipeline

`spool-index` should:

- fetch semantic model definitions from Fabric
- parse TMDL or equivalent model definitions
- generate Tier 1 schema knowledge
- validate bundle integrity

### 7.10 Knowledge Validation

Bundle validation should run across both tiers, not just within isolated files.

Knowledge validation should include at least:

- invalid or missing measure references
- alias collisions
- malformed bundle structure
- incomplete generated schema artifacts
- broken cross-artifact references
- duplicate bundle-local IDs
- recipe references to missing evidence classes or missing artifact classes

### 7.11 Fallback Behavior

If pre-built schema knowledge is missing, Spool should degrade gracefully:

- warn that pre-built schema knowledge is missing
- construct transient runtime schema context if possible
- continue with reduced knowledge quality rather than hard fail

If Tier 2 curated knowledge is missing for a selected LOB, Spool should still run with Tier 1 only and mark business-knowledge coverage as reduced.

### 7.12 Durable Memory

Durable memory is a separate subsystem from curated knowledge.

Curated knowledge is selected-LOB domain truth and investigation guidance. Durable memory is explicit persisted operating context that helps Spool work efficiently across sessions without redefining domain truth.

In v1, durable memory is not an auto-learning subsystem. It should be created, edited, imported, or removed only through explicit human-managed workflows.

Durable memory may guide planning and investigation, but it must not silently override fresh runtime evidence or curated bundle knowledge.

Allowed durable-memory classes in v1:

- user preferences
- team conventions
- recurring issue patterns
- reference sources
- artifact relationship hints
- stable validation recipes

Durable memory handling rules:

- durable memory should be treated as potentially stale until revalidated against current runtime state
- durable memory may accelerate artifact resolution or investigation direction
- durable memory alone must not justify a `confirmed` result
- when durable memory conflicts with fresh runtime evidence, runtime evidence wins
- when durable memory materially affects investigation behavior, that influence should remain inspectable in advanced view or trace

### 7.13 Durable Memory Management

Durable memory in v1 should use explicit management rather than automatic learning.

Management rules:

- Spool must not silently persist new durable-memory entries from completed tasks, evaluator results, or session transcripts in v1
- useful observations discovered during a task may be surfaced in the current task trace or export as candidate follow-up notes, but they must not become durable memory without explicit human action
- durable memory entries should remain inspectable and attributable to a source such as user-managed file content, team-managed file content, or explicit product authoring flow
- the v1 source of truth for durable memory should be configured local managed storage such as user-level or team-managed memory files loaded at session start
- if Spool supports import or replacement flows in v1, those flows must be explicit operator actions rather than side effects of normal task execution

Reuse rules:

- durable memory may be consulted during planning, artifact resolution, and investigation acceleration
- reuse should respect scope
- durable memory should not be treated as universally global unless explicitly marked as such
- when durable memory is consulted, the task trace should record that the step was memory-guided rather than evidence-backed

Scope rules:

- durable memory should carry at least one scope marker such as `lob`, `workspace`, `team`, or `global`
- LOB-scoped memory should not be silently applied across different LOBs
- workspace-specific memory should not be silently treated as globally valid

Review and staleness rules:

- durable memory should be reviewable and may be marked stale or disabled when fresh runtime evidence contradicts it
- stale or disabled memory should remain inspectable in trace or memory history rather than disappearing without explanation
- memory may also be marked stale when the underlying artifact model, business process, or curated knowledge bundle changes materially
- contradiction detected during a task must not silently rewrite stored durable memory during that same task

Minimum metadata for a durable-memory entry in v1 should include:

- `memory_id`
- `memory_type`
- `scope`
- `created_at`
- `updated_at` when available
- `last_validated_at` when available
- `source_basis`
- `status`

Recommended statuses:

- `active`
- `stale`
- `disabled`

## 8. Investigation Recipes

### 8.1 Definition

Recipes are structured authored artifacts inside the curated knowledge bundle. They are not loose prose.

### 8.2 Recipe Shape

Recipes should define at least:

- name
- intent or problem class
- applicability conditions
- required inputs
- recommended artifact targets
- expected evidence classes
- validation expectations
- ordered investigation flow
- anti-patterns
- worked examples

Recipes should follow the generic knowledge-template approach already used for authored business guidance.

Recipe examples remain part of the recipe artifact. They should not be split into a separate prompt appendix or an independent examples lane.

### 8.3 Separation From Schema Source

Recipes are part of the curated knowledge layer. They must not be defined in terms of legacy schema-authoring patterns.

Schema structure comes from Fabric semantic model definitions and TMDL-derived artifacts. If Spool later needs additional authored schema, that should be a Spool-specific schema rather than a carryover from an earlier system.

### 8.4 Recipe Invocation

Recipes are primarily planner-selected in v1. Users are not expected to know the internal recipe catalog, recipe IDs, or recipe names.

Recipe choice may enter the task through:

- planner auto-selection based on the interpreted request and current task context
- planner suggestion surfaced in the user-facing task framing
- user natural-language preference about how to investigate, such as preferring a warehouse-first comparison or a familiar mismatch playbook
- explicit user reference to a known playbook or recipe when the user happens to know one

A user-provided recipe or playbook reference is a preference signal, not a blind command. Spool should interpret it through the planner rather than through a slash command or a separate recipe-selection mode.

### 8.5 Recipe Selection Policy

Recipe selection must be explicit and task-contract-aware in v1.

The planner should evaluate recipe fit using at least:

- problem-class match
- artifact availability
- expected evidence-class fit
- validation-fit with the task's required floor
- applicability to the selected LOB and workspace scope

Planner selection outcomes:

- `auto_select`
  - use when one recipe is a strong fit and no competing recipe is similarly suitable
- `suggest`
  - use when a recipe is helpful but not mandatory, or when multiple plausible recipes exist
- `do_not_use`
  - use when no available recipe fits the task well enough
- `user_requested_override`
  - use when the user expresses a specific investigation preference or known playbook and the planner maps that preference to a compatible recipe

Policy rules:

- if a recipe is selected, the task contract should record it explicitly
- if a recipe is only suggested, the user-facing plan or task framing should make that visible
- the planner should own recipe discovery by default; users should be able to describe the problem without knowing internal recipe details
- user-facing plan or task framing should describe the selected approach in task language rather than requiring internal recipe IDs or taxonomy
- the planner should not force a weakly matched recipe merely because one exists
- the planner may accept a user-requested recipe that is not a perfect fit when it remains compatible enough with the task scope, artifact set, expected evidence classes, and validation needs
- when a partial-fit user-requested recipe is accepted, later execution may deviate from the recipe and must record that deviation explicitly
- the planner may refuse a user-requested recipe when it is incompatible with the task scope, artifact set, or validation needs
- if the planner refuses a user-requested recipe, it should explain why and propose the closest viable alternative when one exists

### 8.6 Deviation Recording

If the generator deviates from a selected recipe, including an accepted partial-fit user preference, the task record should capture:

- which step was skipped, changed, or reordered
- why
- what evidence justified the deviation
- whether confidence changed because of the deviation

### 8.7 Example Recipe Sketch

```yaml
name: report_number_mismatch
intent: Diagnose why a report visual does not match expected totals
applies_when:
  - report number is disputed
  - semantic model and warehouse validation may both be relevant
expected_evidence:
  - report_metadata
  - measure_definition
  - dax_query_result
  - warehouse_query_result
validation_expectations:
  - compare report output against at least one direct query-based validation
steps:
  - resolve report, page, and visual
  - identify backing semantic model object
  - inspect relevant measure definitions and filters
  - run diagnostic DAX query
  - run warehouse validation query when appropriate
  - classify likely mismatch source
anti_patterns:
  - assume warehouse mismatch before checking measure logic
  - rely on report screenshot alone
```

## 9. Evidence And Validation

### 9.1 Evidence Ledger

Each task owns an append-only evidence ledger.

Evidence types should distinguish:

- observed
- derived
- proposed

### 9.2 Truth Hierarchy

Spool must use an explicit truth hierarchy when evidence sources, knowledge sources, and user assertions differ.

This hierarchy exists to keep planner, generator, and evaluator behavior consistent and to prevent lower-trust guidance from being treated as confirmed truth.

The default precedence order in v1 is:

1. runtime observed evidence from the exact target artifact or direct validation query
2. current Fabric or semantic-model metadata for the exact target artifact
3. curated LOB knowledge for the selected bundle
4. durable memory from prior sessions
5. recipe guidance
6. user assertions that have not yet been validated

Interpretation rules:

- higher-precedence sources normally outweigh lower-precedence sources for factual claims
- lower-precedence sources may guide investigation, but they do not by themselves justify a `confirmed` result against stronger conflicting runtime evidence
- user assertions define intent, scope, and expectations, but they do not become validated truth until checked
- durable memory may accelerate investigation, but it must be treated as potentially stale unless revalidated against current runtime state

This hierarchy is a default rule, not a simplistic universal override rule.

Exceptions by claim class:

- business meaning claims
  - curated LOB knowledge may outrank raw technical naming when the question is about intended business interpretation
- current implementation claims
  - live model metadata or direct runtime inspection outrank authored knowledge and memory
- historical investigation context
  - durable memory may outrank recipe defaults when it records a known workspace-specific pattern, but it still does not outrank fresh contradictory runtime evidence

When a lower-precedence source remains materially important, it should not be discarded. It should be preserved as context or surfaced as a contradiction if it meaningfully conflicts with higher-precedence evidence.

### 9.3 Freshness Policy

Spool must treat freshness as part of evidence semantics, not only as metadata decoration.

Freshness matters because report definitions, semantic models, warehouse data, curated knowledge, and prior-session memory can all drift over time.

When available, evidence items should carry freshness context such as:

- `observed_at`
- artifact publish or version marker
- data refresh marker
- bundle version
- memory capture timestamp

Freshness should be interpreted by evidence class:

- runtime query evidence
  - may become stale after relevant data refresh, scope change, or artifact republish
- report or semantic-model metadata
  - may become stale after republish, schema change, or artifact replacement
- curated knowledge
  - should be versioned and treated as potentially stale when the underlying semantic model or business process has changed
- durable memory
  - is useful for guidance and acceleration but must be treated as stale-by-default until revalidated against current runtime state

Freshness rules for v1:

- stale evidence may guide investigation, but it should weaken confidence
- stale evidence alone must not justify a `confirmed` result
- if a current-state claim relies on evidence collected before a known refresh, republish, or schema change, revalidation is required before `confirmed`
- if freshness cannot be determined for material evidence, confidence should be reduced
- when multiple evidence items disagree and one is known to be fresher, freshness should be considered explicitly in contradiction handling

The main spec does not require one universal TTL. Freshness should be evaluated by evidence class and by the type of claim being made.

### 9.4 Contradiction Handling

Spool must treat contradiction as a first-class investigation outcome, not as an incidental note.

A contradiction exists when materially relevant sources support incompatible interpretations of the same claim, metric, artifact state, or comparison.

Contradictions should be recorded explicitly in task state.

The contradiction ledger is the authoritative source of truth for contradiction state in v1.

Other system artifacts may include contradiction summaries or references, but they must not become independent contradiction authorities.

Each contradiction record should capture at least:

- the disputed claim or comparison
- the conflicting evidence sources
- whether the contradiction is material or non-material
- freshness notes when relevant
- whether resolution was attempted
- current status: `open`, `resolved`, or `carried_forward`

Contradiction workflow in v1:

1. detect the conflict
2. determine whether it is material to the task's main claim or recommendation
3. attempt like-for-like comparison, scope alignment, and freshness review where possible
4. attempt targeted revalidation when the contradiction is material and resolvable
5. classify the contradiction as resolved, downgraded in significance, or unresolved

Materiality rules:

- a contradiction is material when it can change the leading conclusion, final recommendation, confidence level, or result state
- a contradiction is non-material when it affects only peripheral context and does not change the main answer

Resolution rules:

- a resolved contradiction should remain in task trace, but it should no longer cap the final result
- an unresolved material contradiction must prevent `confirmed`
- an unresolved material contradiction should normally reduce confidence
- when a contradiction cannot be resolved because of access, missing identifiers, or policy limits, that limitation should be surfaced explicitly in the final result

Contradiction handling should use the truth hierarchy and freshness policy together. A higher-precedence or fresher source may explain why a contradiction is downgraded or resolved, but the contradiction should still be recorded rather than silently erased.

### 9.4.1 Contradiction Ownership And Projection Rules

Contradictions may appear in multiple product surfaces, but ownership must remain singular.

Authoritative ownership:

- the task-scoped contradiction ledger is authoritative

Derived projections:

- canonical task results may include contradiction snapshots or references
- advanced view may render contradiction history and summaries
- compaction summaries may include contradiction summaries
- resume state may include contradiction summaries needed for restoration

Projection rules:

- a canonical task result must not invent contradiction state that is absent from the contradiction ledger
- if a contradiction appears in the task result, it should be a projection, summary, or reference to ledger-owned contradiction records
- compaction and resume should restore contradiction state from the contradiction ledger first, then use result snapshots only as supporting summaries
- if a contradiction snapshot in a prior task result conflicts with the contradiction ledger, the contradiction ledger wins

This rule exists to prevent contradiction drift between evidence state, task results, compaction outputs, and resumed sessions.

### 9.5 Intermediate Artifacts

Meaningful intermediate artifacts are persisted, including:

- candidate hypotheses
- rejected hypotheses
- key failed validation attempts
- materially relevant abandoned branches

These are hidden by default and surfaced in advanced view.

### 9.6 Validation Floor

Every recommendation must include at least one observed evidence item.

Every non-trivial recommendation must include at least one direct validation step tied to:

- relevant artifact
- DAX query result
- warehouse result

### 9.7 Minimum Validation Patterns By Investigation Class

The minimum validation floor should be interpreted by investigation class, not only as a generic policy statement.

Minimum patterns for v1:

- report number mismatch
  - inspect the target report artifact
  - inspect the relevant semantic-model measure or logic
  - run at least one direct query-based validation
- measure logic review
  - inspect the measure definition
  - inspect dependencies or referenced objects when relevant
  - run at least one validating query using the measure
- warehouse-vs-model disagreement
  - run warehouse-side validation
  - run model-side or DAX-side validation
  - compare like-for-like scope and filters
- metadata or artifact-resolution investigation
  - inspect direct metadata
  - do not assign a result state above `inconclusive` unless supported by additional evidence beyond metadata alone

These are minimums, not ceilings. The evaluator may require stronger validation when the claim or recommendation is higher risk.

### 9.8 Higher-Risk Validation

Higher-risk conclusions should use stronger cross-checks such as:

- artifact metadata inspection
- DAX query comparison
- warehouse result comparison
- business-definition comparison from knowledge
- contradiction checks across sources

### 9.9 Example Evidence Items

```json
[
  {
    "id": "ev_12",
    "type": "observed",
    "source": "dax_query_result",
    "summary": "Diagnostic DAX query returned 12.4M for Q1 revenue"
  },
  {
    "id": "ev_19",
    "type": "observed",
    "source": "warehouse_query_result",
    "summary": "Warehouse validation returned 12.4M for Q1 revenue"
  },
  {
    "id": "ev_23",
    "type": "derived",
    "source": "comparison",
    "summary": "Report visual mismatch is likely caused by semantic-model logic rather than warehouse data"
  }
]
```

## 10. Result Model

### 10.1 Structured Result

The authoritative output is a structured task result.

It should contain at least:

- `task_id`
- `state`
- `confidence`
- `summary`
- `findings`
- `evidence_refs`
- `validation_results`
- `recommended_actions`
- `blockers`
- `open_questions`
- `proposed_changes`
- `contradictions`

### 10.2 Result States

These are terminal result states for the canonical task result. They are not live task-status values.

- `confirmed`
- `supported_hypothesis`
- `inconclusive`
- `blocked`

### 10.3 Result-State Semantics

Result states are evaluator-owned in the canonical task result. The generator may propose a provisional state, but only the evaluator may assign the final state.

They do not replace live task status. A task may remain active, interrupted, or waiting on user or approval before any canonical final result exists.

State definitions:

- `confirmed`
  - the main claim is directly supported by sufficient observed evidence
  - required validation has passed
  - no unresolved contradiction remains material to the conclusion
- `supported_hypothesis`
  - the leading explanation is well-supported
  - at least one important uncertainty, missing validation, or unresolved alternative remains
- `inconclusive`
  - evidence is insufficient to prefer one explanation confidently over others
  - or validation is too weak to justify the leading explanation
- `blocked`
  - progress cannot continue meaningfully because of missing access, missing identifiers, missing user clarification, policy boundary, or unavailable required evidence

### 10.4 Minimum Answer Shape For Unresolved Tasks

Unresolved task outcomes are still first-class product outputs in Spool. They must not collapse into vague failure text.

Any final result with state `supported_hypothesis`, `inconclusive`, or `blocked` must include at least:

- the current best answer or leading explanation
- what evidence was gathered
- what remains unresolved
- why the task stopped short of `confirmed`
- what next evidence, clarification, or human action would move the task forward

Additional minimums by state:

- `supported_hypothesis`
  - must state what specifically prevented `confirmed`
- `inconclusive`
  - must state why the evidence could not distinguish between the leading possibilities
  - should name the main competing explanations when known
- `blocked`
  - must identify the concrete blocker
  - must state whether the blocker is access-related, scope-related, artifact-related, clarification-related, or policy-related

This minimum answer shape applies whether the result came from a normal stop, evaluator downgrade, or loop exhaustion.

If the generator's proposed state or leading conclusion materially differs from the evaluator's canonical final state, the normal answer must surface that disagreement explicitly rather than hiding it in advanced view only.

### 10.5 State Assignment Authority

Authority rules:

- planner
  - does not assign final result state
- generator
  - may emit a `proposed_state`
  - may not finalize or upgrade the result state
- evaluator
  - assigns the canonical final `state`
  - may confirm, preserve, or downgrade the generator’s proposed state

If the generator and evaluator disagree, the evaluator’s state wins for the canonical task result. The generator’s proposed state may still be preserved in internal trace or advanced view.

### 10.6 Confidence Levels

- `high`
- `medium`
- `low`

### 10.7 Confidence Calibration

Confidence is evaluator-owned in the canonical task result, just like final result state.

Confidence must reflect evidence quality, validation coverage, contradiction status, and freshness. It must not be a stylistic label or a proxy for model assertiveness.

Calibration guidance for v1:

- `high`
  - the leading conclusion is supported by direct observed evidence
  - required validation coverage is strong for the investigation class
  - no material contradiction remains unresolved
  - evidence freshness is acceptable for the claim being made
- `medium`
  - the leading conclusion is meaningfully supported
  - at least one important uncertainty, validation gap, or scope limitation remains
  - the conclusion is still the best supported explanation among known alternatives
- `low`
  - support is weak, indirect, stale, narrow, or materially incomplete
  - important alternatives remain unresolved
  - contradiction pressure or missing validation materially limits confidence

Confidence downgrade pressures include:

- unresolved contradiction
- stale evidence
- reliance on indirect or derived evidence without enough direct validation
- missing like-for-like comparison across report, model, and warehouse scopes
- missing artifact resolution certainty
- heavy dependence on memory or authored knowledge without current runtime confirmation

Confidence caps in v1:

- a result should not be `high` confidence when a material contradiction remains unresolved
- a metadata-only investigation should not exceed `medium` confidence
- a result state of `inconclusive` should not carry `high` confidence
- a result state of `blocked` should normally carry `low` confidence unless the blocker itself is directly confirmed and narrowly scoped

### 10.8 Canonical Task Result Schema

Spool must maintain a canonical structured task-result object even in a terminal-first product.

V1 does not promise a stable compatibility contract for exported JSON or Markdown artifacts.

The canonical task result is the authoritative internal result contract used by:

- terminal rendering
- advanced view
- persisted task state
- evaluator output handoff
- debugging and audit review

If JSON or Markdown artifacts are emitted in v1, they may be surfaced as user-facing convenience outputs, but they should be treated as non-stable product formats unless and until a separate export contract defines them.

The canonical task result is a terminal investigation artifact. Pending approval requests, pending user-input requests, and other live waiting semantics belong to persisted session or task state rather than to the canonical final result object.

The top-level task-result object should contain at least:

- `task_id`
- `state`
- `confidence`
- `summary`
- `findings`
- `evidence_refs`
- `validation_results`
- `recommended_actions`
- `blockers`
- `open_questions`
- `proposed_changes`
- `contradiction_refs`

Optional but recommended fields:

- `proposed_state`
- `result_generated_at`
- `result_version`

Minimum child object expectations:

- `findings[]`
  - `id`
  - `title`
  - `detail`
- `validation_results[]`
  - `id`
  - `type`
  - `status`
  - `detail`
- `recommended_actions[]`
  - `id`
  - `type`
  - `summary`
- `blockers[]`
  - `id`
  - `type`
  - `summary`
- `open_questions[]`
  - `id`
  - `question`
- `proposed_changes[]`
  - `id`
  - `artifact_type`
  - `artifact_ref`
  - `change_summary`
- `contradiction_refs[]`
  - contradiction ledger record IDs or compact contradiction summary references suitable for rendering

The schema should distinguish required fields from optional enrichments in implementation, but the above shape is the minimum governing contract for v1.

### 10.9 Example Canonical Task Result

```json
{
  "task_id": "task_123",
  "proposed_state": "confirmed",
  "state": "supported_hypothesis",
  "confidence": "medium",
  "summary": "The executive report is likely using a measure definition that no longer matches warehouse-backed business logic.",
  "findings": [
    {
      "id": "finding_1",
      "title": "Revenue variance traced to semantic-model measure logic",
      "detail": "The report measure and validation queries disagree on quarter-over-quarter handling."
    }
  ],
  "evidence_refs": ["ev_12", "ev_19", "ev_23"],
  "validation_results": [
    {
      "id": "val_1",
      "type": "dax_and_warehouse_comparison",
      "status": "passed",
      "detail": "DAX and warehouse validations aligned with each other and disagreed with the report output."
    }
  ],
  "recommended_actions": [
    {
      "id": "action_1",
      "type": "proposed_model_change",
      "summary": "Review and update the quarter-over-quarter revenue measure logic."
    }
  ],
  "blockers": [],
  "open_questions": [],
  "proposed_changes": [
    {
      "id": "change_1",
      "artifact_type": "measure",
      "artifact_ref": "Sales Model.Sales[QoQ Revenue]",
      "change_summary": "Replace current quarter offset logic with prior-quarter comparison logic aligned to business definition."
    }
  ],
  "contradiction_refs": []
}
```

The chat answer is the summary. The structured result is the primary artifact.

## 11. Connectivity And Adapters

### 11.1 Architectural Rule

Spool's core contracts should be platform-neutral.

This includes:

- task contracts
- artifact references
- evidence envelopes
- contradiction records
- validation results
- result states
- planner, generator, and evaluator handoff packets

Concrete platform knowledge belongs in adapter-owned mappings, capability declarations, and platform-specific evidence detail payloads.

Spool should not force every platform into a fake lowest-common-denominator feature set. Shared contracts should be generic only where the semantics are actually shared.

### 11.2 V1 Adapter Boundaries

The first-class adapter boundaries in v1 are:

- LLM provider adapters
- data access adapters
- Fabric operation adapters
- authentication adapters

### 11.3 Platform Adapter Model

V1 ships with one concrete platform family:

- Fabric

Future platform families may be added through separate adapters, such as:

- Snowflake
- Databricks
- BigQuery
- other warehouse or semantic-layer platforms

Every platform adapter should map platform-native artifacts and operations into Spool's platform-neutral contracts.

Core should not encode Fabric-only assumptions as required invariants unless they are explicitly marked as Fabric-specific capability extensions.

### 11.4 Platform Capability Contract

Every concrete platform adapter must publish a capability contract.

The capability contract is the machine-readable declaration of what the adapter can do, what evidence it can produce, and what safety boundaries apply.

At minimum, a capability contract should declare:

- artifact kinds the adapter can resolve
- inspection operations the adapter supports
- validation operations the adapter supports
- mutation operations the adapter supports
- whether mutation is enabled, proposal-only, or disallowed
- supported identity locator shapes
- supported evidence classes
- freshness expectations and known drift surfaces
- auth requirements
- scope limits and tenancy boundaries
- user-confirmation requirements for risky or scope-expanding actions

Capability contracts exist so that:

- planner can choose only valid actions
- generator can avoid hallucinating unsupported operations
- evaluator can judge whether evidence or conclusions exceed adapter capability
- tests can verify adapter conformance without live platform coupling in every core test

### 11.4.1 Example: `spool-fabric` Capability Contract

The following is an illustrative v1-shaped example, not a locked wire format:

```json
{
  "adapter_id": "spool-fabric",
  "platform_family": "fabric",
  "status": "active",
  "artifact_kinds": [
    "report",
    "report_page",
    "visual",
    "semantic_model",
    "measure",
    "table",
    "column",
    "relationship",
    "warehouse"
  ],
  "inspection_capabilities": [
    "resolve_artifact_from_report_url",
    "resolve_artifact_from_workspace_and_guid",
    "inspect_report_metadata",
    "inspect_semantic_model_metadata",
    "inspect_measure_definition",
    "inspect_visual_binding_metadata",
    "inspect_warehouse_metadata"
  ],
  "validation_capabilities": [
    "run_dax_query",
    "run_read_only_warehouse_sql",
    "compare_report_output_to_dax_result",
    "compare_dax_result_to_warehouse_result"
  ],
  "mutation_capabilities": [],
  "mutation_mode": "proposal_only",
  "identity_locator_shapes": [
    "fabric://workspace/{workspace_id}/report/{report_id}",
    "fabric://workspace/{workspace_id}/report/{report_id}/page/{page_name}",
    "fabric://workspace/{workspace_id}/model/{model_id}/measure/{table}[{measure}]",
    "fabric://workspace/{workspace_id}/warehouse/{warehouse_id}"
  ],
  "evidence_classes": [
    "report_metadata",
    "visual_metadata",
    "semantic_model_metadata",
    "measure_definition",
    "dax_query_result",
    "warehouse_query_result",
    "cross_source_comparison"
  ],
  "safety_rules": {
    "warehouse_sql": "read_only_only",
    "fabric_mutation": "disallowed_in_v1",
    "cross_workspace_scope_expansion": "requires_user_confirmation",
    "ambiguous_artifact_resolution": "requires_user_choice"
  },
  "freshness_and_drift": [
    "report definitions can drift",
    "semantic model definitions can drift",
    "warehouse data can change between validations"
  ],
  "auth_requirements": [
    "product_login",
    "fabric_access_auth"
  ]
}
```

This example shows the intended contract shape:

- what Fabric artifacts can be resolved
- what Fabric inspection and validation operations are supported
- what evidence classes can be emitted
- what safety boundaries apply in v1

The example is intentionally medium-detail. It should guide architecture, planning, and testing without freezing exact implementation names too early.

### 11.5 LLM Provider Adapters

V1 implementations:

- Azure OpenAI
- Anthropic

The provider-neutral interface should support:

- streaming responses
- tool calling
- structured output
- reasoning-effort configuration
- model selection and switching

The LLM provider layer should follow the same adapter rule:

- provider-neutral core contract
- provider-specific implementation packages
- explicit capability declaration for features such as structured output, tool calling, streaming, and context-window constraints

### 11.6 Data Access Adapters

V1 data access set:

- semantic-model DAX query path
- Fabric Warehouse

Future data-access adapters may support other warehouses, semantic engines, or query surfaces, but those should attach through the same contract pattern rather than bypassing core.

### 11.7 Fabric Operation Adapters

Candidate operation transports:

- REST
- MCP

This is an explicit blocker for v1:

- determine whether REST fully covers required Fabric operations
- determine whether MCP is still needed for gaps that REST cannot cover cleanly

### 11.8 Authentication Adapters

Spool has two auth concerns:

- product login auth
- Fabric access auth

V1 implementations:

- GitHub OAuth / device-style product login
- Entra/Fabric access auth

### 11.9 Workspace And Package Separation

Spool should use workspace-level logical separation so the platform-neutral contract layer stays stable as adapters and UX evolve.

The preferred package split is:

- `spool-core`
  - harness orchestration
  - planner, generator, evaluator coordination
  - task contracts
  - evidence and contradiction handling
  - result-state rules
- `spool-protocol` or `spool-model`
  - shared domain types
  - artifact identity shapes
  - capability-contract types
  - persisted structured state shapes
- `spool-fabric`
  - Fabric adapters
  - REST or MCP integration
  - Fabric-specific artifact resolution
  - Fabric-specific evidence detail payloads
- `spool-index`
  - schema indexer and bundle generation
  - curated knowledge loading contracts
- `spool-tui`
  - terminal user experience
  - advanced inspection views
  - transcript and result rendering

Dependency direction should remain strict:

- `spool-core` depends on `spool-protocol`
- `spool-fabric` depends on `spool-protocol` and selected `spool-core` interfaces, but should not own task semantics
- `spool-tui` depends on `spool-core` and `spool-protocol`
- `spool-index` emits knowledge artifacts and should not own runtime task orchestration

This keeps Fabric as the first platform implementation without making Fabric the permanent center of the architecture.

## 12. Session, Compaction, And Resume

### 12.1 Persistence Rule

Spool persists full raw session history for:

- audit
- replay
- resume
- advanced inspection

### 12.2 Persisted State Model

Persistence in v1 should distinguish raw history from structured state.

The persisted session state should include at least:

- session metadata
- task-contract records
- live task-status records
- canonical task-result records
- evidence ledgers
- contradiction ledgers
- checkpoint and question history
- pending interaction records
- advanced transcript references
- raw transcript history

Structured state is the primary restore source for resume. Raw transcript history remains available for audit and advanced inspection.

### 12.3 Compaction Rule

Spool does not keep full raw history in active model context. V1 uses local structured compaction.

Compaction should produce or refresh structured working state rather than only truncating text.

Compaction outputs should preserve at least:

- active or last relevant task contract
- current task phase
- evidence ledger summary
- contradiction summary
- unresolved user questions
- unresolved approval requests
- unresolved evaluator requests
- active artifact focus
- latest canonical task result when one exists

### 12.4 Active Context Rule

The active context should be composed from:

- task contract
- evidence ledger
- current working state
- recent unresolved tail
- selected knowledge bundle
- relevant durable memory summaries with inspectable source and scope provenance

### 12.5 Resume Semantics

Resume is session-level from the user point of view, while internally restoring:

- selected session identity
- active task contract when one exists, otherwise the latest completed task contract
- task phase and live status
- selected LOB and workspace scope
- current artifact focus when one exists
- evidence ledger state
- contradiction records
- unresolved checkpoints or pending user questions
- unresolved approval requests
- unresolved evaluator requests for additional evidence
- latest canonical task result when one exists
- advanced transcript references

### 12.6 Resume Resolution Rules

Resume should prefer the latest structured state over naive transcript replay.

If multiple resumable task states exist within the same session, v1 should restore:

1. the active task if one is marked active
2. otherwise the most recently updated incomplete task
3. otherwise the most recent completed task

Resume must not invent hidden context that is not represented in persisted structured state or raw history.

### 12.7 Active Resume Context

Resume should rebuild active model context from structured state, not from raw transcript replay alone.

The active resumed context should prioritize:

- active or last relevant task contract
- current phase of work
- unresolved tail
- evidence ledger summary
- contradiction summary
- pending evaluator or user-facing next steps
- selected knowledge bundle
- relevant durable memory summaries with inspectable source and scope provenance

The following should not be injected into active context by default:

- full raw transcript
- raw tool outputs that are already represented in structured evidence or summaries
- stale abandoned branches with no remaining relevance

### 12.8 Interrupted Task Handling

If a task was interrupted mid-investigation, resume should surface that explicitly rather than pretending the task had ended cleanly.

Resume should indicate at least:

- whether the task was in planning, generation, evaluation, or result-finalization phase
- whether a user answer was pending
- whether an approval response was pending
- whether the evaluator had requested more evidence
- whether the last known state was partial, downgraded, blocked, or not yet finalized

If the evaluator had requested more evidence before interruption, that request should be restored as pending work. It must not be silently dropped during compaction or resume.

If a user-input or approval request was pending before interruption, resume should restore that pending interaction explicitly. It must not silently convert the task to completed.

## 13. UX Model

### 13.1 Plan Mode

Plan mode is analytics-native and used to:

- refine the request
- select scope
- define artifacts
- review or influence the suggested investigation approach
- define evidence expectations
- finalize the task contract

When exiting plan mode, the user must be offered:

- start now
- keep refining

### 13.2 Progress Surface

The default progress surface should emphasize:

- current phase
- current artifact under investigation
- latest meaningful finding
- waiting state if blocked
- waiting state if user input is pending
- waiting state if approval is pending

### 13.3 Advanced View

Advanced view should expose:

- structured transcript
- planner/generator/evaluator activity
- evidence ledger detail
- intermediate artifacts

It should not expose hidden chain-of-thought.

Advanced view should expand on disagreement details, but it must not be the only place where evaluator objection or downgrade is visible. Material generator/evaluator disagreement must already be visible in the normal answer.

## 14. Security And Policy

### 14.1 Confirmation Policy

Spool interrupts for confirmation only when the next step is:

- ambiguous
- scope-expanding
- expectation-shaping
- side-effecting

Routine low-risk investigation inside confirmed scope should proceed without extra confirmation.

### 14.2 SQL Policy

Warehouse SQL is read-only in v1. Non-read statements are not allowed.

## 15. Lifecycle Sketches

### 15.1 Session Startup

1. Launch CLI
2. Authenticate product login
3. Authenticate Fabric access
4. Select LOB
5. Establish workspace scope
6. Load selected LOB Tier 1 + Tier 2
7. Build prompt context
8. Enter chat session

### 15.2 Task Execution

1. User asks question
2. Planner creates or refines task contract
3. User starts task
4. Generator investigates
5. Generator collects evidence and validations
6. Evaluator subagent reviews bounded packet
7. Result is emitted
8. User may inspect, continue, or resume later

## 16. Open Questions And Blockers

- exact warehouse SQL transport
- whether REST alone is sufficient for required Fabric operations
- whether MCP remains necessary for semantic-model or report gaps
- exact provider-specific model constraints
- exact TUI component tree and rendering details
- exact local persistence file formats

## 17. Required References

The following references are required inputs for dev planning and implementation. They are not optional background material.

Any future implementation plan or major design refinement for Spool should explicitly review these references and should not silently ignore them when defining package boundaries, harness behavior, resume semantics, adapter surfaces, or knowledge-bundle design.

### 17.1 Primary Governing Spec

- [2026-04-06-spool-refined-spec.md](/Users/gaxx/Github/copex/docs/superpowers/specs/2026-04-06-spool-refined-spec.md)
  - governing product spec for Spool v1

### 17.2 Copex References

- [copex repo root](/Users/gaxx/Github/copex)
  - overall source repository and surrounding product context
- [codex-rs workspace manifest](/Users/gaxx/Github/copex/codex-rs/Cargo.toml)
  - reference for workspace-level crate decomposition and long-term package boundaries
- [codex-rs core](/Users/gaxx/Github/copex/codex-rs/core)
  - reference for harness and execution-core implementation patterns to borrow carefully rather than rebuild blindly
- [codex-rs protocol](/Users/gaxx/Github/copex/codex-rs/protocol)
  - reference for keeping shared contracts/types as a distinct seam
- [codex-rs tui](/Users/gaxx/Github/copex/codex-rs/tui)
  - reference for terminal UX structure, transcript rendering, and resumable-session interaction patterns

### 17.3 Claude Code Snap Reference

- [ClaudeCodeSnap repo root](/Users/gaxx/Github/ClaudeCodeSnap)
  - reference for Claude Code-inspired session resume behavior, long-running harness ideas, and terminal agent interaction patterns

### 17.4 Knowledge Layer Reference

- [knowledge template root](/Users/gaxx/Work/agent-console.feature-structured-knowledge-base/knowledge)
  - reference for authored knowledge-bundle structure, domain guidance patterns, and reusable template direction for Spool Tier 2 knowledge

### 17.5 External Design Reference

- Anthropic engineering blog: `Harness design for long-running application development`
  - URL: `https://www.anthropic.com/engineering/harness-design-long-running-apps`
  - published March 24, 2026
  - reference for planner, generator, evaluator separation, structured handoff design, and long-running harness behavior

## 18. Final Product Definition

Spool is a terminal-native Fabric analytics agent built around structured investigation rather than code generation.

The defining combination is:

- planner, generator, evaluator harness
- task-contract-centered execution
- proposal-first v1 behavior
- selected-LOB knowledge loading
- named structured investigation recipes
- append-only evidence
- DAX-first and warehouse-backed validation
- local structured compaction
- resumable sessions
- canonical machine-readable internal results with optional human-readable and machine-readable user-facing outputs in v1, without promising a stable export contract yet

This is the governing product direction for v1.
