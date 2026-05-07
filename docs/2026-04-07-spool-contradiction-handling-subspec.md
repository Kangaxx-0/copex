# Spool Contradiction Handling Sub-Spec

**Parent spec:** [2026-04-06-spool-refined-spec.md](2026-04-06-spool-refined-spec.md) Section 9.4  
**Scope:** Defines LLM-level agent behavior for contradiction detection, materiality assessment, resolution, and evaluator adjudication. Fills the gap between the structural data contracts (Plan 01) and the runtime agent prompting strategy.  
**Contract changes:** This sub-spec requires additions to `ContradictionRecord`, `EvidenceClass`, and evaluator interaction semantics defined in Plan 01. All required contract changes are listed in section 8.

---

## 1. Problem Statement

The refined spec defines contradiction data structures (`ContradictionRecord`, `ContradictionLedger`) and structural invariants (e.g., confirmed+high cannot coexist with open material contradictions). The dev plans prove these mechanics work with scripted fakes.

Neither the spec nor the plans define:

- How the generator LLM detects contradictions during investigation
- How the generator LLM assesses materiality
- How the generator LLM attempts resolution before escalating
- How the evaluator LLM adjudicates contradictions in the bounded packet
- What prompting structure ensures contradictions are consistently surfaced rather than silently absorbed into narrative

This sub-spec defines those behaviors.

---

## 2. Design Principles

1. **Contradictions are structural, not narrative.** The generator must emit `ContradictionRecord` entries into the ledger, not merely mention conflicts in prose. If a conflict only exists in the summary text, it is invisible to the evaluator's structured review and to the validation invariants.

2. **Detection before resolution.** The generator must record the contradiction *before* attempting to resolve it. This prevents silent absorption where the LLM reasons through a conflict internally and presents only the winning interpretation.

3. **Materiality is a judgment call with guardrails.** The LLM decides materiality, but the spec constrains what "material" means: a contradiction is material when it can change the leading conclusion, recommendation, confidence, or result state.

4. **Resolution is evidence-gated.** A contradiction moves from `Open` to `Resolved` only when new observed evidence supports one side over the other. Reasoning alone does not resolve a contradiction.

5. **The evaluator is the final arbiter.** The generator proposes contradiction status. The evaluator adjudicates contradiction correctness through bounded outcome instructions that the generator must apply on the next iteration.

---

## 3. Generator Contradiction Behavior

### 3.1 Detection Trigger

The generator must check for contradictions whenever it produces or receives a new evidence item that overlaps in scope with an existing evidence item. Specifically:

- After each observed evidence item is appended to the ledger, the generator must compare it against all prior evidence items that reference the same artifact, measure, metric, scope, or time period.
- After each derived evidence item is appended, the generator must check whether the derivation is consistent with existing observed and derived evidence.

The generator prompt must include an explicit instruction:

> After recording each new evidence item, compare it against prior evidence in the ledger. If the new item contradicts or materially disagrees with any prior item on the same claim, artifact, or scope, you MUST create a ContradictionRecord before continuing investigation.

### 3.2 Detection Criteria

A contradiction exists when:

- Two observed evidence items report incompatible values for the same metric, measure, or artifact state under the same scope and time period
- A derived conclusion conflicts with an observed evidence item
- A knowledge-bundle claim (evidence class `BusinessKnowledge`) conflicts with observed runtime evidence
- A durable-memory claim conflicts with current observed evidence. **Note: requires `DurableMemory` evidence class. See section 8.2.**
- A user assertion conflicts with observed evidence (note: the user assertion is not wrong per se, but the conflict must be recorded). **Note: requires `UserAssertion` evidence class. See section 8.2.**

A contradiction does NOT exist when:

- Two evidence items cover different scopes, time periods, or artifacts and the difference is expected
- A proposed hypothesis is simply unvalidated (that is an open question, not a contradiction)
- Evidence items differ in precision but not in direction (e.g., 12.4M vs 12.41M due to rounding)

### 3.3 Contradiction Record Creation

When the generator detects a contradiction, it must emit a `ContradictionRecord` with:

| Field (Plan 01 name) | Generator responsibility |
|-------|------------------------|
| `contradiction_id` | Generate a unique ID |
| `disputed_claim` | State the specific claim or comparison in dispute |
| `conflicting_evidence_refs` | Reference the evidence IDs on each side |
| `materiality` | Assess as Material or NonMaterial (see 3.4) |
| `freshness_note` | Note if one side is known to be staler |
| `resolution_attempted` | `false` at creation time. **Note: this field does not exist in Plan 01's current contract. See section 8.1 for the required addition.** |
| `resolution_note` | Empty at creation time. **Note: this field does not exist in Plan 01's current contract. See section 8.1 for the required addition.** |
| `status` | `Open` at creation time |

### 3.4 Materiality Assessment

The generator assesses materiality by answering one question:

> If this contradiction were resolved in favor of the OTHER side, would the task's leading conclusion, recommendation, confidence level, or result state change?

- If yes: `Material`
- If no: `NonMaterial`

Materiality assessment guidance for common cases:

| Scenario | Default materiality |
|----------|-------------------|
| Report value vs warehouse value disagree on the metric the user asked about | Material |
| Report value vs warehouse value disagree on an unrelated metric | NonMaterial |
| DAX result vs semantic-model definition imply different calculation logic for the target measure | Material |
| Two metadata sources disagree on artifact publish date | NonMaterial unless freshness is the investigation question |
| Knowledge-bundle business definition (`BusinessKnowledge`) vs observed runtime behavior | Material if the task involves business-meaning interpretation |
| Durable-memory claim (`DurableMemory`, see section 8.2) vs current runtime state | Material if the task relies on that claim |

### 3.5 Resolution Attempts

Before escalating a material contradiction to the evaluator, the generator SHOULD attempt resolution through the following steps, in order:

1. **Scope alignment check.** Are the conflicting evidence items actually measuring the same thing at the same scope? If not, the contradiction may be reclassified or downgraded.

2. **Freshness check.** Is one evidence item known to be staler than the other? If so, and if the fresher evidence is higher in the truth hierarchy, the staler item may be noted as superseded. The contradiction moves to `Resolved` with a freshness-based resolution note.

3. **Like-for-like revalidation.** Can the generator run a targeted validation that directly compares the conflicting claims under identical scope and filters? If so, run it. The new observed evidence either confirms one side (resolving the contradiction) or deepens it.

4. **Knowledge-source check.** Does curated knowledge or durable memory explain the discrepancy (e.g., a known business-rule exception)? If so, this may downgrade materiality but does not resolve the contradiction unless the explanation is confirmed by observed evidence.

If resolution succeeds:
- Set `resolution_attempted` to `true`
- Set `resolution_note` explaining which evidence won and why
- Set `status` to `Resolved`

If resolution fails:
- Set `resolution_attempted` to `true`
- Set `resolution_note` explaining what was tried and why it did not resolve
- Leave `status` as `Open`

### 3.6 Generator Must Not Self-Promote Past Contradictions

The generator MUST NOT propose `Confirmed` + `High` confidence when any material contradiction remains `Open` in the ledger. This is enforced structurally by `CanonicalTaskResult.validate()`, but the generator prompt should also include this rule to prevent wasted evaluator round-trips:

> Do not propose state=confirmed with confidence=high if any material contradiction is open. Propose supported_hypothesis or reduce confidence instead.

---

## 4. Evaluator Contradiction Behavior

### 4.1 Evaluator Review Scope

The evaluator receives the bounded packet containing:

- The generator's proposed `CanonicalTaskResult`
- The full `EvidenceLedger`
- The full `ContradictionLedger`

The evaluator must review ALL contradiction records, not just the ones referenced in `contradiction_refs`.

### 4.2 Evaluator Contradiction Checks

The evaluator does not directly modify contradiction records. Plan 01's `EvaluatorOutcome` returns structured outcomes with string reasons, not ledger patches. When the evaluator identifies a contradiction problem, it returns an outcome whose `reason` string instructs the generator to make the correction on the next iteration.

For each contradiction record, the evaluator must verify:

1. **Detection completeness.** Are there evidence items in the ledger that conflict but have no corresponding contradiction record? If the evaluator spots an unrecorded conflict, it should return `EvaluatorOutcome::Contradiction { reason }` instructing the generator to record the missing conflict.

2. **Materiality correctness.** Does the generator's materiality assessment match the evaluator's independent judgment? If not, the evaluator should return `EvaluatorOutcome::Contradiction { reason }` instructing the generator to reclassify the materiality (e.g., "reclassify contradiction_1 as Material: the disputed claim directly affects the task's leading conclusion").

3. **Resolution validity.** If a contradiction is marked `Resolved`:
   - Is the resolution backed by observed evidence, not just reasoning?
   - Does the `resolution_note` correctly identify which evidence won and why?
   - Is the winning evidence higher in the truth hierarchy and fresher?
   - If not, the evaluator should return `EvaluatorOutcome::RequestMoreEvidence { requested_targets }` instructing the generator to reopen the contradiction and gather specific additional evidence.

4. **Impact on result state.** Does the proposed result state and confidence correctly account for remaining open contradictions? Apply the calibration rules from spec section 10.7. If the generator proposed too strong a result, return `EvaluatorOutcome::Downgrade { reason }`.

In all cases, the generator is responsible for interpreting the evaluator's `reason` string and applying the corresponding ledger updates on the next iteration. Test scenario 7.4 and 7.5 validate this round-trip.

### 4.3 Evaluator Outcome Mapping

| Contradiction state | Evaluator action |
|---|---|
| No contradictions exist | Proceed with normal evaluation |
| All contradictions resolved with valid evidence | Proceed with normal evaluation; contradictions do not cap result |
| Material contradiction is Open but resolution was not attempted | Return `RequestMoreEvidence` with specific resolution steps |
| Material contradiction is Open, resolution attempted but failed | Return `Contradiction` or `Downgrade` depending on impact |
| Material contradiction is Open, resolution impossible (access/policy) | Return `Blocked` if it prevents meaningful conclusion; otherwise `Downgrade` with the limitation surfaced |
| Generator missed an obvious conflict in the evidence | Return `Contradiction` with instruction to record the missed conflict |
| NonMaterial contradictions only | Proceed; may note them but they do not block or downgrade |

### 4.4 Evaluator Must Not Instruct Erasure Of Contradictions

The evaluator's `reason` string must never instruct the generator to remove contradiction records from the ledger. The contradiction ledger is a mutable current-state ledger: fields on existing records may be updated, but records must never be deleted (see section 9, question 3). If the evaluator disagrees with a contradiction's existence, it should instruct the generator to set status to `Resolved` with a `resolution_note`, not to delete the record.

---

## 5. Harness Loop Integration

### 5.1 Contradiction In The Bounded Loop

The harness loop from Plan 01 already supports `EvaluatorOutcome::Contradiction`. This sub-spec defines what happens at each step:

```
Generator produces candidate result
  -> Harness passes (result + evidence + contradictions) to evaluator
  -> Evaluator checks contradictions per section 4.2
  -> If EvaluatorOutcome::Contradiction:
       -> Harness returns to generator with the evaluator's reason
       -> Generator must either:
           (a) record the missing contradiction and attempt resolution, or
           (b) attempt resolution on the flagged contradiction
       -> Generator produces updated candidate
       -> Loop continues (bounded by max_iterations)
  -> If EvaluatorOutcome::Downgrade due to unresolvable contradiction:
       -> Harness applies the downgrade to the final result
       -> Task completes with reduced state/confidence
  -> If EvaluatorOutcome::Accept:
       -> All contradiction checks passed
       -> Task completes
```

### 5.2 Loop Exhaustion With Open Contradictions

If the harness reaches `max_iterations` with material contradictions still open:

- The result state must be one of `supported_hypothesis`, `inconclusive`, or `blocked`. The result state must not be `confirmed`. This aligns with the parent spec's loop exhaustion rule (section 4.6).
- Confidence must not be `high`. The appropriate level (`medium` or `low`) depends on the strength of remaining evidence and the nature of the unresolved contradiction.
- The unresolved contradictions must appear in `contradiction_refs` and in the result summary.
- The result must include an `open_question` or `recommended_action` describing what would resolve the contradiction.

---

## 6. Prompt Structure Requirements

### 6.1 Generator System Prompt Must Include

1. The contradiction detection trigger rule (section 3.1)
2. The materiality assessment question (section 3.4)
3. The resolution attempt sequence (section 3.5)
4. The self-promotion guard (section 3.6)
5. The truth hierarchy from spec section 9.2
6. The freshness policy from spec section 9.3

### 6.2 Evaluator System Prompt Must Include

1. The four contradiction checks (section 4.2)
2. The outcome mapping table (section 4.3)
3. The no-deletion current-state ledger rule (section 4.4)
4. The confidence calibration rules from spec section 10.7
5. An explicit instruction to review ALL contradiction records, not just those referenced in the proposed result

### 6.3 Prompt Anti-Patterns To Avoid

- Do NOT instruct the LLM to "resolve contradictions when possible" without specifying that resolution requires observed evidence. This leads to narrative resolution where the LLM reasons its way out of conflicts.
- Do NOT instruct the evaluator to "check for contradictions" without specifying what to do when they are found. Vague instructions lead to contradictions being noted in prose but not affecting the outcome.
- Do NOT include contradiction examples that are all cleanly resolved. Include at least one example of an unresolvable contradiction that correctly caps the result.

---

## 7. Test Scenarios Required

These scenarios extend the scripted fakes in Plan 01, Task 7. Each should be a deterministic fixture that validates the behavioral contract, not just the data structures.

### 7.1 Generator Detects And Records Contradiction

- Generator receives two conflicting observed evidence items
- Generator emits a `ContradictionRecord` with status `Open`, materiality `Material`
- Evaluator receives the record in the bounded packet

### 7.2 Generator Resolves Contradiction Via Revalidation

- Generator detects a contradiction
- Generator runs a targeted revalidation query
- New evidence confirms one side
- Generator sets contradiction to `Resolved`
- Evaluator accepts the resolution

### 7.3 Generator Fails To Resolve, Evaluator Downgrades

- Generator detects a material contradiction
- Generator attempts resolution but targeted query is inconclusive
- Generator proposes `supported_hypothesis` with `medium` confidence
- Evaluator confirms the downgrade

### 7.4 Evaluator Catches Missed Contradiction

- Generator produces a result with two conflicting evidence items but no contradiction record
- Evaluator returns `EvaluatorOutcome::Contradiction` flagging the unrecorded conflict
- Generator records the contradiction on the next iteration

### 7.5 Evaluator Reopens Improperly Resolved Contradiction

- Generator marks a contradiction as `Resolved` based on reasoning alone (no new observed evidence)
- Evaluator reopens the contradiction and returns `RequestMoreEvidence`

### 7.6 Loop Exhaustion With Open Material Contradiction

- Contradiction remains unresolvable across all iterations
- Final result state is one of `supported_hypothesis`, `inconclusive`, or `blocked` (not `confirmed`)
- Confidence is `medium` or `low` (not `high`)
- Contradiction appears in the result output

### 7.7 NonMaterial Contradiction Does Not Block Confirmed

- Generator detects a contradiction and correctly assesses it as NonMaterial
- Evaluator confirms NonMaterial assessment
- Result can still reach `Confirmed` / `High`

---

## 8. Required Contract Changes And Relationship To Existing Plans

### 8.1 ContradictionRecord Additions (Plan 01)

Plan 01's current `ContradictionRecord` (`spool-model/src/contradiction.rs`) has six fields:

```rust
pub struct ContradictionRecord {
    pub contradiction_id: String,
    pub disputed_claim: String,
    pub conflicting_evidence_refs: Vec<String>,
    pub materiality: ContradictionMateriality,
    pub freshness_note: String,
    pub status: ContradictionStatus,
}
```

This sub-spec requires two additional fields:

| New field | Type | Purpose |
|-----------|------|---------|
| `resolution_attempted` | `bool` | Distinguishes "never tried to resolve" from "tried and failed." Required for the evaluator to decide between `RequestMoreEvidence` (go try) and `Downgrade`/`Contradiction` (you tried, it didn't work). |
| `resolution_note` | `Option<String>` | Records what resolution was attempted, which evidence won, and why. Required for the evaluator to verify that resolution is evidence-backed, not narrative-only. |

The updated struct:

```rust
pub struct ContradictionRecord {
    pub contradiction_id: String,
    pub disputed_claim: String,
    pub conflicting_evidence_refs: Vec<String>,
    pub materiality: ContradictionMateriality,
    pub freshness_note: String,
    pub resolution_attempted: bool,
    pub resolution_note: Option<String>,
    pub status: ContradictionStatus,
}
```

`ContradictionLedger::single()` and `ContradictionLedger::single_material_open()` constructors must be updated to initialize `resolution_attempted: false` and `resolution_note: None`.

### 8.2 EvidenceClass Additions (Plan 01)

Plan 01's current `EvidenceClass` (`spool-model/src/evidence.rs`):

```rust
pub enum EvidenceClass {
    ReportMetadata,
    VisualConfiguration,
    SemanticModelMetadata,
    MeasureDefinition,
    DaxQueryResult,
    WarehouseQueryResult,
    BusinessKnowledge,
}
```

This sub-spec requires two additional variants so that durable-memory claims and user assertions can participate as first-class evidence items in contradiction records:

| New variant | Purpose |
|-------------|---------|
| `DurableMemory` | Represents a claim carried forward from a prior session. The refined spec's truth hierarchy (section 9.2) lists durable memory as precedence level 4. Without this variant, contradictions between remembered state and current runtime evidence cannot be recorded with both sides as proper evidence items. |
| `UserAssertion` | Represents a user-provided claim that has not yet been validated. The refined spec's truth hierarchy lists user assertions as precedence level 6. Without this variant, contradictions between what the user stated and what runtime evidence shows cannot be recorded cleanly. |

### 8.3 Evaluator Interaction Model (No Contract Change, Operational Risk)

The evaluator does not need a richer `EvaluatorOutcome` variant for v1. All evaluator contradiction actions (reclassify materiality, reopen improperly resolved contradiction, flag missed conflict) are expressed as instructions in the `reason` string of existing outcome variants (`Contradiction`, `RequestMoreEvidence`, `Downgrade`). The generator is responsible for interpreting and applying these instructions on the next iteration.

This is a deliberate v1 trade-off: it avoids expanding the evaluator contract at the cost of relying on LLM instruction-following for the round-trip.

**This is an explicit operational reliability risk.** String-driven evaluator-to-generator correction depends on the generator LLM consistently parsing natural-language instructions and translating them into correct ledger updates. Failure modes include:

- Generator ignores or misinterprets the evaluator's reclassification instruction
- Generator applies the instruction to the wrong contradiction record
- Generator partially applies the instruction (e.g., updates status but not materiality)
- Generator applies the instruction but introduces inconsistency with other ledger state

**Plan 06 must include reliability validation for this mechanism.** Specifically:

- At least test scenarios 7.4 (evaluator catches missed contradiction) and 7.5 (evaluator reopens improperly resolved contradiction) must be run with real or realistic mock LLM calls, not just scripted fakes.
- These tests must verify that the generator correctly applies the evaluator's string instructions by asserting on the resulting ledger state, not just on the final result.
- If the round-trip failure rate exceeds an acceptable threshold during Plan 06 validation, a structured `ContradictionPatch` payload should be added to `EvaluatorOutcome::Contradiction` as a remediation.

### 8.4 Plan-By-Plan Impact Summary

| Plan | Impact |
|------|--------|
| Plan 01 (Harness Semantics) | **Contract changes required.** Add `resolution_attempted` and `resolution_note` to `ContradictionRecord` (section 8.1). Add `DurableMemory` and `UserAssertion` to `EvidenceClass` (section 8.2). Update constructors and existing tests accordingly. Extend scripted fakes with scenarios from section 7. |
| Plan 02 (Fabric Adapters) | No direct impact. Fabric adapters produce evidence; contradiction handling consumes it. |
| Plan 03 (Validation Paths) | Validation execution may be triggered by contradiction resolution attempts (section 3.5 step 3). The validation adapter must support targeted scope-aligned revalidation queries. No contract changes. |
| Plan 04 (Knowledge/Indexing) | Knowledge-bundle claims are subject to contradiction detection when they conflict with observed evidence. No contract changes. |
| Plan 05 (TUI/UX) | Advanced view already renders contradiction count. The contradiction detail view should additionally render `resolution_attempted`, `resolution_note`, and evaluator reclassification history (visible via the evaluator's `reason` strings in trace). No new contract changes beyond consuming the updated `ContradictionRecord`. |
| Plan 06 (Operationalization) | **Operational risk validation required.** Integration tests should include at least scenarios 7.1, 7.3, 7.4, 7.5, and 7.6 with real (or realistic mock) LLM calls. Tests must use the updated `ContradictionRecord` shape. Scenarios 7.4 and 7.5 specifically validate the string-driven evaluator correction round-trip (section 8.3) and must assert on ledger state, not just final result. |

---

## 9. Open Questions

1. **Contradiction detection cost.** Comparing every new evidence item against all prior items is O(n^2) in evidence count. For v1, is the evidence ledger small enough that this is negligible, or do we need a scoping heuristic (e.g., only compare evidence items that share the same artifact or measure)?

2. **Evaluator contradiction detection completeness.** The evaluator checking for missed contradictions requires it to cross-reference all evidence pairs. Should this be a structured pre-check (code-assisted comparison before the LLM evaluator prompt) or purely LLM-driven?

3. ~~**Contradiction record immutability vs append-only updates.**~~ **Resolved: mutable current-state ledger for v1.** The refined spec uses the phrase "append-only" for the evidence ledger (new items are never deleted), but the contradiction ledger is a **mutable current-state ledger** in v1. Contradiction records may be updated in place (status, materiality, resolution_attempted, resolution_note). Records must never be removed. This matches Plan 01's data structure (`pub` fields, `Vec<ContradictionRecord>`) and avoids the complexity of an event-sourced contradiction log. The "append-only" constraint applies to the *set of records* (no deletions), not to individual record fields. If audit-grade immutability is needed later, a superseding-event model can be introduced, but v1 does not require it.

4. **Multi-contradiction interactions.** When multiple contradictions exist, can resolving one affect the materiality of another? Should the generator re-assess materiality of remaining contradictions after each resolution?
