# Spool Dev Planning Readiness Design

## Document Status

- Status: Draft for review
- Date: 2026-04-06
- Type: Design review and planning-readiness addendum
- Governing product spec: `2026-04-06-spool-refined-spec.md`

## 1. Purpose

This document records the planning-oriented design conclusions reached after reviewing the Spool refined spec.

It answers a narrower question than the governing product spec:

- is the refined spec ready to move into development planning
- if so, what shape should the dev planning take

This document does not replace the governing product spec. It constrains how the next planning step should be performed.

## 2. Readiness Assessment

The refined Spool spec is ready to move into dev planning.

The spec is strong enough at the product-semantics layer to support implementation planning for:

- planner, generator, evaluator harness behavior
- canonical task contracts
- artifact identity rules
- evidence and contradiction handling
- selected-LOB knowledge composition and prompt-projection behavior
- planner-owned recipe discovery with natural-language user steering
- result-state semantics
- checkpoint policy
- session persistence, compaction, and resume concepts

The remaining uncertainty is mostly implementation-facing rather than product-definitional.

The main open items still visible in the governing spec are:

- exact warehouse SQL transport
- whether REST alone is sufficient for required Fabric operations
- whether MCP remains necessary for any semantic-model or report gaps
- exact provider-specific model constraints
- exact TUI component-tree details
- exact local persistence file formats

These open items should not block dev planning across the board. They should instead be surfaced and owned by the specific dev plans that need to resolve them.

## 3. Planning Shape

Spool should not move into a single master implementation plan.

The correct next step is a coordinated set of dev plans.

The recommended planning model is hybrid:

- each dev plan should own a clean subsystem boundary or a tightly bounded vertical slice
- the overall roadmap should group those plans into delivery waves with explicit dependencies

This hybrid model is preferred over phase-only planning because the Spool spec spans multiple technical seams with different risks:

- harness semantics
- Fabric adapters
- validation execution paths
- knowledge and indexing
- TUI and session UX
- exports, durable memory, and operational hardening

Using only delivery phases would make it too easy to mix ownership and acceptance criteria inside one plan. Using only subsystem plans would make the roadmap harder to sequence and evaluate as product delivery. The plan set should use subsystem-clean plans presented in phased order.

## 4. Plan 1 Rule

Plan 1 is a special case.

Plan 1 should validate harness semantics in isolation and should not depend on live Fabric readiness.

Plan 1 should focus on:

- canonical task-contract modeling
- evidence-ledger and contradiction-ledger semantics
- evaluator loop protocol
- live task-status and waiting-state semantics
- checkpoint behavior
- canonical task-result semantics
- pending interaction semantics for approval and user input
- persisted structured state needed for resume and compaction

Plan 1 should use simulated or fixture-backed adapters only.

Plan 1 completion should be proven through:

- contract tests
- state-machine tests
- deterministic fixture scenarios

Those proofs should cover both:

- terminal result production
- non-terminal waiting states such as pending user input or pending approval

Plan 1 should not require:

- live auth
- live Fabric artifact resolution
- live DAX execution
- live warehouse validation

## 5. Integration Validation Rule For Later Plans

Every dev plan after Plan 1 should include an explicit integration-validation path.

That validation path should identify:

- the seam being proven
- the scenario being exercised
- the environment used
- the success condition

Whenever possible, these validation paths should use real external systems rather than only local fixtures.

For Spool, this means later plans should prefer validation against the dev Fabric workspace when the plan meaningfully touches live Fabric behavior or another live external seam.

Fixture-only validation remains acceptable when:

- the plan is intentionally isolated from live systems
- a real external seam is not yet owned by the plan
- the relevant live environment does not exist or cannot support the scenario

## 6. Validation Environment Assumptions

The current planning assumptions are:

- a dev Fabric workspace exists
- Spool can read required environment or connection details from its configuration file

Validation fixtures should use a mixed strategy:

- define a small stable shared fixture set that multiple plans can reuse
- allow additional plan-specific fixtures when justified by the plan's scope

Plans should prefer the shared fixture set first and explicitly justify any plan-specific fixture additions.

## 7. Required Dev-Plan Structure

The `writing-plans` skill should be used to author the actual dev plans.

For Spool, each plan should include both the normal `writing-plans` structure and the following product-specific requirements:

- a clear subsystem or bounded-slice scope
- explicit out-of-scope section
- dependencies on prior plans and runtime prerequisites
- contract-impact section stating whether the plan implements, refines, or pressures a governing contract
- validation section with the plan's integration-validation path
- open-items section listing relevant unresolved decisions

The required open-items section should distinguish:

- open items owned by the plan
- open items noted but deferred to another plan
- review triggers that would require reopening the plan

This is necessary so deferred technical uncertainty remains visible during plan review instead of disappearing behind seemingly clean plan scope.

## 8. Recommended Dev-Plan Set

The current recommended plan set is:

1. Harness Semantics Foundation
2. Fabric Adapter Foundations
3. Validation Execution Paths
4. Knowledge And Indexing
5. TUI And Session UX
6. Operationalization And Hardening

These plans should be presented in delivery waves, but each plan should remain technically clean in scope and reviewable on its own.

Representative intent for each plan:

1. Harness Semantics Foundation
   - prove the core task/evidence/result/resume semantics in isolation
2. Fabric Adapter Foundations
   - establish real auth, artifact resolution, metadata inspection, and capability declarations
3. Validation Execution Paths
   - establish real DAX and warehouse validation behavior and evidence capture
4. Knowledge And Indexing
   - establish Tier 1 generation, Tier 2 structure, embedded examples inside the selected LOB bundle, planner-facing recipe-selection contracts, natural-language user-preference mapping, bundle validation, runtime loading, and single-block knowledge projection for prompt composition
5. TUI And Session UX
   - establish plan mode, progress, advanced view, compaction, and resume rendering with user-facing investigation-approach summaries rather than internal recipe identifiers
6. Operationalization And Hardening
   - establish exports, durable memory, policy hardening, telemetry, and polish

The first concrete live scenario does not need to be fixed before writing the plan set. It should instead be finalized as part of the relevant later plan's validation section.

## 9. Governing Review Standard

During dev-plan review, reviewers should distinguish between:

- governing product contracts that should remain stable unless disproven
- implementation choices that a specific plan is expected to resolve

The purpose of the plan set is not to reopen the entire product definition. The purpose is to move from the refined product semantics into executable engineering work while keeping open technical decisions visible and owned.

## 10. Conclusion

Spool is ready for multiple dev plans now.

The correct next step is to use the `writing-plans` skill to author a coordinated plan set based on the refined spec and this planning-readiness design.
