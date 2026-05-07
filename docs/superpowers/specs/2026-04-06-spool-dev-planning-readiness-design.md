# Spool Development Planning Specification

## Document Status

- Status: Canonical planning specification
- Date: 2026-04-06
- Type: Development planning specification
- Governing product spec: `2026-04-06-spool-refined-spec.md`

## 1. Purpose

This document defines the planning architecture for implementing Spool from the governing product spec.

It answers a narrower question than the governing product spec:

- the planning shape required to implement the product spec
- the required structure and sequencing of the dev plan set

This document does not replace the governing product spec. It defines how development planning must be structured from that spec.

## 2. Readiness Assessment

The refined Spool spec is sufficient to support development planning.

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

These open items do not block development planning across the board. They must instead be surfaced and owned by the specific dev plans that need to resolve them.

## 3. Planning Shape

Spool does not use a single master implementation plan.

Spool uses a coordinated set of dev plans.

The planning model is hybrid:

- each dev plan owns a clean subsystem boundary or a tightly bounded vertical slice
- the overall roadmap groups those plans into delivery waves with explicit dependencies

This hybrid model is required because the Spool spec spans multiple technical seams with different risks:

- harness semantics
- Fabric adapters
- validation execution paths
- knowledge and indexing
- TUI and session UX
- exports, durable memory, and operational hardening

Using only delivery phases would mix ownership and acceptance criteria inside one plan. Using only subsystem plans would make the roadmap harder to sequence and evaluate as product delivery. The plan set therefore uses subsystem-clean plans presented in phased order.

## 4. Plan 1 Rule

Plan 1 is a special case.

Plan 1 validates harness semantics in isolation and does not depend on live Fabric readiness.

Plan 1 focuses on:

- canonical task-contract modeling
- evidence-ledger and contradiction-ledger semantics
- evaluator loop protocol
- live task-status and waiting-state semantics
- checkpoint behavior
- canonical task-result semantics
- pending interaction semantics for approval and user input
- persisted structured state needed for resume and compaction

Plan 1 uses simulated or fixture-backed adapters only.

Plan 1 completion is proven through:

- contract tests
- state-machine tests
- deterministic fixture scenarios

Those proofs cover both:

- terminal result production
- non-terminal waiting states such as pending user input or pending approval

Plan 1 does not require:

- live auth
- live Fabric artifact resolution
- live DAX execution
- live warehouse validation

## 5. Integration Validation Rule For Later Plans

Every dev plan after Plan 1 includes an explicit integration-validation path.

That validation path identifies:

- the seam being proven
- the scenario being exercised
- the environment used
- the success condition

Whenever possible, these validation paths use real external systems rather than only local fixtures.

For Spool, later plans prefer validation against the dev Fabric workspace when the plan meaningfully touches live Fabric behavior or another live external seam.

Fixture-only validation remains acceptable when:

- the plan is intentionally isolated from live systems
- a real external seam is not yet owned by the plan
- the relevant live environment does not exist or cannot support the scenario

## 6. Validation Environment Assumptions

Planning assumptions:

- a dev Fabric workspace exists
- Spool can read required environment or connection details from its configuration file

Validation fixtures use a mixed strategy:

- define a small stable shared fixture set that multiple plans can reuse
- allow additional plan-specific fixtures when justified by the plan's scope

Plans prefer the shared fixture set first and explicitly justify any plan-specific fixture additions.

## 7. Required Dev-Plan Structure

The `writing-plans` skill is used to author the dev plans.

For Spool, each plan includes both the normal `writing-plans` structure and the following product-specific requirements:

- a clear subsystem or bounded-slice scope
- explicit out-of-scope section
- dependencies on prior plans and runtime prerequisites
- contract-impact section stating whether the plan implements, refines, or pressures a governing contract
- validation section with the plan's integration-validation path
- open-items section listing relevant unresolved decisions

The required open-items section distinguishes:

- open items owned by the plan
- open items noted but deferred to another plan
- review triggers that would require reopening the plan

This keeps deferred technical uncertainty visible during plan review instead of allowing it to disappear behind seemingly clean plan scope.

## 8. Development Plan Set

The development plan set for v1 is:

1. Harness Semantics Foundation
2. Fabric Adapter Foundations
3. Validation Execution Paths
4. Knowledge And Indexing
5. TUI And Session UX
6. Operationalization And Hardening

These plans are presented in delivery waves, but each plan remains technically clean in scope and reviewable on its own.

Plan intent:

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

The first concrete live scenario does not need to be fixed before writing the plan set. It is finalized as part of the relevant later plan's validation section.

## 8.1 Planned Workspace Shape

The plans target a future Rust workspace rooted at:

```text
spool/
  Cargo.toml
  rust-toolchain.toml
  README.md
  docs/
    specs/
    architecture/
  spool/
  spool-core/
  spool-model/
  spool-fabric/
  spool-knowledge/
  spool-index/
  spool-tui/
  spool-otel/
```

This workspace shape includes a thin `spool/` app crate. It does not use a separate `spool-cli/` crate in v1 planning.

## 9. Governing Review Standard

During dev-plan review, reviewers distinguish between:

- governing product contracts that should remain stable unless disproven
- implementation choices that a specific plan is expected to resolve

The purpose of the plan set is not to reopen the entire product definition. The purpose is to move from refined product semantics into executable engineering work while keeping open technical decisions visible and owned.

## 10. Conclusion

Spool uses multiple coordinated dev plans.

The next planning action is to use the `writing-plans` skill to author the coordinated plan set defined by this document and the governing product spec.
