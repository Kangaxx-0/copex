# Spool Knowledge And Indexing Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the Spool knowledge bundle system, including Tier 1 schema packages, Tier 2 curated artifacts with embedded examples, recipe contracts, planner-facing recipe-selection surfaces, bundle validation, runtime selected-LOB loading, single-block knowledge projection for prompt composition, and the `spool-index` build path.

**Architecture:** This plan introduces a dedicated `spool-knowledge` crate for stable bundle contracts, validation, planner-facing recipe-selection surfaces, runtime loading, and selected-LOB knowledge projection, plus a separate `spool-index` binary for build-time Tier 1 generation from Fabric semantic-model definitions. Runtime loading remains selected-LOB-only, bundle validation spans both tiers, worked examples stay inside owned Tier 2 artifacts rather than becoming a separate prompt-time examples section, and recipe selection remains planner-owned even when the user supplies a natural-language investigation preference.

**Tech Stack:** Rust 2024, Serde, serde_json, serde_yaml, anyhow, clap, Tokio, pretty_assertions

---

## Context For The Implementer

This plan establishes the formal knowledge model for Spool:

- Tier 1: generated structural knowledge from Fabric semantic-model definitions
- Tier 2: curated authored business artifacts
- embedded examples owned by metric, pattern, and recipe artifacts inside the selected LOB bundle
- bundle validation across both tiers
- runtime loading for one selected LOB
- single-block knowledge projection for the main agent prompt
- recipe contracts and planner-facing selection outputs that later planner work will consume without requiring users to know internal recipe IDs

The governing references are:

- `docs/superpowers/specs/2026-04-06-spool-refined-spec.md`
- `docs/superpowers/specs/2026-04-06-spool-dev-planning-readiness-design.md`
- `docs/plans/codex/2026-04-06-spool-01-harness-semantics-foundation.md`
- `docs/plans/codex/2026-04-06-spool-02-fabric-adapter-foundations.md`
- `docs/plans/codex/2026-04-06-spool-plan-index.md`

## Out Of Scope

- live planner integration beyond exposing recipe-selection contracts
- TUI knowledge visualization
- remote bundle registries
- cross-LOB loading in one runtime session
- durable-memory managed storage and explicit authoring
- Fabric-side mutations
- a separate top-level examples prompt section

## Dependencies

- `docs/plans/codex/2026-04-06-spool-01-harness-semantics-foundation.md`
- `docs/plans/codex/2026-04-06-spool-02-fabric-adapter-foundations.md`

## Contract Impact

This plan implements:

- bundle manifest contract
- Tier 1 schema package contract
- Tier 2 authored artifact contracts
- recipe contract and planner-facing recipe-selection input and output contracts
- bundle validation diagnostics
- selected-LOB runtime loading policy
- single-block knowledge projection for prompt composition

This plan should keep bundle-local IDs stable and explicit. Later plans may consume these contracts, but they should not replace them with display-name-only, prompt-only, or parallel examples sections.

## Requirement Traceability

| Requirement source | Required behavior in this plan | Planned coverage |
|---|---|---|
| Refined spec section 2.1 | Curated LOB knowledge loading, reusable recipes, and local context compaction are in scope for v1 | Tasks 1-7 define the bundle manifest, Tier 1 and Tier 2 contracts, runtime store, projection, and recipe-selection outputs |
| Refined spec knowledge and recipe rules | Recipes are planner-owned, selected through natural-language steering, and should not require users to know internal IDs | Tasks 4, 6, and 7 define recipe contracts, planner-facing selection outputs, and user-preference mapping without exposing recipe IDs as mandatory UX input |
| Planning spec recommended plan set | This plan owns knowledge bundles, indexing, bundle validation, and planner-facing selection contracts as one clean subsystem | Tasks 1-9 keep schema extraction, curated bundle contracts, validation diagnostics, runtime loading, and build tooling inside one knowledge/index boundary |
| Plan index live-validation rule | The live build path must load shared config, exercise a real Fabric seam, and assert canonical Spool contracts rather than transport payloads alone | Task 8 uses shared config and real metadata fetch to build Tier 1, validate a representative bundle, and render the projected knowledge block |

## Execution Invariants

- Tier 1 and Tier 2 are distinct authorities. Tier 1 captures extracted structural facts from Fabric; Tier 2 captures curated business semantics. Neither may silently substitute for the other.
- Bundle-local IDs are the only stable references inside Tier 2. Human-readable names help operators, but cross-artifact links must use bundle-local identifiers.
- Worked examples stay attached to their owning metric, pattern, or recipe artifact. A separate top-level examples dump is explicitly forbidden because it destroys provenance.
- Recipe selection is planner-owned. This plan may produce selection inputs and planner-facing decisions, but it must not collapse into a user-facing "pick recipe id X" requirement.
- Runtime loading is selected-LOB-only in v1. Do not design global multi-bundle prompt composition in this plan.
- Bundle validation diagnostics must explain why a bundle is unusable so authors can fix the content without re-reading source code.

## Live Fixture Inputs And Success Conditions

Task 8 depends on the shared config and fixture contract from the plan index:

- `workspace_name` or `workspace_id` to locate the dev workspace
- `semantic_model_name` or `semantic_model_id` to fetch a representative model definition
- a representative on-disk Tier 2 bundle, such as `fixtures/bundles/finance`

The build-path gate is complete only when one ignored live run proves:

- a real semantic-model definition can be fetched from the configured workspace
- TMDL extraction produces a non-empty `Tier1SchemaPackage`
- one representative Tier 2 bundle validates against that generated Tier 1 package
- runtime loading can provide both full-bundle and Tier-1-only fallback behavior
- the rendered knowledge projection keeps examples under their owning artifacts rather than emitting a separate examples section

## Handoff Artifacts For Later Plans

- stable knowledge contracts and validation diagnostics consumed by planner, TUI, and export work
- runtime store and single-block projection helpers used during prompt composition
- recipe-selection policy outputs that later plans can render in task language
- an architecture note recording the Tier 1 versus Tier 2 split and selected-LOB runtime scope

## Integration Validation

Real validation gate:

- use the dev Fabric workspace to fetch one real semantic-model definition
- run `spool-index` to parse that definition and emit a Tier 1 schema package
- validate one representative Tier 2 bundle from disk against that Tier 1 output
- load the resulting bundle through `spool-knowledge`
- prove Tier 1-only fallback remains usable when Tier 2 is absent
- prove recipe lookup, planner-facing recipe selection, and natural-language user-preference mapping work against the loaded bundle without planner-specific hacks
- render one knowledge projection from the loaded bundle and prove metric, pattern, and recipe examples appear through that single projection without a separate examples payload

## Open Items / Deferred Decisions

### Owned By This Plan

- exact on-disk file shape for Tier 1 schema artifacts
- exact authored YAML or JSON shapes for Tier 2 artifact families
- exact authored shape for embedded examples on metric, pattern, and recipe artifacts
- exact bundle-validation diagnostic format
- exact TMDL parsing boundary between `spool-index` and `spool-fabric`

### Deferred To Later Plans

- planner-side recipe auto-selection behavior in live sessions beyond the shared selection contracts
- TUI visualization of bundle coverage and knowledge provenance
- remote or shared bundle distribution
- compaction policy for oversized knowledge projection

### Review Triggers

- if Tier 1 artifact shapes are not expressive enough to represent tables, columns, measures, relationships, and aliases together
- if recipe references need artifact or evidence classes not available in the canonical contract layer
- if runtime selected-LOB loading requires hidden cross-LOB state
- if runtime prompt composition requires a separate examples lane instead of one selected-LOB knowledge projection
- if real semantic-model definitions require a different parse boundary than the current `spool-index` design allows

## File Structure

| Path | Responsibility |
|---|---|
| `spool/spool-knowledge/Cargo.toml` | runtime knowledge crate manifest |
| `spool/spool-knowledge/src/lib.rs` | knowledge exports |
| `spool/spool-knowledge/src/manifest.rs` | bundle manifest types |
| `spool/spool-knowledge/src/tier1.rs` | Tier 1 schema package types |
| `spool/spool-knowledge/src/tier2.rs` | Tier 2 context, metric, rule, and pattern contracts with embedded examples where applicable |
| `spool/spool-knowledge/src/recipe.rs` | recipe contracts plus planner-facing selection request and decision types |
| `spool/spool-knowledge/src/diagnostics.rs` | bundle-validation diagnostics |
| `spool/spool-knowledge/src/validator.rs` | cross-tier validation rules |
| `spool/spool-knowledge/src/store.rs` | selected-LOB runtime store and fallback loading |
| `spool/spool-knowledge/src/projection.rs` | selected-LOB knowledge projection for main-prompt composition |
| `spool/spool-knowledge/tests/*.rs` | manifest, contract, store, and policy tests |
| `spool/spool-index/Cargo.toml` | indexer binary manifest |
| `spool/spool-index/src/main.rs` | CLI entrypoint |
| `spool/spool-index/src/build.rs` | schema-build orchestration |
| `spool/spool-index/src/tmdl.rs` | TMDL parse and extraction helpers |
| `spool/spool-index/tests/bundle_validation.rs` | bundle-validation tests |
| `spool/spool-index/tests/live_schema_build.rs` | ignored live schema-build test |
| `spool/docs/architecture/knowledge-model.md` | architecture note for bundle and index boundaries |

### Task 1: Add The Knowledge And Indexer Skeletons

**Files:**
- Modify: `spool/Cargo.toml`
- Create: `spool/spool-knowledge/Cargo.toml`
- Create: `spool/spool-knowledge/src/lib.rs`
- Create: `spool/spool-knowledge/src/manifest.rs`
- Create: `spool/spool-index/Cargo.toml`
- Create: `spool/spool-index/src/main.rs`
- Create: `spool/spool-knowledge/tests/manifest_roundtrip.rs`

- [ ] **Step 1: Write the failing manifest roundtrip test**

Create `spool/spool-knowledge/tests/manifest_roundtrip.rs`:

```rust
use spool_knowledge::BundleManifest;

#[test]
fn manifest_roundtrips_with_declared_artifact_classes_and_recipe_ids() {
    let manifest = BundleManifest {
        bundle_id: "bundle_finance".into(),
        lob_id: "finance".into(),
        version: "2026.04.06".into(),
        display_name: "Finance".into(),
        default_workspace_scope: Some("Executive BI".into()),
        tier1_schema_version: "2026.04.06".into(),
        tier2_bundle_version: "2026.04.06".into(),
        build_timestamp: "2026-04-06T10:00:00Z".into(),
        source_summary: "generated from finance semantic model and curated bundle".into(),
        declared_artifact_classes: vec![
            "context".into(),
            "metric".into(),
            "rule".into(),
            "pattern".into(),
            "recipe".into(),
        ],
        declared_recipe_ids: vec!["report_number_mismatch".into()],
    };

    let json = serde_json::to_string_pretty(&manifest).unwrap();
    let restored: BundleManifest = serde_json::from_str(&json).unwrap();

    assert_eq!(restored, manifest);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run:
```bash
cd spool
cargo test -p spool-knowledge manifest_roundtrips_with_declared_artifact_classes_and_recipe_ids
```

Expected: FAIL because the knowledge crate does not exist yet.

- [ ] **Step 3: Implement the crate skeleton and manifest contract**

Create `spool/spool-knowledge/src/manifest.rs`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct BundleManifest {
    pub bundle_id: String,
    pub lob_id: String,
    pub version: String,
    pub display_name: String,
    pub default_workspace_scope: Option<String>,
    pub tier1_schema_version: String,
    pub tier2_bundle_version: String,
    pub build_timestamp: String,
    pub source_summary: String,
    pub declared_artifact_classes: Vec<String>,
    pub declared_recipe_ids: Vec<String>,
}
```

Create `spool/spool-knowledge/src/lib.rs`:

```rust
mod manifest;

pub use manifest::BundleManifest;
```

Create `spool/spool-index/src/main.rs`:

```rust
use clap::Parser;

#[derive(Parser)]
struct Args {
    #[arg(long)]
    workspace: String,
}

fn main() {
    let args = Args::parse();
    println!("{}", args.workspace);
}
```

- [ ] **Step 4: Run test to verify it passes**

Run:
```bash
cd spool
cargo test -p spool-knowledge manifest_roundtrips_with_declared_artifact_classes_and_recipe_ids
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
cd spool
git add Cargo.toml spool-knowledge spool-index
git commit -m "feat: add spool knowledge and indexer skeletons"
```

### Task 2: Define The Tier 1 Schema Package

**Files:**
- Create: `spool/spool-knowledge/src/tier1.rs`
- Modify: `spool/spool-knowledge/src/lib.rs`
- Create: `spool/spool-knowledge/tests/tier1_roundtrip.rs`

- [ ] **Step 1: Write the failing Tier 1 roundtrip test**

Create `spool/spool-knowledge/tests/tier1_roundtrip.rs`:

```rust
use spool_knowledge::{RelationshipDef, TableDef, Tier1SchemaPackage};

#[test]
fn tier1_package_roundtrips_with_tables_measures_columns_relationships_and_aliases() {
    let package = Tier1SchemaPackage {
        semantic_models: vec!["sales_model".into()],
        tables: vec![TableDef {
            table_id: "sales".into(),
            display_name: "Sales".into(),
            column_ids: vec!["sales.order_date".into(), "sales.revenue".into()],
            measure_ids: vec!["sales.revenue_total".into()],
            aliases: vec!["fact sales".into()],
        }],
        columns: vec!["sales.order_date".into(), "sales.revenue".into()],
        measures: vec!["sales.revenue_total".into()],
        relationships: vec![RelationshipDef {
            relationship_id: "sales_to_calendar".into(),
            from_column_id: "sales.order_date".into(),
            to_column_id: "calendar.date".into(),
        }],
        aliases: vec!["revenue".into(), "bookings".into()],
    };

    let json = serde_json::to_string_pretty(&package).unwrap();
    let restored: Tier1SchemaPackage = serde_json::from_str(&json).unwrap();

    assert_eq!(restored, package);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run:
```bash
cd spool
cargo test -p spool-knowledge tier1_package_roundtrips_with_tables_measures_columns_relationships_and_aliases
```

Expected: FAIL because the Tier 1 contract types do not exist.

- [ ] **Step 3: Implement the Tier 1 schema contract**

Create `spool/spool-knowledge/src/tier1.rs`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TableDef {
    pub table_id: String,
    pub display_name: String,
    pub column_ids: Vec<String>,
    pub measure_ids: Vec<String>,
    pub aliases: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RelationshipDef {
    pub relationship_id: String,
    pub from_column_id: String,
    pub to_column_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Tier1SchemaPackage {
    pub semantic_models: Vec<String>,
    pub tables: Vec<TableDef>,
    pub columns: Vec<String>,
    pub measures: Vec<String>,
    pub relationships: Vec<RelationshipDef>,
    pub aliases: Vec<String>,
}
```

- [ ] **Step 4: Run test to verify it passes**

Run:
```bash
cd spool
cargo test -p spool-knowledge tier1_package_roundtrips_with_tables_measures_columns_relationships_and_aliases
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
cd spool
git add spool-knowledge
git commit -m "feat: add spool tier1 schema package contracts"
```

### Task 3: Define Tier 2 Artifact Families And Embedded Examples

**Files:**
- Create: `spool/spool-knowledge/src/tier2.rs`
- Modify: `spool/spool-knowledge/src/lib.rs`
- Create: `spool/spool-knowledge/tests/tier2_roundtrip.rs`

- [ ] **Step 1: Write the failing Tier 2 roundtrip test**

Create `spool/spool-knowledge/tests/tier2_roundtrip.rs`:

```rust
use spool_knowledge::{ContextDef, MetricDef, PatternDef, RuleDef};

#[test]
fn tier2_artifacts_roundtrip_with_bundle_local_ids() {
    let context = ContextDef {
        context_id: "ctx_quarterly_reporting".into(),
        name: "Quarterly Reporting".into(),
        relationships: vec!["metric_revenue".into()],
        usage_notes: vec!["Use for executive quarter-close analysis.".into()],
    };
    let metric = MetricDef {
        metric_id: "metric_revenue".into(),
        name: "Revenue".into(),
        aliases: vec!["sales".into()],
        linked_measure_ids: vec!["sales.revenue_total".into()],
        worked_examples: vec!["monthly revenue variance drilldown".into()],
    };
    let rule = RuleDef {
        rule_id: "rule_exclude_voided_orders".into(),
        name: "Exclude Voided Orders".into(),
        logic_summary: "Revenue excludes voided orders.".into(),
    };
    let pattern = PatternDef {
        pattern_id: "pattern_compare_report_and_model".into(),
        name: "Compare Report And Model".into(),
        anti_patterns: vec!["using display names as identifiers".into()],
        worked_examples: vec!["compare quarterly board slide to semantic-model measure output".into()],
    };

    assert_eq!(serde_json::from_str::<ContextDef>(&serde_json::to_string(&context).unwrap()).unwrap(), context);
    assert_eq!(serde_json::from_str::<MetricDef>(&serde_json::to_string(&metric).unwrap()).unwrap(), metric);
    assert_eq!(serde_json::from_str::<RuleDef>(&serde_json::to_string(&rule).unwrap()).unwrap(), rule);
    assert_eq!(serde_json::from_str::<PatternDef>(&serde_json::to_string(&pattern).unwrap()).unwrap(), pattern);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run:
```bash
cd spool
cargo test -p spool-knowledge tier2_artifacts_roundtrip_with_bundle_local_ids
```

Expected: FAIL because the Tier 2 contracts do not exist.

- [ ] **Step 3: Implement the Tier 2 artifact contracts**

Create `spool/spool-knowledge/src/tier2.rs`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ContextDef {
    pub context_id: String,
    pub name: String,
    pub relationships: Vec<String>,
    pub usage_notes: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct MetricDef {
    pub metric_id: String,
    pub name: String,
    pub aliases: Vec<String>,
    pub linked_measure_ids: Vec<String>,
    pub worked_examples: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RuleDef {
    pub rule_id: String,
    pub name: String,
    pub logic_summary: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PatternDef {
    pub pattern_id: String,
    pub name: String,
    pub anti_patterns: Vec<String>,
    pub worked_examples: Vec<String>,
}
```

- [ ] **Step 4: Run test to verify it passes**

Run:
```bash
cd spool
cargo test -p spool-knowledge tier2_artifacts_roundtrip_with_bundle_local_ids
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
cd spool
git add spool-knowledge
git commit -m "feat: add spool tier2 artifact contracts"
```

### Task 4: Define Recipe Contracts And Recipe-Selection Policy Outputs

**Files:**
- Create: `spool/spool-knowledge/src/recipe.rs`
- Modify: `spool/spool-knowledge/src/lib.rs`
- Create: `spool/spool-knowledge/tests/recipe_roundtrip.rs`

- [ ] **Step 1: Write the failing recipe contract test**

Create `spool/spool-knowledge/tests/recipe_roundtrip.rs`:

```rust
use spool_knowledge::{RecipeDef, RecipeSelectionOutcome};

#[test]
fn recipe_roundtrips_with_expected_evidence_and_validation_expectations() {
    let recipe = RecipeDef {
        recipe_id: "report_number_mismatch".into(),
        name: "Report Number Mismatch".into(),
        intent: "Diagnose why a report value does not match expected totals.".into(),
        applicability_conditions: vec!["report number is disputed".into()],
        required_inputs: vec!["report".into()],
        recommended_artifact_targets: vec!["report".into(), "measure".into()],
        expected_evidence_classes: vec![
            "report_metadata".into(),
            "measure_definition".into(),
            "dax_query_result".into(),
        ],
        validation_expectations: vec!["at least one direct query-backed comparison".into()],
        ordered_investigation_flow: vec![
            "resolve report".into(),
            "identify backing measure".into(),
            "run diagnostic validation".into(),
        ],
        anti_patterns: vec!["jumping to warehouse validation without narrowing scope".into()],
        worked_examples: vec!["quarterly revenue mismatch".into()],
    };

    let json = serde_json::to_string_pretty(&recipe).unwrap();
    let restored: RecipeDef = serde_json::from_str(&json).unwrap();

    assert_eq!(restored, recipe);
    assert_eq!(RecipeSelectionOutcome::AutoSelect.as_str(), "auto_select");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run:
```bash
cd spool
cargo test -p spool-knowledge recipe_roundtrips_with_expected_evidence_and_validation_expectations
```

Expected: FAIL because the recipe contracts do not exist.

- [ ] **Step 3: Implement the recipe and selection-policy contracts**

Create `spool/spool-knowledge/src/recipe.rs`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RecipeDef {
    pub recipe_id: String,
    pub name: String,
    pub intent: String,
    pub applicability_conditions: Vec<String>,
    pub required_inputs: Vec<String>,
    pub recommended_artifact_targets: Vec<String>,
    pub expected_evidence_classes: Vec<String>,
    pub validation_expectations: Vec<String>,
    pub ordered_investigation_flow: Vec<String>,
    pub anti_patterns: Vec<String>,
    pub worked_examples: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecipeSelectionOutcome {
    AutoSelect,
    Suggest,
    DoNotUse,
    UserRequestedOverride,
}

impl RecipeSelectionOutcome {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::AutoSelect => "auto_select",
            Self::Suggest => "suggest",
            Self::DoNotUse => "do_not_use",
            Self::UserRequestedOverride => "user_requested_override",
        }
    }
}
```

- [ ] **Step 4: Run test to verify it passes**

Run:
```bash
cd spool
cargo test -p spool-knowledge recipe_roundtrips_with_expected_evidence_and_validation_expectations
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
cd spool
git add spool-knowledge
git commit -m "feat: add spool recipe contracts and selection outcomes"
```

### Task 5: Implement Cross-Tier Bundle Validation Diagnostics

**Files:**
- Create: `spool/spool-knowledge/src/diagnostics.rs`
- Create: `spool/spool-knowledge/src/validator.rs`
- Modify: `spool/spool-knowledge/src/lib.rs`
- Create: `spool/spool-index/tests/bundle_validation.rs`

- [ ] **Step 1: Write the failing bundle-validation tests**

Create `spool/spool-index/tests/bundle_validation.rs`:

```rust
use spool_knowledge::{
    validate_bundle,
    BundleManifest,
    BundleValidationSeverity,
    MetricDef,
    RecipeDef,
    Tier1SchemaPackage,
};

#[test]
fn validation_rejects_missing_measure_links_and_unknown_evidence_classes() {
    let manifest = BundleManifest {
        bundle_id: "bundle_finance".into(),
        lob_id: "finance".into(),
        version: "2026.04.06".into(),
        display_name: "Finance".into(),
        default_workspace_scope: None,
        tier1_schema_version: "2026.04.06".into(),
        tier2_bundle_version: "2026.04.06".into(),
        build_timestamp: "2026-04-06T10:00:00Z".into(),
        source_summary: "bundle".into(),
        declared_artifact_classes: vec!["metric".into(), "recipe".into()],
        declared_recipe_ids: vec!["report_number_mismatch".into()],
    };
    let tier1 = Tier1SchemaPackage {
        semantic_models: vec!["sales_model".into()],
        tables: vec![],
        columns: vec![],
        measures: vec!["sales.revenue_total".into()],
        relationships: vec![],
        aliases: vec![],
    };
    let metrics = vec![MetricDef {
        metric_id: "metric_margin".into(),
        name: "Margin".into(),
        aliases: vec![],
        linked_measure_ids: vec!["sales.margin_total".into()],
        worked_examples: vec![],
    }];
    let recipes = vec![RecipeDef {
        recipe_id: "report_number_mismatch".into(),
        name: "Report Number Mismatch".into(),
        intent: "Diagnose mismatches".into(),
        applicability_conditions: vec![],
        required_inputs: vec![],
        recommended_artifact_targets: vec!["report".into()],
        expected_evidence_classes: vec!["made_up_evidence".into()],
        validation_expectations: vec![],
        ordered_investigation_flow: vec![],
        anti_patterns: vec![],
        worked_examples: vec![],
    }];

    let diagnostics = validate_bundle(&manifest, &tier1, &metrics, &recipes);

    assert_eq!(diagnostics.len(), 2);
    assert_eq!(diagnostics[0].severity, BundleValidationSeverity::Error);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run:
```bash
cd spool
cargo test -p spool-index validation_rejects_missing_measure_links_and_unknown_evidence_classes
```

Expected: FAIL because the validation diagnostics do not exist.

- [ ] **Step 3: Implement the diagnostics and validator**

Create `spool/spool-knowledge/src/diagnostics.rs`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BundleValidationSeverity {
    Error,
    Warning,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct BundleValidationDiagnostic {
    pub severity: BundleValidationSeverity,
    pub code: String,
    pub message: String,
}
```

Create `spool/spool-knowledge/src/validator.rs` with validation rules for at least:

- missing measure links
- malformed bundle structure
- duplicate bundle-local IDs
- alias collisions
- broken recipe references
- unknown evidence classes in recipes

Return `Vec<BundleValidationDiagnostic>`.

- [ ] **Step 4: Run test to verify it passes**

Run:
```bash
cd spool
cargo test -p spool-index validation_rejects_missing_measure_links_and_unknown_evidence_classes
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
cd spool
git add spool-knowledge spool-index
git commit -m "feat: add spool bundle validation diagnostics"
```

### Task 6: Implement The Selected-LOB Runtime Store, Tier 1-Only Fallback, And Knowledge Projection

**Files:**
- Create: `spool/spool-knowledge/src/store.rs`
- Create: `spool/spool-knowledge/src/projection.rs`
- Modify: `spool/spool-knowledge/src/lib.rs`
- Create: `spool/spool-knowledge/tests/store_loading.rs`

- [ ] **Step 1: Write the failing runtime-store tests**

Create `spool/spool-knowledge/tests/store_loading.rs`:

```rust
use spool_knowledge::{
    render_knowledge_projection,
    BundleManifest,
    KnowledgeStore,
    MetricDef,
    PatternDef,
    RecipeDef,
    Tier1SchemaPackage,
};

#[test]
fn loader_returns_tier1_only_when_tier2_is_missing() {
    let tier1 = Tier1SchemaPackage {
        semantic_models: vec!["sales_model".into()],
        tables: vec![],
        columns: vec![],
        measures: vec!["sales.revenue_total".into()],
        relationships: vec![],
        aliases: vec!["revenue".into()],
    };

    let store = KnowledgeStore::tier1_only("finance", tier1.clone());

    assert_eq!(store.selected_lob_id(), "finance");
    assert_eq!(store.bundle_manifest(), None);
    assert_eq!(store.tier1(), &tier1);
    assert!(store.coverage_note().contains("tier1 only"));
}

#[test]
fn loader_returns_manifest_when_bundle_is_present() {
    let tier1 = Tier1SchemaPackage {
        semantic_models: vec!["sales_model".into()],
        tables: vec![],
        columns: vec![],
        measures: vec!["sales.revenue_total".into()],
        relationships: vec![],
        aliases: vec![],
    };
    let manifest = BundleManifest {
        bundle_id: "bundle_finance".into(),
        lob_id: "finance".into(),
        version: "2026.04.06".into(),
        display_name: "Finance".into(),
        default_workspace_scope: Some("Executive BI".into()),
        tier1_schema_version: "2026.04.06".into(),
        tier2_bundle_version: "2026.04.06".into(),
        build_timestamp: "2026-04-06T10:00:00Z".into(),
        source_summary: "bundle".into(),
        declared_artifact_classes: vec!["metric".into(), "recipe".into()],
        declared_recipe_ids: vec!["report_number_mismatch".into()],
    };

    let store = KnowledgeStore::with_bundle(manifest.clone(), tier1);

    assert_eq!(store.bundle_manifest(), Some(&manifest));
}

#[test]
fn knowledge_projection_embeds_examples_inside_owned_artifacts() {
    let tier1 = Tier1SchemaPackage {
        semantic_models: vec!["sales_model".into()],
        tables: vec![],
        columns: vec![],
        measures: vec!["sales.revenue_total".into()],
        relationships: vec![],
        aliases: vec!["revenue".into()],
    };
    let metrics = vec![MetricDef {
        metric_id: "metric_revenue".into(),
        name: "Revenue".into(),
        aliases: vec!["sales".into()],
        linked_measure_ids: vec!["sales.revenue_total".into()],
        worked_examples: vec!["monthly revenue variance drilldown".into()],
    }];
    let patterns = vec![PatternDef {
        pattern_id: "pattern_compare_report_and_model".into(),
        name: "Compare Report And Model".into(),
        anti_patterns: vec!["using display names as identifiers".into()],
        worked_examples: vec!["compare report total to semantic-model total".into()],
    }];
    let recipes = vec![RecipeDef {
        recipe_id: "report_number_mismatch".into(),
        name: "Report Number Mismatch".into(),
        intent: "Diagnose mismatches".into(),
        applicability_conditions: vec![],
        required_inputs: vec![],
        recommended_artifact_targets: vec![],
        expected_evidence_classes: vec![],
        validation_expectations: vec![],
        ordered_investigation_flow: vec![],
        anti_patterns: vec![],
        worked_examples: vec!["quarterly revenue mismatch".into()],
    }];

    let projection = render_knowledge_projection(&tier1, &metrics, &patterns, &recipes);

    assert!(projection.contains("monthly revenue variance drilldown"));
    assert!(projection.contains("compare report total to semantic-model total"));
    assert!(projection.contains("quarterly revenue mismatch"));
    assert!(!projection.contains("\n# Examples\n"));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run:
```bash
cd spool
cargo test -p spool-knowledge store_loading
```

Expected: FAIL because the runtime store and knowledge projection do not exist.

- [ ] **Step 3: Implement the runtime store and projection**

Create `spool/spool-knowledge/src/store.rs`:

```rust
use crate::{BundleManifest, Tier1SchemaPackage};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LoadedBundle {
    pub manifest: BundleManifest,
    pub metrics: Vec<crate::MetricDef>,
    pub patterns: Vec<crate::PatternDef>,
    pub recipes: Vec<crate::RecipeDef>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KnowledgeStore {
    selected_lob_id: String,
    manifest: Option<BundleManifest>,
    tier1: Tier1SchemaPackage,
    coverage_note: String,
}
```

Implement:

- `KnowledgeStore::tier1_only(selected_lob_id, tier1)`
- `KnowledgeStore::with_bundle(manifest, tier1)`
- `load_bundle_from_disk(path) -> anyhow::Result<LoadedBundle>`
- `selected_lob_id()`
- `bundle_manifest()`
- `tier1()`
- `coverage_note()`

Create `spool/spool-knowledge/src/projection.rs` with:

```rust
use crate::{MetricDef, PatternDef, RecipeDef, Tier1SchemaPackage};

pub fn render_knowledge_projection(
    tier1: &Tier1SchemaPackage,
    metrics: &[MetricDef],
    patterns: &[PatternDef],
    recipes: &[RecipeDef],
) -> String
```

Projection rules:

- render one selected-LOB knowledge block for prompt composition
- include Tier 1 structural summary first
- include metric, pattern, and recipe examples only under their owning artifacts
- omit empty example blocks
- do not emit a separate top-level `Examples` section

- [ ] **Step 4: Run test to verify it passes**

Run:
```bash
cd spool
cargo test -p spool-knowledge store_loading
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
cd spool
git add spool-knowledge
git commit -m "feat: add spool runtime knowledge store"
```

### Task 7: Add The Recipe-Selection Policy Evaluator

**Files:**
- Modify: `spool/spool-knowledge/src/recipe.rs`
- Create: `spool/spool-knowledge/tests/recipe_selection.rs`

- [ ] **Step 1: Write the failing recipe-selection tests**

Create `spool/spool-knowledge/tests/recipe_selection.rs`:

```rust
use spool_knowledge::{
    evaluate_recipe_fit, RecipeDef, RecipeSelectionDecision, RecipeSelectionOutcome,
    RecipeSelectionRequest, UserApproachPreference,
};
use spool_model::ValidationFloor;

#[test]
fn evaluator_auto_selects_when_single_recipe_is_clear_best_fit() {
    let recipes = vec![RecipeDef {
        recipe_id: "report_number_mismatch".into(),
        name: "Report Number Mismatch".into(),
        intent: "Diagnose report mismatches".into(),
        applicability_conditions: vec!["report number is disputed".into()],
        required_inputs: vec!["report".into()],
        recommended_artifact_targets: vec!["report".into(), "measure".into()],
        expected_evidence_classes: vec!["report_metadata".into(), "measure_definition".into()],
        validation_expectations: vec!["direct validation".into()],
        ordered_investigation_flow: vec![],
        anti_patterns: vec![],
        worked_examples: vec!["quarterly revenue mismatch".into()],
    }];
    let request = RecipeSelectionRequest {
        problem_summary: "Revenue on the report does not match expected quarter totals.".into(),
        lob_id: "finance".into(),
        workspace: "Executive BI".into(),
        artifact_targets: vec!["report".into(), "measure".into()],
        expected_evidence_classes: vec!["report_metadata".into(), "measure_definition".into()],
        validation_floor: ValidationFloor::DirectValidationRequired,
        user_preference: None,
    };

    let decision = evaluate_recipe_fit(&recipes, &request);

    assert_eq!(
        decision,
        RecipeSelectionDecision {
            outcome: RecipeSelectionOutcome::AutoSelect,
            selected_recipe_id: Some("report_number_mismatch".into()),
            selected_recipe_label: Some("Report Number Mismatch".into()),
            closest_viable_alternative_id: None,
            closest_viable_alternative_label: None,
            rationale: "Single strong-fit recipe matched the task context.".into(),
            user_preference_honored: false,
        }
    );
}

#[test]
fn evaluator_honors_compatible_user_preference_without_requiring_recipe_ids() {
    let recipes = vec![RecipeDef {
        recipe_id: "report_number_mismatch".into(),
        name: "Report Number Mismatch".into(),
        intent: "Diagnose report mismatches".into(),
        applicability_conditions: vec!["report number is disputed".into()],
        required_inputs: vec!["report".into()],
        recommended_artifact_targets: vec!["report".into(), "measure".into()],
        expected_evidence_classes: vec![
            "report_metadata".into(),
            "measure_definition".into(),
            "warehouse_query_result".into(),
        ],
        validation_expectations: vec!["direct validation".into()],
        ordered_investigation_flow: vec![],
        anti_patterns: vec![],
        worked_examples: vec!["quarterly revenue mismatch".into()],
    }];
    let request = RecipeSelectionRequest {
        problem_summary: "Revenue on the report does not match expected quarter totals.".into(),
        lob_id: "finance".into(),
        workspace: "Executive BI".into(),
        artifact_targets: vec!["report".into(), "measure".into()],
        expected_evidence_classes: vec![
            "report_metadata".into(),
            "measure_definition".into(),
            "warehouse_query_result".into(),
        ],
        validation_floor: ValidationFloor::DirectValidationRequired,
        user_preference: Some(UserApproachPreference::InvestigationStyle(
            "Treat this like the usual report total mismatch playbook.".into(),
        )),
    };

    let decision = evaluate_recipe_fit(&recipes, &request);

    assert_eq!(decision.outcome, RecipeSelectionOutcome::UserRequestedOverride);
    assert_eq!(
        decision.selected_recipe_id.as_deref(),
        Some("report_number_mismatch")
    );
    assert!(decision.user_preference_honored);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run:
```bash
cd spool
cargo test -p spool-knowledge recipe_selection
```

Expected: FAIL because the recipe-fit evaluator does not exist.

- [ ] **Step 3: Implement the recipe-fit evaluator**

Extend `spool/spool-knowledge/src/recipe.rs` with:

```rust
use spool_model::ValidationFloor;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UserApproachPreference {
    InvestigationStyle(String),
    KnownPlaybook(String),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecipeSelectionRequest {
    pub problem_summary: String,
    pub lob_id: String,
    pub workspace: String,
    pub artifact_targets: Vec<String>,
    pub expected_evidence_classes: Vec<String>,
    pub validation_floor: ValidationFloor,
    pub user_preference: Option<UserApproachPreference>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecipeSelectionDecision {
    pub outcome: RecipeSelectionOutcome,
    pub selected_recipe_id: Option<String>,
    pub selected_recipe_label: Option<String>,
    pub closest_viable_alternative_id: Option<String>,
    pub closest_viable_alternative_label: Option<String>,
    pub rationale: String,
    pub user_preference_honored: bool,
}

pub fn evaluate_recipe_fit(
    recipes: &[RecipeDef],
    request: &RecipeSelectionRequest,
) -> RecipeSelectionDecision
```

Use this policy:

- score recipes against problem-class, artifact-target, evidence-class, validation-floor, LOB, and workspace compatibility
- return `DoNotUse` when no recipe satisfies the minimum compatibility floor
- return `AutoSelect` when exactly one recipe is a clear strong fit
- return `Suggest` when multiple plausible recipes remain and the planner should surface the approach rather than silently choosing one
- when `user_preference` is present, map the natural-language preference or known playbook to a compatible recipe without requiring the user to know an internal recipe ID
- return `UserRequestedOverride` only when that mapped recipe remains compatible enough for the task even if it is not a perfect fit
- when a user-requested preference or known playbook is incompatible, refuse it explicitly and populate the closest viable alternative fields when one exists
- populate `rationale` with one short planner-facing explanation that later plans can render in task language
- set `user_preference_honored` so later plans can show whether the user steering was followed

- [ ] **Step 4: Run test to verify it passes**

Run:
```bash
cd spool
cargo test -p spool-knowledge recipe_selection
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
cd spool
git add spool-knowledge
git commit -m "feat: add spool recipe fit evaluator"
```

### Task 8: Implement The `spool-index` Build Path

**Files:**
- Create: `spool/spool-index/src/build.rs`
- Create: `spool/spool-index/src/tmdl.rs`
- Modify: `spool/spool-index/src/main.rs`
- Create: `spool/spool-index/tests/live_schema_build.rs`

- [ ] **Step 1: Write the failing live schema-build test**

Create `spool/spool-index/tests/live_schema_build.rs`:

```rust
use spool_index::build::build_schema_package;
use spool_fabric::FabricConfig;
use spool_knowledge::{
    load_bundle_from_disk, render_knowledge_projection, validate_bundle,
    BundleValidationSeverity,
};

#[tokio::test]
#[ignore = "requires dev Fabric workspace config"]
async fn builds_tier1_schema_from_dev_workspace() {
    let config_path = std::env::var("SPOOL_CONFIG_PATH").unwrap_or_else(|_| {
        format!(
            "{}/.config/spool/dev.toml",
            std::env::var("HOME").unwrap()
        )
    });
    let config = FabricConfig::load_from_path(std::path::Path::new(&config_path)).unwrap();
    let package = build_schema_package(&config.workspace_name).await.unwrap();
    let bundle = load_bundle_from_disk("fixtures/bundles/finance").unwrap();
    let diagnostics = validate_bundle(
        &bundle.manifest,
        &package,
        &bundle.metrics,
        &bundle.recipes,
    );
    let projection = render_knowledge_projection(
        &package,
        &bundle.metrics,
        &bundle.patterns,
        &bundle.recipes,
    );

    assert!(!package.semantic_models.is_empty());
    assert!(!package.measures.is_empty());
    assert!(diagnostics.iter().all(|item| item.severity != BundleValidationSeverity::Error));
    assert!(projection.contains("quarterly revenue mismatch"));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run:
```bash
cd spool
cargo test -p spool-index builds_tier1_schema_from_dev_workspace -- --ignored
```

Expected: FAIL because the build path does not exist.

- [ ] **Step 3: Implement the TMDL parse and schema-build path**

Create `spool/spool-index/src/tmdl.rs` with extraction helpers that normalize:

- semantic-model identifiers
- tables
- columns
- measures
- relationships
- aliases when present

Create `spool/spool-index/src/build.rs` with:

- `build_schema_package(workspace: &str) -> anyhow::Result<Tier1SchemaPackage>`

The build path should:

1. fetch or receive a semantic-model definition
2. parse it through `tmdl.rs`
3. produce a `Tier1SchemaPackage`
4. validate one representative Tier 2 bundle from disk against the generated Tier 1 package
5. load the validated bundle through `spool-knowledge`
6. render one selected-LOB knowledge projection and assert that embedded examples appear through owned artifacts rather than a separate examples section

Update `spool/spool-index/src/main.rs` to call `build_schema_package()` and print the number of measures generated.

- [ ] **Step 4: Run the live schema-build validation**

Run:
```bash
cd spool
SPOOL_CONFIG_PATH=~/.config/spool/dev.toml \
cargo test -p spool-index builds_tier1_schema_from_dev_workspace -- --ignored --nocapture
```

Expected: PASS against the dev Fabric workspace configuration, with the ignored integration test exercising the real metadata fetch, Tier 2 bundle validation, runtime loading, recipe lookup surfaces, Tier 1-only fallback, and single-block projection behavior rather than only the raw schema build.

- [ ] **Step 5: Commit**

```bash
cd spool
git add spool-index
git commit -m "feat: add spool schema build path"
```

### Task 9: Write The Knowledge-Model Architecture Note

**Files:**
- Create: `spool/docs/architecture/knowledge-model.md`

- [ ] **Step 1: Write the architecture note**

Create `spool/docs/architecture/knowledge-model.md` describing:

- why Tier 1 and Tier 2 are separate but validated together
- why recipes live in curated knowledge rather than schema output
- why embedded examples stay inside metric, pattern, and recipe artifacts rather than becoming a separate prompt section
- why bundle-local IDs are authoritative for Tier 2 references
- why runtime loading is selected-LOB-only in v1
- why prompt composition consumes one selected-LOB knowledge projection
- what later plans may consume from the recipe-selection policy outputs

- [ ] **Step 2: Review the note for missing concepts**

Run:
```bash
cd spool
rg -n "Tier 1|Tier 2|recipe|selected-LOB|bundle-local|knowledge projection|embedded examples" docs/architecture/knowledge-model.md
```

Expected: the note explicitly mentions all seven concepts.

- [ ] **Step 3: Commit**

```bash
cd spool
git add docs/architecture/knowledge-model.md
git commit -m "docs: add spool knowledge model architecture note"
```

## Self-Review Checklist

- Spec coverage: This plan covers the bundle manifest, Tier 1 schema package, Tier 2 artifact families, embedded examples, recipe contracts, recipe-selection outputs, validation diagnostics, selected-LOB loading, knowledge projection, and the `spool-index` build path.
- Placeholder scan: No step may reduce Tier 1 to measure IDs only, reduce Tier 2 to metrics only, split examples into a separate prompt-time section, or leave recipes as undeclared prose.
- Type consistency: Keep `BundleManifest`, `Tier1SchemaPackage`, `ContextDef`, `MetricDef`, `RuleDef`, `PatternDef`, `RecipeDef`, `RecipeSelectionOutcome`, `BundleValidationDiagnostic`, `KnowledgeStore`, and `render_knowledge_projection()` stable because later planner and TUI plans consume them directly.
