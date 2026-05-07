# Spool Plan Index

## Purpose

This index groups the greenfield Spool implementation plans into dependency-ordered delivery waves and defines the shared execution contract that all six plans rely on.

These plans are authored inside the `copex` repository for review convenience, but they target a future standalone Spool workspace that will eventually move under `/Users/gaxx/Work`.

## Governing Inputs

Every plan in this set is governed by:

- `docs/superpowers/specs/2026-04-06-spool-refined-spec.md`
- `docs/superpowers/specs/2026-04-06-spool-dev-planning-readiness-design.md`
- `docs/superpowers/specs/2026-04-07-spool-contradiction-handling-subspec.md`

If a later subspec changes one of the contracts owned by a numbered plan, the numbered plan must be updated before implementation starts. The April 7 contradiction subspec is already in force for Plans 1, 5, and 6.

## Planned Workspace Shape

The plans assume a future Rust workspace rooted at:

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

## Execution Portability Rule

The implementation target is the future `spool/` workspace, but the authoritative plan and spec artifacts currently live in `copex/docs/...`.

Before any implementation session starts, the operator must choose one of these two supported execution carriers:

1. keep the plan and spec documents in `copex/docs/...` and implement against a sibling `spool/` workspace while leaving the docs in place
2. copy the full `docs/plans/codex/` spool plan set and the governing `docs/superpowers/specs/` spool specs into `spool/docs/` before implementation begins

Implementation must not start from a standalone `spool/` repo that lacks access to the governing plan and spec documents.

## Execution-Ready Contract For Every Numbered Plan

Each numbered plan in this set must be executable by an implementation agent without re-deriving core product semantics from the specs. For this plan set, "detailed enough to execute" means the plan itself must provide all of the following:

- a requirement-to-task mapping showing which spec or planning requirements the plan owns
- explicit non-negotiable subsystem invariants so the implementer does not infer behavior from minimal code stubs alone
- exact create-versus-modify ownership for planned files
- verification steps that distinguish local contract tests from live-integration gates
- live-fixture and config assumptions written in task language rather than hidden in shell folklore
- handoff artifacts that later plans can depend on without reopening the earlier plan

If a numbered plan lacks one of those elements, it is not execution-ready even if the task list looks structurally complete.

## Cross-Plan Handoff Rule

Every numbered plan must leave behind concrete outputs that later plans can consume directly. At minimum, those outputs are:

- stable Rust types and module boundaries for the contracts owned by the plan
- test files that prove the owned behavior in isolation
- one architecture note that records why the boundary exists and what later plans may rely on
- explicit notes about unresolved decisions that later plans must not silently guess through

Later plans may extend those outputs, but they must not reinterpret them silently. If a later plan needs a contract shape that an earlier plan cannot support, the earlier plan must be updated first.

## Shared Validation Environment And Fixture Contract

All live integration paths after Plan 1 use one shared dev environment contract rather than plan-specific shell snippets.

### Canonical config path

- default local config file: `~/.config/spool/dev.toml`
- override mechanism: `spool --config <path>`

### Required config keys

The shared dev config must expose these values, either directly or through nested sections that the corresponding plan defines explicitly:

- `tenant_id`
- `client_id`
- `workspace_name`
- `workspace_id` when the operator already knows the stable workspace GUID
- `report_name`
- `report_id` when available
- `page_name`
- `visual_name`
- `semantic_model_name`
- `semantic_model_id` when available
- `warehouse_name`
- `warehouse_dsn` when warehouse validation is enabled
- `access_token_env`

The default token source for live tests is the environment variable named by `access_token_env`. Plans must not rely on inline `security find-generic-password`, `grep`, or similar shell-only extraction as the primary execution path.

### Shared live fixture set

The plans assume one small shared fixture set in the dev Fabric workspace:

- one workspace fixture
- one report-side fixture path: report -> page -> visual
- one semantic-model fixture path: model -> measure -> table -> column -> relationship
- one warehouse fixture

Fixture values may differ by environment, but the config keys and the shape of the fixture set are stable across Plans 2-6.

### Live validation rule

When a plan declares a real integration gate, that gate must be backed by at least one explicit task that:

1. loads the shared config contract
2. exercises the stated seam against the shared live fixture set
3. asserts on canonical Spool contracts rather than raw transport payloads alone

## Delivery Waves

### Wave 1: Contract Backbone

1. [Harness Semantics Foundation](./2026-04-06-spool-01-harness-semantics-foundation.md)

### Wave 2: Live Platform Seams

2. [Fabric Adapter Foundations](./2026-04-06-spool-02-fabric-adapter-foundations.md)
3. [Validation Execution Paths](./2026-04-06-spool-03-validation-execution-paths.md)

### Wave 3: Domain Context And Operator Experience

4. [Knowledge And Indexing](./2026-04-06-spool-04-knowledge-and-indexing.md)
5. [TUI And Session UX](./2026-04-06-spool-05-tui-and-session-ux.md)

### Wave 4: Product Hardening

6. [Operationalization And Hardening](./2026-04-06-spool-06-operationalization-and-hardening.md)

## Global Planning Rules

- Plan 1 validates harness semantics with simulated or fixture-backed adapters only.
- Every later plan must include an explicit integration-validation path.
- Later plans should prefer real external validation against the dev Fabric workspace whenever the seam being tested supports it.
- Every plan must include an `Open Items / Deferred Decisions` section that distinguishes owned decisions from deferred ones.
- Plans that touch recipe behavior must treat recipe discovery as planner-owned and must describe user-facing approaches in task language rather than exposing internal recipe IDs as required user knowledge.
- Plans that touch contradiction handling must stay aligned with `2026-04-07-spool-contradiction-handling-subspec.md`.
- Live steps must use the shared config contract and shared fixture set from this index instead of ad hoc literals or shell-only credential lookups.
- Copex is reference material only. All implementation paths in the plans target the future Spool workspace, not existing `codex-rs` crates.
