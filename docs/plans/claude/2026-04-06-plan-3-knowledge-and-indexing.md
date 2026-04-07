# Plan 3: Knowledge And Indexing

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Establish the knowledge bundle system for Spool — Tier 1 auto-generated schema knowledge from TMDL, Tier 2 curated business knowledge artifacts, bundle validation, recipe schema, and runtime loading policy — all proven through fixture-backed tests with no live Fabric dependency.

**Architecture:** Two new crates — `spool-knowledge` (shared knowledge types used by both indexer and runtime) and `spool-index` (build-time tool that parses TMDL and generates Tier 1 bundles). Knowledge types are separate from `spool-protocol` because they represent a distinct subsystem with its own validation rules and lifecycle. `spool-protocol` gains only the minimal integration types needed for task contracts to reference knowledge (recipe selection mode already exists from Plan 1).

**Tech Stack:** Rust 2024 edition, serde/serde_json/serde_yaml, chrono, uuid, thiserror, tempfile (dev), walkdir

**Governing spec:** `docs/superpowers/specs/2026-04-06-spool-refined-spec.md`
**Planning readiness:** `docs/superpowers/specs/2026-04-06-spool-dev-planning-readiness-design.md`
**Plan 1 reference:** `docs/plans/claude/2026-04-06-plan-1-harness-semantics-foundation.md`

---

## Plan-Specific Sections

### Subsystem Scope

This plan owns:

- bundle manifest schema (Spec Section 7.3)
- Tier 1 schema package structure — tables, measures, relationships, aliases (Spec Section 7.4)
- Tier 2 curated knowledge package — contexts, metrics, rules, patterns, recipes (Spec Sections 7.5, 8.1-8.7)
- bundle naming and reference rules with stable bundle-local IDs (Spec Section 7.6)
- bundle validation across both tiers — cross-reference integrity, alias collisions, missing measure refs, duplicate IDs (Spec Section 7.10)
- selected-LOB loading policy types (Spec Section 7.7)
- cold-start and fallback behavior types (Spec Sections 7.8, 7.11)
- TMDL parsing foundations — `spool-index` parses fixture TMDL into Tier 1 schema (Spec Section 7.9)
- recipe schema with investigation flow, anti-patterns, worked examples (Spec Section 8.2)
- recipe selection policy types (Spec Section 8.5)
- recipe deviation recording types (Spec Section 8.6)

### Out Of Scope

- live Fabric auth or API calls (Plan 2)
- live TMDL fetching from Fabric REST API (Plan 2 provides the adapter; this plan uses fixture TMDL)
- durable memory subsystem (Plan 6 — Spec Sections 7.12-7.13)
- TUI rendering of knowledge state (Plan 5)
- harness integration — how the planner/generator consume knowledge at runtime (Plan 4 or later integration)
- knowledge prompt composition — the actual prompt assembly from loaded bundles (future plan)

### Dependencies

- **Plan 1** (spool-protocol types): This plan uses `ArtifactType`, `EvidenceClass`, `RecipeSelectionMode` from `spool-protocol`. Plan 1 must be implemented first or the workspace must include `spool-protocol` with those types.
- **No dependency on Plan 2** (Fabric adapters). All TMDL input uses fixture files.

### Contract Impact

This plan **implements** the following governing contracts from the refined spec:

- Knowledge bundle composition contract (Spec Section 7.2)
- Bundle manifest schema (Spec Section 7.3)
- Tier 1 schema package contract (Spec Section 7.4)
- Tier 2 curated knowledge package contract (Spec Section 7.5)
- Bundle naming and reference rules (Spec Section 7.6)
- Selected-LOB loading policy (Spec Section 7.7)
- Cold-start structural awareness (Spec Section 7.8)
- Knowledge build pipeline foundations (Spec Section 7.9)
- Knowledge validation contract (Spec Section 7.10)
- Fallback behavior contract (Spec Section 7.11)
- Recipe shape contract (Spec Section 8.2)
- Recipe selection policy (Spec Section 8.5)
- Deviation recording contract (Spec Section 8.6)

This plan **does not modify** any contracts from Plan 1. It adds new types alongside the existing `spool-protocol` types.

### Validation

Plan 3 is proven through:

- schema tests: all knowledge types serialize/deserialize round-trip (YAML and JSON)
- TMDL parsing tests: fixture TMDL files produce correct Tier 1 schema packages
- bundle validation tests: cross-reference integrity, alias collision detection, missing measure refs, duplicate IDs
- recipe schema tests: recipe YAML round-trip, deviation recording
- integration fixture: a complete sample LOB bundle with Tier 1 + Tier 2 content passes full validation
- cold-start tests: missing Tier 2 produces degraded-but-valid bundle state
- loading policy tests: selected LOB loads correctly, unselected LOBs are not loaded

No live systems. No network. All fixture-backed.

**Integration validation justification (per planning readiness addendum Section 5):** Plan 3 is intentionally isolated from live Fabric systems. It consumes TMDL as input files, not through live Fabric API calls. Live TMDL fetching is owned by Plan 2's adapter; this plan uses fixture TMDL files. Fixture-only validation is acceptable here because the live external seam (Fabric API) is not owned by this plan.

### Open Items

**Owned by this plan:**

- exact TMDL parsing subset needed for v1 Tier 1 (resolved during implementation — start with tables, measures, relationships)
- whether Tier 2 artifacts use YAML or TOML (resolved: YAML, matching existing knowledge template patterns)
- exact alias collision rules (resolved during implementation)

**Deferred to later plans:**

- live TMDL fetching from Fabric API (Plan 2 adapter)
- prompt composition from loaded bundles (future plan)
- durable memory integration with knowledge (Plan 6)
- how the evaluator loop consumes recipes at runtime (Plan 4 or integration plan)

**Review triggers:**

- if Plan 2 reveals that TMDL structure differs from fixture assumptions, revisit Tier 1 parser
- if Plan 4 reveals that recipe consumption needs richer types, revisit recipe schema
- if real LOB bundles reveal validation gaps, extend validation rules

---

## Task 1: Workspace Extension — spool-knowledge and spool-index Crates

**Files:**

- Modify: `spool/Cargo.toml`
- Create: `spool/spool-knowledge/Cargo.toml`
- Create: `spool/spool-knowledge/src/lib.rs`
- Create: `spool/spool-index/Cargo.toml`
- Create: `spool/spool-index/src/lib.rs`

**Step 1: Extend workspace Cargo.toml**

Add the two new crate members to the existing workspace:

```toml
# spool/Cargo.toml — add to [workspace] members list
[workspace]
members = [
    "spool-protocol",
    "spool-core",
    "spool-knowledge",
    "spool-index",
]
resolver = "2"

[workspace.package]
version = "0.1.0"
edition = "2024"
license = "Apache-2.0"

[workspace.dependencies]
spool-protocol = { path = "spool-protocol" }
spool-knowledge = { path = "spool-knowledge" }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
serde_yaml = "0.9"
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1", features = ["v4", "serde"] }
tokio = { version = "1", features = ["full"] }
async-trait = "0.1"
thiserror = "2"
walkdir = "2"
```

**Step 2: Create spool-knowledge crate**

```toml
# spool/spool-knowledge/Cargo.toml
[package]
name = "spool-knowledge"
version.workspace = true
edition.workspace = true
license.workspace = true

[dependencies]
spool-protocol = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
serde_yaml = { workspace = true }
chrono = { workspace = true }
uuid = { workspace = true }
thiserror = { workspace = true }
```

```rust
// spool/spool-knowledge/src/lib.rs
pub mod bundle;
pub mod tier1;
pub mod tier2;
pub mod recipe;
pub mod validation;
pub mod loading;
pub mod error;
```

**Step 3: Create spool-index crate**

```toml
# spool/spool-index/Cargo.toml
[package]
name = "spool-index"
version.workspace = true
edition.workspace = true
license.workspace = true

[dependencies]
spool-protocol = { workspace = true }
spool-knowledge = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
serde_yaml = { workspace = true }
chrono = { workspace = true }
thiserror = { workspace = true }
walkdir = { workspace = true }

[dev-dependencies]
tempfile = "3"
```

```rust
// spool/spool-index/src/lib.rs
pub mod tmdl;
pub mod tier1_gen;
pub mod bundle_builder;
pub mod error;
```

**Step 4: Create placeholder modules**

Create empty files for each module declared in both lib.rs files. Each file should contain only a comment:

```rust
// placeholder — implemented in later tasks
```

Also create the error modules:

```rust
// spool/spool-knowledge/src/error.rs
use thiserror::Error;

#[derive(Debug, Error)]
pub enum KnowledgeError {
    #[error("bundle error: {0}")]
    Bundle(String),

    #[error("validation error: {0}")]
    Validation(String),

    #[error("loading error: {0}")]
    Loading(String),

    #[error("recipe error: {0}")]
    Recipe(String),

    #[error("tier1 error: {0}")]
    Tier1(String),

    #[error("tier2 error: {0}")]
    Tier2(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("yaml error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
}
```

```rust
// spool/spool-index/src/error.rs
use thiserror::Error;

#[derive(Debug, Error)]
pub enum IndexError {
    #[error("tmdl parse error: {0}")]
    TmdlParse(String),

    #[error("tier1 generation error: {0}")]
    Tier1Generation(String),

    #[error("bundle build error: {0}")]
    BundleBuild(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("knowledge error: {0}")]
    Knowledge(#[from] spool_knowledge::error::KnowledgeError),
}
```

**Step 5: Verify build**

Run: `cd spool && cargo check`
Expected: compiles with no errors

**Step 6: Commit**

```bash
git add spool/
git commit -m "feat(spool): scaffold spool-knowledge and spool-index crates for knowledge and indexing subsystem"
```

---

## Task 2: Bundle Manifest Schema

**Files:**

- Create: `spool/spool-knowledge/src/bundle.rs`

**Step 1: Write the failing test**

Add to `spool/spool-knowledge/src/bundle.rs`:

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// placeholder — implementation goes here in Step 3

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn bundle_manifest_round_trip_json() {
        let manifest = BundleManifest {
            bundle_id: "bundle_finance_q1_2026".into(),
            lob_id: "finance".into(),
            version: "1.0.0".into(),
            display_name: "Finance LOB Bundle".into(),
            default_workspace_scope: Some("Executive BI".into()),
            tier1_schema_version: "1.0.0".into(),
            tier2_bundle_version: "1.0.0".into(),
            build_timestamp: Utc::now(),
            source_summary: "Generated from Sales Model TMDL + curated finance knowledge".into(),
            declared_artifact_classes: vec![
                "context".into(),
                "metric".into(),
                "rule".into(),
                "pattern".into(),
                "recipe".into(),
            ],
            declared_recipe_ids: vec![
                "report_number_mismatch".into(),
                "measure_drift_detection".into(),
            ],
        };

        let json = serde_json::to_string_pretty(&manifest).unwrap();
        let restored: BundleManifest = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.bundle_id, manifest.bundle_id);
        assert_eq!(restored.lob_id, "finance");
        assert_eq!(restored.declared_artifact_classes.len(), 5);
        assert_eq!(restored.declared_recipe_ids.len(), 2);
    }

    #[test]
    fn bundle_manifest_round_trip_yaml() {
        let manifest = BundleManifest {
            bundle_id: "bundle_finance_q1_2026".into(),
            lob_id: "finance".into(),
            version: "1.0.0".into(),
            display_name: "Finance LOB Bundle".into(),
            default_workspace_scope: None,
            tier1_schema_version: "1.0.0".into(),
            tier2_bundle_version: "1.0.0".into(),
            build_timestamp: Utc::now(),
            source_summary: "Generated from fixture TMDL".into(),
            declared_artifact_classes: vec!["metric".into()],
            declared_recipe_ids: vec![],
        };

        let yaml = serde_yaml::to_string(&manifest).unwrap();
        let restored: BundleManifest = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(restored.bundle_id, "bundle_finance_q1_2026");
        assert!(restored.default_workspace_scope.is_none());
    }

    #[test]
    fn bundle_manifest_no_workspace_scope_serializes_as_null() {
        let manifest = BundleManifest {
            bundle_id: "bundle_test".into(),
            lob_id: "test".into(),
            version: "0.1.0".into(),
            display_name: "Test Bundle".into(),
            default_workspace_scope: None,
            tier1_schema_version: "0.1.0".into(),
            tier2_bundle_version: "0.1.0".into(),
            build_timestamp: Utc::now(),
            source_summary: "test".into(),
            declared_artifact_classes: vec![],
            declared_recipe_ids: vec![],
        };

        let json = serde_json::to_string(&manifest).unwrap();
        let restored: BundleManifest = serde_json::from_str(&json).unwrap();
        assert!(restored.default_workspace_scope.is_none());
        assert!(restored.declared_artifact_classes.is_empty());
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cd spool && cargo test -p spool-knowledge -- bundle`
Expected: FAIL — `BundleManifest` not defined yet

**Step 3: Write the implementation**

Replace the placeholder in `spool/spool-knowledge/src/bundle.rs` with the full file:

```rust
// spool/spool-knowledge/src/bundle.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// The authoritative index for what a LOB bundle contains.
/// Spec Section 7.3.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleManifest {
    pub bundle_id: String,
    pub lob_id: String,
    pub version: String,
    pub display_name: String,
    pub default_workspace_scope: Option<String>,
    pub tier1_schema_version: String,
    pub tier2_bundle_version: String,
    pub build_timestamp: DateTime<Utc>,
    pub source_summary: String,
    pub declared_artifact_classes: Vec<String>,
    pub declared_recipe_ids: Vec<String>,
}

/// The status of a loaded knowledge bundle in the runtime.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BundleLoadStatus {
    /// Both tiers loaded successfully.
    FullyLoaded,
    /// Tier 1 loaded but Tier 2 is missing or empty.
    Tier1Only,
    /// Neither tier available — cold-start / fallback mode.
    ColdStart,
    /// Bundle failed validation — loaded with warnings.
    LoadedWithWarnings { warning_count: usize },
}

/// Top-level representation of a loaded LOB bundle in memory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadedBundle {
    pub manifest: BundleManifest,
    pub load_status: BundleLoadStatus,
    pub tier1: Option<crate::tier1::Tier1SchemaPackage>,
    pub tier2: Option<crate::tier2::Tier2Package>,
    pub validation_warnings: Vec<String>,
}

// tests at bottom (from Step 1)
```

**Step 4: Run test to verify it passes**

Run: `cd spool && cargo test -p spool-knowledge -- bundle`
Expected: 3 tests PASS

**Step 5: Commit**

```bash
git add spool/spool-knowledge/src/bundle.rs
git commit -m "feat(spool-knowledge): bundle manifest schema with load status and LOB bundle structure"
```

---

## Task 3: Tier 1 Schema Package Types

**Files:**

- Create: `spool/spool-knowledge/src/tier1.rs`

**Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn sample_tier1() -> Tier1SchemaPackage {
        Tier1SchemaPackage {
            schema_version: "1.0.0".into(),
            models: vec![SemanticModelSchema {
                model_id: "model_sales".into(),
                model_name: "Sales Model".into(),
                tables: vec![
                    TableSchema {
                        table_id: "table_fact_sales".into(),
                        table_name: "FactSales".into(),
                        columns: vec![
                            ColumnSchema {
                                column_id: "col_revenue".into(),
                                column_name: "Revenue".into(),
                                data_type: "Decimal".into(),
                                is_hidden: false,
                            },
                            ColumnSchema {
                                column_id: "col_date_key".into(),
                                column_name: "DateKey".into(),
                                data_type: "Int64".into(),
                                is_hidden: false,
                            },
                        ],
                        is_hidden: false,
                    },
                    TableSchema {
                        table_id: "table_dim_date".into(),
                        table_name: "DimDate".into(),
                        columns: vec![ColumnSchema {
                            column_id: "col_calendar_year".into(),
                            column_name: "CalendarYear".into(),
                            data_type: "Int64".into(),
                            is_hidden: false,
                        }],
                        is_hidden: false,
                    },
                ],
                measures: vec![
                    MeasureSchema {
                        measure_id: "measure_total_revenue".into(),
                        measure_name: "Total Revenue".into(),
                        table_name: "FactSales".into(),
                        expression: "SUM(FactSales[Revenue])".into(),
                        format_string: Some("#,##0.00".into()),
                        description: Some("Sum of all revenue".into()),
                        aliases: vec!["revenue".into(), "total rev".into()],
                        is_hidden: false,
                    },
                    MeasureSchema {
                        measure_id: "measure_qoq_revenue".into(),
                        measure_name: "QoQ Revenue".into(),
                        table_name: "FactSales".into(),
                        expression: "CALCULATE([Total Revenue], DATEADD(DimDate[Date], -1, QUARTER))".into(),
                        format_string: Some("#,##0.00".into()),
                        description: Some("Quarter-over-quarter revenue comparison".into()),
                        aliases: vec!["QoQ rev".into(), "quarter over quarter revenue".into()],
                        is_hidden: false,
                    },
                ],
                relationships: vec![RelationshipSchema {
                    relationship_id: "rel_sales_date".into(),
                    from_table: "FactSales".into(),
                    from_column: "DateKey".into(),
                    to_table: "DimDate".into(),
                    to_column: "DateKey".into(),
                    cardinality: Cardinality::ManyToOne,
                    cross_filter_direction: CrossFilterDirection::Single,
                    is_active: true,
                }],
            }],
        }
    }

    #[test]
    fn tier1_round_trip_json() {
        let tier1 = sample_tier1();
        let json = serde_json::to_string_pretty(&tier1).unwrap();
        let restored: Tier1SchemaPackage = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.models.len(), 1);
        assert_eq!(restored.models[0].tables.len(), 2);
        assert_eq!(restored.models[0].measures.len(), 2);
        assert_eq!(restored.models[0].relationships.len(), 1);
    }

    #[test]
    fn tier1_round_trip_yaml() {
        let tier1 = sample_tier1();
        let yaml = serde_yaml::to_string(&tier1).unwrap();
        let restored: Tier1SchemaPackage = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(restored.models[0].model_name, "Sales Model");
    }

    #[test]
    fn measure_aliases_preserved() {
        let tier1 = sample_tier1();
        let measure = &tier1.models[0].measures[0];
        assert_eq!(measure.aliases.len(), 2);
        assert!(measure.aliases.contains(&"revenue".to_string()));
        assert!(measure.aliases.contains(&"total rev".to_string()));
    }

    #[test]
    fn relationship_cardinality_round_trip() {
        let cardinalities = vec![
            Cardinality::OneToOne,
            Cardinality::OneToMany,
            Cardinality::ManyToOne,
            Cardinality::ManyToMany,
        ];
        for c in cardinalities {
            let json = serde_json::to_string(&c).unwrap();
            let restored: Cardinality = serde_json::from_str(&json).unwrap();
            assert_eq!(restored, c);
        }
    }

    #[test]
    fn hidden_artifacts_tracked() {
        let tier1 = sample_tier1();
        let table = &tier1.models[0].tables[0];
        assert!(!table.is_hidden);
        let col = &table.columns[0];
        assert!(!col.is_hidden);
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cd spool && cargo test -p spool-knowledge -- tier1`
Expected: FAIL

**Step 3: Write the implementation**

```rust
// spool/spool-knowledge/src/tier1.rs
use serde::{Deserialize, Serialize};

/// Tier 1 schema package generated from Fabric semantic model definitions.
/// Contains compact structural knowledge — not interpretive.
/// Spec Section 7.4.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tier1SchemaPackage {
    pub schema_version: String,
    pub models: Vec<SemanticModelSchema>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticModelSchema {
    pub model_id: String,
    pub model_name: String,
    pub tables: Vec<TableSchema>,
    pub measures: Vec<MeasureSchema>,
    pub relationships: Vec<RelationshipSchema>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableSchema {
    pub table_id: String,
    pub table_name: String,
    pub columns: Vec<ColumnSchema>,
    pub is_hidden: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnSchema {
    pub column_id: String,
    pub column_name: String,
    pub data_type: String,
    pub is_hidden: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeasureSchema {
    pub measure_id: String,
    pub measure_name: String,
    pub table_name: String,
    pub expression: String,
    pub format_string: Option<String>,
    pub description: Option<String>,
    pub aliases: Vec<String>,
    pub is_hidden: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipSchema {
    pub relationship_id: String,
    pub from_table: String,
    pub from_column: String,
    pub to_table: String,
    pub to_column: String,
    pub cardinality: Cardinality,
    pub cross_filter_direction: CrossFilterDirection,
    pub is_active: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Cardinality {
    OneToOne,
    OneToMany,
    ManyToOne,
    ManyToMany,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CrossFilterDirection {
    Single,
    Both,
    None,
}

// tests at bottom (from Step 1)
```

**Step 4: Run test to verify it passes**

Run: `cd spool && cargo test -p spool-knowledge -- tier1`
Expected: 5 tests PASS

**Step 5: Commit**

```bash
git add spool/spool-knowledge/src/tier1.rs
git commit -m "feat(spool-knowledge): Tier 1 schema package types — models, tables, columns, measures, relationships with cardinality"
```

---

## Task 4: Tier 2 Curated Knowledge Package Types

**Files:**

- Create: `spool/spool-knowledge/src/tier2.rs`

**Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn context_artifact_round_trip_yaml() {
        let ctx = ContextArtifact {
            id: "ctx_data_model".into(),
            name: "data_model".into(),
            aliases: vec!["data sources".into(), "LOB mapping".into()],
            description: "How AI and non-AI data are structured in Tech Router".into(),
            body: "The Tech Router data model is built on both human and AI data.".into(),
            key_relationships: vec![
                "AI views contain data for all businesses — must be filtered".into(),
            ],
            usage_notes: vec!["When querying AI data, use the configured views".into()],
        };

        let yaml = serde_yaml::to_string(&ctx).unwrap();
        let restored: ContextArtifact = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(restored.id, "ctx_data_model");
        assert_eq!(restored.aliases.len(), 2);
    }

    #[test]
    fn metric_artifact_round_trip_yaml() {
        let metric = MetricArtifact {
            id: "metric_total_revenue".into(),
            name: "total_revenue".into(),
            aliases: vec!["revenue".into(), "total rev".into()],
            description: "Sum of all revenue across the reporting period".into(),
            measure_refs: vec!["FactSales.Total Revenue".into()],
            formula: Some("SUM(FactSales[Revenue])".into()),
            disambiguation: vec![DisambiguationEntry {
                versus: "net_revenue".into(),
                difference: "total_revenue includes returns; net_revenue excludes them".into(),
            }],
        };

        let yaml = serde_yaml::to_string(&metric).unwrap();
        let restored: MetricArtifact = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(restored.measure_refs.len(), 1);
        assert_eq!(restored.disambiguation.len(), 1);
        assert_eq!(restored.disambiguation[0].versus, "net_revenue");
    }

    #[test]
    fn rule_artifact_round_trip_yaml() {
        let rule = RuleArtifact {
            id: "rule_data_source_selection".into(),
            name: "data_source_selection".into(),
            aliases: vec!["source selection".into()],
            description: "Which data source to use for AI vs non-AI queries".into(),
            decision_logic: "IF query is about AI interactions: use AI views\nIF query is about non-AI: use pre-filtered views\nIF unclear: ask user".into(),
            why_it_matters: "Wrong source leads to incorrect totals due to business filtering".into(),
            examples: vec!["AI deflection uses AIAgentInteractionView".into()],
        };

        let yaml = serde_yaml::to_string(&rule).unwrap();
        let restored: RuleArtifact = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(restored.id, "rule_data_source_selection");
    }

    #[test]
    fn pattern_artifact_round_trip_yaml() {
        let pattern = PatternArtifact {
            id: "pattern_subtraction".into(),
            name: "subtraction_pattern".into(),
            aliases: vec!["subtraction method".into()],
            description: "Derive human metrics by subtracting AI from total".into(),
            when_to_apply: "When no dedicated human fact table exists".into(),
            approach: vec![
                "Query total from blended view".into(),
                "Query AI from AI view".into(),
                "Subtract AI from total to get human".into(),
            ],
            anti_patterns: vec![
                "Do not query human metrics directly from AI views".into(),
            ],
        };

        let yaml = serde_yaml::to_string(&pattern).unwrap();
        let restored: PatternArtifact = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(restored.approach.len(), 3);
        assert_eq!(restored.anti_patterns.len(), 1);
    }

    #[test]
    fn tier2_package_round_trip_json() {
        let package = Tier2Package {
            contexts: vec![ContextArtifact {
                id: "ctx_test".into(),
                name: "test_context".into(),
                aliases: vec![],
                description: "test".into(),
                body: "body".into(),
                key_relationships: vec![],
                usage_notes: vec![],
            }],
            metrics: vec![],
            rules: vec![],
            patterns: vec![],
            recipe_ids: vec!["recipe_test".into()],
        };

        let json = serde_json::to_string(&package).unwrap();
        let restored: Tier2Package = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.contexts.len(), 1);
        assert_eq!(restored.recipe_ids.len(), 1);
    }

    #[test]
    fn all_artifact_ids_from_tier2() {
        let package = Tier2Package {
            contexts: vec![ContextArtifact {
                id: "ctx_a".into(),
                name: "a".into(),
                aliases: vec![],
                description: "".into(),
                body: "".into(),
                key_relationships: vec![],
                usage_notes: vec![],
            }],
            metrics: vec![MetricArtifact {
                id: "metric_b".into(),
                name: "b".into(),
                aliases: vec![],
                description: "".into(),
                measure_refs: vec![],
                formula: None,
                disambiguation: vec![],
            }],
            rules: vec![RuleArtifact {
                id: "rule_c".into(),
                name: "c".into(),
                aliases: vec![],
                description: "".into(),
                decision_logic: "".into(),
                why_it_matters: "".into(),
                examples: vec![],
            }],
            patterns: vec![PatternArtifact {
                id: "pattern_d".into(),
                name: "d".into(),
                aliases: vec![],
                description: "".into(),
                when_to_apply: "".into(),
                approach: vec![],
                anti_patterns: vec![],
            }],
            recipe_ids: vec!["recipe_e".into()],
        };

        let ids = package.all_artifact_ids();
        assert_eq!(ids.len(), 4);
        assert!(ids.contains(&"ctx_a".to_string()));
        assert!(ids.contains(&"metric_b".to_string()));
        assert!(ids.contains(&"rule_c".to_string()));
        assert!(ids.contains(&"pattern_d".to_string()));
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cd spool && cargo test -p spool-knowledge -- tier2`
Expected: FAIL

**Step 3: Write the implementation**

```rust
// spool/spool-knowledge/src/tier2.rs
use serde::{Deserialize, Serialize};

/// Tier 2 curated knowledge package — human-authored business knowledge.
/// Spec Section 7.5.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tier2Package {
    pub contexts: Vec<ContextArtifact>,
    pub metrics: Vec<MetricArtifact>,
    pub rules: Vec<RuleArtifact>,
    pub patterns: Vec<PatternArtifact>,
    /// Recipe IDs declared in this package. Actual recipe content
    /// is stored in the recipe module and cross-referenced here.
    pub recipe_ids: Vec<String>,
}

impl Tier2Package {
    /// Returns all bundle-local artifact IDs from this package.
    /// Does not include recipe_ids — those are tracked separately.
    pub fn all_artifact_ids(&self) -> Vec<String> {
        let mut ids = Vec::new();
        for c in &self.contexts {
            ids.push(c.id.clone());
        }
        for m in &self.metrics {
            ids.push(m.id.clone());
        }
        for r in &self.rules {
            ids.push(r.id.clone());
        }
        for p in &self.patterns {
            ids.push(p.id.clone());
        }
        ids
    }

    /// Returns all aliases declared across all artifacts.
    pub fn all_aliases(&self) -> Vec<(String, String)> {
        let mut aliases = Vec::new();
        for c in &self.contexts {
            for a in &c.aliases {
                aliases.push((a.clone(), c.id.clone()));
            }
        }
        for m in &self.metrics {
            for a in &m.aliases {
                aliases.push((a.clone(), m.id.clone()));
            }
        }
        for r in &self.rules {
            for a in &r.aliases {
                aliases.push((a.clone(), r.id.clone()));
            }
        }
        for p in &self.patterns {
            for a in &p.aliases {
                aliases.push((a.clone(), p.id.clone()));
            }
        }
        aliases
    }
}

/// Defines a business concept, relationships, and usage notes.
/// Template: knowledge/templates/context.md
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextArtifact {
    pub id: String,
    pub name: String,
    pub aliases: Vec<String>,
    pub description: String,
    pub body: String,
    pub key_relationships: Vec<String>,
    pub usage_notes: Vec<String>,
}

/// Defines business meaning, aliases, and linked semantic-model measures.
/// Template: knowledge/templates/metric.yml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricArtifact {
    pub id: String,
    pub name: String,
    pub aliases: Vec<String>,
    pub description: String,
    pub measure_refs: Vec<String>,
    pub formula: Option<String>,
    pub disambiguation: Vec<DisambiguationEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisambiguationEntry {
    pub versus: String,
    pub difference: String,
}

/// Defines decision logic or interpretation logic.
/// Template: knowledge/templates/rule.md
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleArtifact {
    pub id: String,
    pub name: String,
    pub aliases: Vec<String>,
    pub description: String,
    pub decision_logic: String,
    pub why_it_matters: String,
    pub examples: Vec<String>,
}

/// Defines a reusable analytical approach and anti-patterns.
/// Template: knowledge/templates/pattern.md
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternArtifact {
    pub id: String,
    pub name: String,
    pub aliases: Vec<String>,
    pub description: String,
    pub when_to_apply: String,
    pub approach: Vec<String>,
    pub anti_patterns: Vec<String>,
}

// tests at bottom (from Step 1)
```

**Step 4: Run test to verify it passes**

Run: `cd spool && cargo test -p spool-knowledge -- tier2`
Expected: 6 tests PASS

**Step 5: Commit**

```bash
git add spool/spool-knowledge/src/tier2.rs
git commit -m "feat(spool-knowledge): Tier 2 curated knowledge types — contexts, metrics, rules, patterns with aliases and disambiguation"
```

---

## Task 5: Recipe Schema

**Files:**

- Create: `spool/spool-knowledge/src/recipe.rs`

**Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn sample_recipe() -> Recipe {
        Recipe {
            id: "recipe_report_number_mismatch".into(),
            name: "report_number_mismatch".into(),
            intent: "Diagnose why a report visual does not match expected totals".into(),
            problem_class: "number_mismatch".into(),
            applies_when: vec![
                "report number is disputed".into(),
                "semantic model and warehouse validation may both be relevant".into(),
            ],
            required_inputs: vec![
                RecipeInput {
                    name: "report_reference".into(),
                    description: "Report URL, name, or ID".into(),
                    input_type: RecipeInputType::ArtifactReference,
                    required: true,
                },
                RecipeInput {
                    name: "expected_value".into(),
                    description: "The number the user expects to see".into(),
                    input_type: RecipeInputType::UserProvided,
                    required: false,
                },
            ],
            recommended_artifact_targets: vec![
                "report".into(),
                "semantic_model".into(),
                "measure".into(),
            ],
            expected_evidence_classes: vec![
                "report_metadata".into(),
                "measure_definition".into(),
                "dax_query_result".into(),
                "warehouse_query_result".into(),
            ],
            validation_expectations: vec![
                "compare report output against at least one direct query-based validation".into(),
            ],
            steps: vec![
                RecipeStep {
                    order: 1,
                    description: "Resolve report, page, and visual".into(),
                    evidence_class: Some("report_metadata".into()),
                    artifact_target: Some("report".into()),
                },
                RecipeStep {
                    order: 2,
                    description: "Identify backing semantic model object".into(),
                    evidence_class: Some("semantic_model_metadata".into()),
                    artifact_target: Some("semantic_model".into()),
                },
                RecipeStep {
                    order: 3,
                    description: "Inspect relevant measure definitions and filters".into(),
                    evidence_class: Some("measure_definition".into()),
                    artifact_target: Some("measure".into()),
                },
                RecipeStep {
                    order: 4,
                    description: "Run diagnostic DAX query".into(),
                    evidence_class: Some("dax_query_result".into()),
                    artifact_target: None,
                },
                RecipeStep {
                    order: 5,
                    description: "Run warehouse validation query when appropriate".into(),
                    evidence_class: Some("warehouse_query_result".into()),
                    artifact_target: None,
                },
                RecipeStep {
                    order: 6,
                    description: "Classify likely mismatch source".into(),
                    evidence_class: None,
                    artifact_target: None,
                },
            ],
            anti_patterns: vec![
                "Assume warehouse mismatch before checking measure logic".into(),
                "Rely on report screenshot alone".into(),
            ],
            worked_examples: vec![WorkedExample {
                title: "Q1 revenue mismatch between report and warehouse".into(),
                scenario: "User reports Executive Revenue Report shows $12.4M for Q1 but warehouse shows $13.1M".into(),
                walkthrough: vec![
                    "Resolved report to workspace Executive BI, page Revenue Overview".into(),
                    "Identified backing model: Sales Model".into(),
                    "Inspected Total Revenue measure: SUM(FactSales[Revenue])".into(),
                    "DAX query returned $12.4M — matches report".into(),
                    "Warehouse query returned $13.1M — mismatch confirmed".into(),
                    "Root cause: measure has an implicit date filter excluding late-arriving transactions".into(),
                ],
                outcome: "Confirmed mismatch due to implicit date filter on measure".into(),
            }],
        }
    }

    #[test]
    fn recipe_round_trip_yaml() {
        let recipe = sample_recipe();
        let yaml = serde_yaml::to_string(&recipe).unwrap();
        let restored: Recipe = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(restored.id, "recipe_report_number_mismatch");
        assert_eq!(restored.steps.len(), 6);
        assert_eq!(restored.anti_patterns.len(), 2);
        assert_eq!(restored.worked_examples.len(), 1);
    }

    #[test]
    fn recipe_round_trip_json() {
        let recipe = sample_recipe();
        let json = serde_json::to_string_pretty(&recipe).unwrap();
        let restored: Recipe = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.name, "report_number_mismatch");
        assert_eq!(restored.required_inputs.len(), 2);
        assert!(restored.required_inputs[0].required);
        assert!(!restored.required_inputs[1].required);
    }

    #[test]
    fn recipe_step_ordering() {
        let recipe = sample_recipe();
        for (i, step) in recipe.steps.iter().enumerate() {
            assert_eq!(step.order, (i + 1) as u32);
        }
    }

    #[test]
    fn recipe_selection_outcome_round_trip() {
        let outcomes = vec![
            RecipeSelectionOutcome::AutoSelect {
                recipe_id: "recipe_test".into(),
                reason: "strong fit".into(),
            },
            RecipeSelectionOutcome::Suggest {
                recipe_ids: vec!["recipe_a".into(), "recipe_b".into()],
                reason: "multiple plausible recipes".into(),
            },
            RecipeSelectionOutcome::DoNotUse {
                reason: "no recipe fits".into(),
            },
            RecipeSelectionOutcome::UserRequestedOverride {
                recipe_id: "recipe_custom".into(),
                accepted: true,
                refusal_reason: None,
            },
        ];

        for outcome in outcomes {
            let json = serde_json::to_string(&outcome).unwrap();
            let restored: RecipeSelectionOutcome = serde_json::from_str(&json).unwrap();
            let json2 = serde_json::to_string(&restored).unwrap();
            assert_eq!(json, json2);
        }
    }

    #[test]
    fn recipe_deviation_round_trip() {
        let deviation = RecipeDeviation {
            recipe_id: "recipe_report_number_mismatch".into(),
            step_order: 5,
            deviation_type: DeviationType::Skipped,
            reason: "Warehouse access not available for this workspace".into(),
            evidence_justification: Some("ev_access_denied".into()),
            confidence_impact: Some("Reduced from high to medium — no warehouse validation".into()),
        };

        let json = serde_json::to_string(&deviation).unwrap();
        let restored: RecipeDeviation = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.step_order, 5);
        assert!(matches!(restored.deviation_type, DeviationType::Skipped));
    }

    #[test]
    fn all_deviation_types_serialize() {
        let types = vec![
            DeviationType::Skipped,
            DeviationType::Reordered,
            DeviationType::Modified,
            DeviationType::Added,
        ];
        for t in types {
            let json = serde_json::to_string(&t).unwrap();
            let restored: DeviationType = serde_json::from_str(&json).unwrap();
            assert_eq!(json, serde_json::to_string(&restored).unwrap());
        }
    }

    #[test]
    fn recipe_input_types_serialize() {
        let types = vec![
            RecipeInputType::ArtifactReference,
            RecipeInputType::UserProvided,
            RecipeInputType::DerivedFromContext,
        ];
        for t in types {
            let json = serde_json::to_string(&t).unwrap();
            let restored: RecipeInputType = serde_json::from_str(&json).unwrap();
            assert_eq!(json, serde_json::to_string(&restored).unwrap());
        }
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cd spool && cargo test -p spool-knowledge -- recipe`
Expected: FAIL

**Step 3: Write the implementation**

```rust
// spool/spool-knowledge/src/recipe.rs
use serde::{Deserialize, Serialize};

/// A structured investigation playbook tied to evidence classes
/// and validation expectations. Spec Section 8.2.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recipe {
    pub id: String,
    pub name: String,
    pub intent: String,
    pub problem_class: String,
    pub applies_when: Vec<String>,
    pub required_inputs: Vec<RecipeInput>,
    pub recommended_artifact_targets: Vec<String>,
    pub expected_evidence_classes: Vec<String>,
    pub validation_expectations: Vec<String>,
    pub steps: Vec<RecipeStep>,
    pub anti_patterns: Vec<String>,
    pub worked_examples: Vec<WorkedExample>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeInput {
    pub name: String,
    pub description: String,
    pub input_type: RecipeInputType,
    pub required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecipeInputType {
    ArtifactReference,
    UserProvided,
    DerivedFromContext,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeStep {
    pub order: u32,
    pub description: String,
    pub evidence_class: Option<String>,
    pub artifact_target: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkedExample {
    pub title: String,
    pub scenario: String,
    pub walkthrough: Vec<String>,
    pub outcome: String,
}

/// Planner selection outcomes for recipe fit evaluation.
/// Spec Section 8.5.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum RecipeSelectionOutcome {
    AutoSelect {
        recipe_id: String,
        reason: String,
    },
    Suggest {
        recipe_ids: Vec<String>,
        reason: String,
    },
    DoNotUse {
        reason: String,
    },
    UserRequestedOverride {
        recipe_id: String,
        accepted: bool,
        refusal_reason: Option<String>,
    },
}

/// Records a deviation from a selected recipe during execution.
/// Spec Section 8.6.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeDeviation {
    pub recipe_id: String,
    pub step_order: u32,
    pub deviation_type: DeviationType,
    pub reason: String,
    pub evidence_justification: Option<String>,
    pub confidence_impact: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeviationType {
    Skipped,
    Reordered,
    Modified,
    Added,
}

// tests at bottom (from Step 1)
```

**Step 4: Run test to verify it passes**

Run: `cd spool && cargo test -p spool-knowledge -- recipe`
Expected: 7 tests PASS

**Step 5: Commit**

```bash
git add spool/spool-knowledge/src/recipe.rs
git commit -m "feat(spool-knowledge): recipe schema with steps, anti-patterns, worked examples, selection outcomes, and deviation recording"
```

---

## Task 6: Bundle Naming and Reference Rules — Validation Types

**Files:**

- Create: `spool/spool-knowledge/src/validation.rs`

**Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::bundle::BundleManifest;
    use crate::recipe::Recipe;
    use crate::tier1::{
        Cardinality, ColumnSchema, CrossFilterDirection, MeasureSchema, RelationshipSchema,
        SemanticModelSchema, TableSchema, Tier1SchemaPackage,
    };
    use crate::tier2::*;
    use chrono::Utc;

    fn sample_manifest() -> BundleManifest {
        BundleManifest {
            bundle_id: "bundle_test".into(),
            lob_id: "test".into(),
            version: "1.0.0".into(),
            display_name: "Test Bundle".into(),
            default_workspace_scope: None,
            tier1_schema_version: "1.0.0".into(),
            tier2_bundle_version: "1.0.0".into(),
            build_timestamp: Utc::now(),
            source_summary: "test".into(),
            declared_artifact_classes: vec!["metric".into(), "context".into()],
            declared_recipe_ids: vec!["recipe_test".into()],
        }
    }

    fn sample_tier1() -> Tier1SchemaPackage {
        Tier1SchemaPackage {
            schema_version: "1.0.0".into(),
            models: vec![SemanticModelSchema {
                model_id: "model_sales".into(),
                model_name: "Sales Model".into(),
                tables: vec![TableSchema {
                    table_id: "table_fact".into(),
                    table_name: "FactSales".into(),
                    columns: vec![ColumnSchema {
                        column_id: "col_rev".into(),
                        column_name: "Revenue".into(),
                        data_type: "Decimal".into(),
                        is_hidden: false,
                    }],
                    is_hidden: false,
                }],
                measures: vec![MeasureSchema {
                    measure_id: "measure_total_rev".into(),
                    measure_name: "Total Revenue".into(),
                    table_name: "FactSales".into(),
                    expression: "SUM(FactSales[Revenue])".into(),
                    format_string: None,
                    description: None,
                    aliases: vec!["revenue".into()],
                    is_hidden: false,
                }],
                relationships: vec![],
            }],
        }
    }

    fn sample_tier2() -> Tier2Package {
        Tier2Package {
            contexts: vec![ContextArtifact {
                id: "ctx_data".into(),
                name: "data_model".into(),
                aliases: vec!["data sources".into()],
                description: "test".into(),
                body: "body".into(),
                key_relationships: vec![],
                usage_notes: vec![],
            }],
            metrics: vec![MetricArtifact {
                id: "metric_total_rev".into(),
                name: "total_revenue".into(),
                aliases: vec!["revenue".into()],
                description: "total revenue".into(),
                measure_refs: vec!["FactSales.Total Revenue".into()],
                formula: None,
                disambiguation: vec![],
            }],
            rules: vec![],
            patterns: vec![],
            recipe_ids: vec!["recipe_test".into()],
        }
    }

    fn sample_recipe() -> Recipe {
        Recipe {
            id: "recipe_test".into(),
            name: "test_recipe".into(),
            intent: "test".into(),
            problem_class: "test".into(),
            applies_when: vec![],
            required_inputs: vec![],
            recommended_artifact_targets: vec![],
            expected_evidence_classes: vec!["report_metadata".into()],
            validation_expectations: vec![],
            steps: vec![],
            anti_patterns: vec![],
            worked_examples: vec![],
        }
    }

    #[test]
    fn valid_bundle_passes_validation() {
        let manifest = sample_manifest();
        let tier1 = sample_tier1();
        let tier2 = sample_tier2();
        let recipes = vec![sample_recipe()];

        let result = validate_bundle(&manifest, &tier1, &tier2, &recipes);
        assert!(result.is_valid());
        assert!(result.errors.is_empty());
    }

    #[test]
    fn duplicate_artifact_ids_detected() {
        let manifest = sample_manifest();
        let tier1 = sample_tier1();
        let mut tier2 = sample_tier2();
        // Add a context with the same ID as the metric
        tier2.contexts.push(ContextArtifact {
            id: "metric_total_rev".into(), // duplicate of the metric ID
            name: "duplicate".into(),
            aliases: vec![],
            description: "dup".into(),
            body: "body".into(),
            key_relationships: vec![],
            usage_notes: vec![],
        });
        let recipes = vec![sample_recipe()];

        let result = validate_bundle(&manifest, &tier1, &tier2, &recipes);
        assert!(!result.is_valid());
        assert!(result.errors.iter().any(|e| matches!(e, ValidationError::DuplicateArtifactId { .. })));
    }

    #[test]
    fn missing_measure_ref_detected() {
        let manifest = sample_manifest();
        let tier1 = sample_tier1();
        let mut tier2 = sample_tier2();
        // Add a metric referencing a measure that does not exist in Tier 1
        tier2.metrics.push(MetricArtifact {
            id: "metric_ghost".into(),
            name: "ghost_metric".into(),
            aliases: vec![],
            description: "references nonexistent measure".into(),
            measure_refs: vec!["Nonexistent.Measure".into()],
            formula: None,
            disambiguation: vec![],
        });
        let recipes = vec![sample_recipe()];

        let result = validate_bundle(&manifest, &tier1, &tier2, &recipes);
        assert!(result.warnings.iter().any(|w| matches!(w, ValidationWarning::UnresolvedMeasureRef { .. })));
    }

    #[test]
    fn alias_collision_detected() {
        let manifest = sample_manifest();
        let tier1 = sample_tier1();
        let mut tier2 = sample_tier2();
        // Add a context with an alias that collides with the metric alias
        tier2.contexts[0].aliases.push("revenue".into()); // same alias as metric
        let recipes = vec![sample_recipe()];

        let result = validate_bundle(&manifest, &tier1, &tier2, &recipes);
        assert!(result.warnings.iter().any(|w| matches!(w, ValidationWarning::AliasCollision { .. })));
    }

    #[test]
    fn missing_recipe_detected() {
        let manifest = sample_manifest();
        let tier1 = sample_tier1();
        let tier2 = sample_tier2();
        // Pass empty recipes — manifest declares recipe_test but it is missing
        let recipes: Vec<Recipe> = vec![];

        let result = validate_bundle(&manifest, &tier1, &tier2, &recipes);
        assert!(!result.is_valid());
        assert!(result.errors.iter().any(|e| matches!(e, ValidationError::MissingRecipe { .. })));
    }

    #[test]
    fn recipe_references_missing_evidence_class_detected() {
        let manifest = sample_manifest();
        let tier1 = sample_tier1();
        let tier2 = sample_tier2();
        let mut recipe = sample_recipe();
        recipe.expected_evidence_classes = vec!["nonexistent_evidence_class".into()];
        let recipes = vec![recipe];

        let result = validate_bundle(&manifest, &tier1, &tier2, &recipes);
        assert!(result.warnings.iter().any(|w| matches!(w, ValidationWarning::UnknownEvidenceClass { .. })));
    }

    #[test]
    fn validation_result_serializes() {
        let result = BundleValidationResult {
            bundle_id: "bundle_test".into(),
            errors: vec![ValidationError::DuplicateArtifactId {
                id: "dup_id".into(),
                locations: vec!["context".into(), "metric".into()],
            }],
            warnings: vec![ValidationWarning::AliasCollision {
                alias: "revenue".into(),
                owners: vec!["ctx_data".into(), "metric_total_rev".into()],
            }],
        };

        let json = serde_json::to_string_pretty(&result).unwrap();
        let restored: BundleValidationResult = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.errors.len(), 1);
        assert_eq!(restored.warnings.len(), 1);
        assert!(!restored.is_valid());
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cd spool && cargo test -p spool-knowledge -- validation`
Expected: FAIL

**Step 3: Write the implementation**

```rust
// spool/spool-knowledge/src/validation.rs
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use crate::bundle::BundleManifest;
use crate::recipe::Recipe;
use crate::tier1::Tier1SchemaPackage;
use crate::tier2::Tier2Package;

/// Known valid evidence classes in v1.
const KNOWN_EVIDENCE_CLASSES: &[&str] = &[
    "report_metadata",
    "visual_metadata",
    "semantic_model_metadata",
    "measure_definition",
    "dax_query_result",
    "warehouse_query_result",
    "cross_source_comparison",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleValidationResult {
    pub bundle_id: String,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
}

impl BundleValidationResult {
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "error_type", rename_all = "snake_case")]
pub enum ValidationError {
    DuplicateArtifactId {
        id: String,
        locations: Vec<String>,
    },
    MissingRecipe {
        recipe_id: String,
    },
    MalformedBundleStructure {
        detail: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "warning_type", rename_all = "snake_case")]
pub enum ValidationWarning {
    UnresolvedMeasureRef {
        metric_id: String,
        measure_ref: String,
    },
    AliasCollision {
        alias: String,
        owners: Vec<String>,
    },
    UnknownEvidenceClass {
        recipe_id: String,
        evidence_class: String,
    },
    IncompleteSchemaArtifact {
        detail: String,
    },
}

/// Validate a bundle across both tiers and all recipes.
/// Spec Section 7.10.
pub fn validate_bundle(
    manifest: &BundleManifest,
    tier1: &Tier1SchemaPackage,
    tier2: &Tier2Package,
    recipes: &[Recipe],
) -> BundleValidationResult {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    // 1. Check for duplicate artifact IDs within Tier 2
    check_duplicate_ids(tier2, &mut errors);

    // 2. Check measure refs resolve against Tier 1
    check_measure_refs(tier1, tier2, &mut warnings);

    // 3. Check alias collisions
    check_alias_collisions(tier2, &mut warnings);

    // 4. Check declared recipes exist
    check_declared_recipes(manifest, recipes, &mut errors);

    // 5. Check recipe evidence class references
    check_recipe_evidence_classes(recipes, &mut warnings);

    BundleValidationResult {
        bundle_id: manifest.bundle_id.clone(),
        errors,
        warnings,
    }
}

fn check_duplicate_ids(tier2: &Tier2Package, errors: &mut Vec<ValidationError>) {
    let mut seen: HashMap<String, Vec<String>> = HashMap::new();

    for c in &tier2.contexts {
        seen.entry(c.id.clone())
            .or_default()
            .push("context".into());
    }
    for m in &tier2.metrics {
        seen.entry(m.id.clone())
            .or_default()
            .push("metric".into());
    }
    for r in &tier2.rules {
        seen.entry(r.id.clone()).or_default().push("rule".into());
    }
    for p in &tier2.patterns {
        seen.entry(p.id.clone())
            .or_default()
            .push("pattern".into());
    }

    for (id, locations) in seen {
        if locations.len() > 1 {
            errors.push(ValidationError::DuplicateArtifactId { id, locations });
        }
    }
}

fn check_measure_refs(
    tier1: &Tier1SchemaPackage,
    tier2: &Tier2Package,
    warnings: &mut Vec<ValidationWarning>,
) {
    // Build a set of all available measures as "TableName.MeasureName"
    let mut available_measures: HashSet<String> = HashSet::new();
    for model in &tier1.models {
        for measure in &model.measures {
            available_measures.insert(format!("{}.{}", measure.table_name, measure.measure_name));
        }
    }

    for metric in &tier2.metrics {
        for measure_ref in &metric.measure_refs {
            if !available_measures.contains(measure_ref) {
                warnings.push(ValidationWarning::UnresolvedMeasureRef {
                    metric_id: metric.id.clone(),
                    measure_ref: measure_ref.clone(),
                });
            }
        }
    }
}

fn check_alias_collisions(tier2: &Tier2Package, warnings: &mut Vec<ValidationWarning>) {
    let all_aliases = tier2.all_aliases();
    let mut alias_map: HashMap<String, Vec<String>> = HashMap::new();

    for (alias, owner_id) in all_aliases {
        let normalized = alias.to_lowercase();
        alias_map.entry(normalized).or_default().push(owner_id);
    }

    for (alias, owners) in alias_map {
        if owners.len() > 1 {
            warnings.push(ValidationWarning::AliasCollision { alias, owners });
        }
    }
}

fn check_declared_recipes(
    manifest: &BundleManifest,
    recipes: &[Recipe],
    errors: &mut Vec<ValidationError>,
) {
    let available_recipe_ids: HashSet<&str> =
        recipes.iter().map(|r| r.id.as_str()).collect();

    for declared_id in &manifest.declared_recipe_ids {
        if !available_recipe_ids.contains(declared_id.as_str()) {
            errors.push(ValidationError::MissingRecipe {
                recipe_id: declared_id.clone(),
            });
        }
    }
}

fn check_recipe_evidence_classes(recipes: &[Recipe], warnings: &mut Vec<ValidationWarning>) {
    let known: HashSet<&str> = KNOWN_EVIDENCE_CLASSES.iter().copied().collect();

    for recipe in recipes {
        for ec in &recipe.expected_evidence_classes {
            if !known.contains(ec.as_str()) {
                warnings.push(ValidationWarning::UnknownEvidenceClass {
                    recipe_id: recipe.id.clone(),
                    evidence_class: ec.clone(),
                });
            }
        }
    }
}

// tests at bottom (from Step 1)
```

**Step 4: Run test to verify it passes**

Run: `cd spool && cargo test -p spool-knowledge -- validation`
Expected: 7 tests PASS

**Step 5: Commit**

```bash
git add spool/spool-knowledge/src/validation.rs
git commit -m "feat(spool-knowledge): bundle validation — duplicate IDs, missing measure refs, alias collisions, missing recipes, unknown evidence classes"
```

---

## Task 7: Selected-LOB Loading Policy and Cold-Start Types

**Files:**

- Create: `spool/spool-knowledge/src/loading.rs`

**Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::bundle::{BundleLoadStatus, BundleManifest, LoadedBundle};
    use crate::tier1::{MeasureSchema, SemanticModelSchema, TableSchema, Tier1SchemaPackage};
    use crate::tier2::*;
    use chrono::Utc;

    fn sample_manifest(lob_id: &str) -> BundleManifest {
        BundleManifest {
            bundle_id: format!("bundle_{lob_id}"),
            lob_id: lob_id.into(),
            version: "1.0.0".into(),
            display_name: format!("{lob_id} Bundle"),
            default_workspace_scope: None,
            tier1_schema_version: "1.0.0".into(),
            tier2_bundle_version: "1.0.0".into(),
            build_timestamp: Utc::now(),
            source_summary: "test".into(),
            declared_artifact_classes: vec![],
            declared_recipe_ids: vec![],
        }
    }

    fn sample_tier1() -> Tier1SchemaPackage {
        Tier1SchemaPackage {
            schema_version: "1.0.0".into(),
            models: vec![SemanticModelSchema {
                model_id: "m1".into(),
                model_name: "Model".into(),
                tables: vec![],
                measures: vec![],
                relationships: vec![],
            }],
        }
    }

    fn sample_tier2() -> Tier2Package {
        Tier2Package {
            contexts: vec![],
            metrics: vec![],
            rules: vec![],
            patterns: vec![],
            recipe_ids: vec![],
        }
    }

    #[test]
    fn load_selected_lob_fully() {
        let available = vec![BundleSource {
            manifest: sample_manifest("finance"),
            tier1: Some(sample_tier1()),
            tier2: Some(sample_tier2()),
            recipes: vec![],
        }];

        let policy = LoadingPolicy {
            selected_lob: "finance".into(),
        };

        let result = policy.load(&available);
        assert!(result.is_ok());
        let loaded = result.unwrap();
        assert_eq!(loaded.manifest.lob_id, "finance");
        assert_eq!(loaded.load_status, BundleLoadStatus::FullyLoaded);
        assert!(loaded.tier1.is_some());
        assert!(loaded.tier2.is_some());
    }

    #[test]
    fn load_selected_lob_tier1_only() {
        let available = vec![BundleSource {
            manifest: sample_manifest("finance"),
            tier1: Some(sample_tier1()),
            tier2: None,
            recipes: vec![],
        }];

        let policy = LoadingPolicy {
            selected_lob: "finance".into(),
        };

        let result = policy.load(&available);
        assert!(result.is_ok());
        let loaded = result.unwrap();
        assert_eq!(loaded.load_status, BundleLoadStatus::Tier1Only);
        assert!(loaded.tier1.is_some());
        assert!(loaded.tier2.is_none());
    }

    #[test]
    fn load_selected_lob_cold_start() {
        let available = vec![BundleSource {
            manifest: sample_manifest("finance"),
            tier1: None,
            tier2: None,
            recipes: vec![],
        }];

        let policy = LoadingPolicy {
            selected_lob: "finance".into(),
        };

        let result = policy.load(&available);
        assert!(result.is_ok());
        let loaded = result.unwrap();
        assert_eq!(loaded.load_status, BundleLoadStatus::ColdStart);
        assert!(loaded.tier1.is_none());
        assert!(loaded.tier2.is_none());
    }

    #[test]
    fn load_nonexistent_lob_returns_error() {
        let available = vec![BundleSource {
            manifest: sample_manifest("finance"),
            tier1: Some(sample_tier1()),
            tier2: Some(sample_tier2()),
            recipes: vec![],
        }];

        let policy = LoadingPolicy {
            selected_lob: "marketing".into(),
        };

        let result = policy.load(&available);
        assert!(result.is_err());
    }

    #[test]
    fn unselected_lobs_not_loaded() {
        let available = vec![
            BundleSource {
                manifest: sample_manifest("finance"),
                tier1: Some(sample_tier1()),
                tier2: Some(sample_tier2()),
                recipes: vec![],
            },
            BundleSource {
                manifest: sample_manifest("marketing"),
                tier1: Some(sample_tier1()),
                tier2: Some(sample_tier2()),
                recipes: vec![],
            },
        ];

        let policy = LoadingPolicy {
            selected_lob: "finance".into(),
        };

        let result = policy.load(&available);
        assert!(result.is_ok());
        let loaded = result.unwrap();
        // Only finance should be loaded
        assert_eq!(loaded.manifest.lob_id, "finance");
    }

    #[test]
    fn loading_policy_round_trip() {
        let policy = LoadingPolicy {
            selected_lob: "finance".into(),
        };
        let json = serde_json::to_string(&policy).unwrap();
        let restored: LoadingPolicy = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.selected_lob, "finance");
    }

    #[test]
    fn cold_start_warning_message() {
        let warning = cold_start_warning("finance");
        assert!(warning.contains("finance"));
        assert!(warning.to_lowercase().contains("missing"));
    }

    #[test]
    fn tier1_only_warning_message() {
        let warning = tier1_only_warning("finance");
        assert!(warning.contains("finance"));
        assert!(warning.to_lowercase().contains("tier 2") || warning.to_lowercase().contains("curated"));
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cd spool && cargo test -p spool-knowledge -- loading`
Expected: FAIL

**Step 3: Write the implementation**

```rust
// spool/spool-knowledge/src/loading.rs
use serde::{Deserialize, Serialize};

use crate::bundle::{BundleLoadStatus, BundleManifest, LoadedBundle};
use crate::error::KnowledgeError;
use crate::recipe::Recipe;
use crate::tier1::Tier1SchemaPackage;
use crate::tier2::Tier2Package;
use crate::validation;

/// A source bundle available for loading — may have partial content.
#[derive(Debug, Clone)]
pub struct BundleSource {
    pub manifest: BundleManifest,
    pub tier1: Option<Tier1SchemaPackage>,
    pub tier2: Option<Tier2Package>,
    pub recipes: Vec<Recipe>,
}

/// Loading policy: load only the selected LOB bundle.
/// Spec Section 7.7.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadingPolicy {
    pub selected_lob: String,
}

impl LoadingPolicy {
    /// Load the selected LOB bundle from the available sources.
    /// Only the selected LOB is loaded — all others are ignored.
    pub fn load(&self, available: &[BundleSource]) -> Result<LoadedBundle, KnowledgeError> {
        let source = available
            .iter()
            .find(|s| s.manifest.lob_id == self.selected_lob)
            .ok_or_else(|| {
                KnowledgeError::Loading(format!(
                    "no bundle found for selected LOB: {}",
                    self.selected_lob
                ))
            })?;

        let mut validation_warnings = Vec::new();

        // Determine load status based on tier availability
        let load_status = match (&source.tier1, &source.tier2) {
            (Some(_), Some(_)) => {
                // Run validation when both tiers present
                let validation_result = validation::validate_bundle(
                    &source.manifest,
                    source.tier1.as_ref().unwrap(),
                    source.tier2.as_ref().unwrap(),
                    &source.recipes,
                );
                if !validation_result.is_valid() {
                    for err in &validation_result.errors {
                        validation_warnings
                            .push(format!("validation error: {}", serde_json::to_string(err).unwrap_or_default()));
                    }
                }
                for warn in &validation_result.warnings {
                    validation_warnings
                        .push(format!("validation warning: {}", serde_json::to_string(warn).unwrap_or_default()));
                }
                if validation_result.is_valid() && validation_warnings.is_empty() {
                    BundleLoadStatus::FullyLoaded
                } else if validation_result.is_valid() {
                    BundleLoadStatus::LoadedWithWarnings {
                        warning_count: validation_warnings.len(),
                    }
                } else {
                    BundleLoadStatus::LoadedWithWarnings {
                        warning_count: validation_warnings.len(),
                    }
                }
            }
            (Some(_), None) => {
                validation_warnings.push(tier1_only_warning(&self.selected_lob));
                BundleLoadStatus::Tier1Only
            }
            (None, _) => {
                validation_warnings.push(cold_start_warning(&self.selected_lob));
                BundleLoadStatus::ColdStart
            }
        };

        Ok(LoadedBundle {
            manifest: source.manifest.clone(),
            load_status,
            tier1: source.tier1.clone(),
            tier2: source.tier2.clone(),
            validation_warnings,
        })
    }
}

/// Warning message when pre-built schema knowledge is missing.
/// Spec Section 7.11.
pub fn cold_start_warning(lob_id: &str) -> String {
    format!(
        "Pre-built schema knowledge is missing for LOB '{lob_id}'. \
         Operating in cold-start mode with reduced knowledge quality. \
         Run spool-index to generate Tier 1 schema knowledge."
    )
}

/// Warning message when Tier 2 curated knowledge is missing.
/// Spec Section 7.11.
pub fn tier1_only_warning(lob_id: &str) -> String {
    format!(
        "Curated Tier 2 business knowledge is missing for LOB '{lob_id}'. \
         Operating with Tier 1 schema knowledge only. \
         Business-knowledge coverage is reduced."
    )
}

// tests at bottom (from Step 1)
```

**Step 4: Run test to verify it passes**

Run: `cd spool && cargo test -p spool-knowledge -- loading`
Expected: 8 tests PASS

**Step 5: Commit**

```bash
git add spool/spool-knowledge/src/loading.rs
git commit -m "feat(spool-knowledge): selected-LOB loading policy with cold-start and Tier-1-only fallback behavior"
```

---

## Task 8: Fixture TMDL Files and TMDL Parser

**Files:**

- Create: `spool/spool-index/tests/fixtures/sales_model/model.tmdl`
- Create: `spool/spool-index/tests/fixtures/sales_model/tables/FactSales.tmdl`
- Create: `spool/spool-index/tests/fixtures/sales_model/tables/DimDate.tmdl`
- Create: `spool/spool-index/src/tmdl.rs`

**Step 1: Create fixture TMDL files**

TMDL (Tabular Model Definition Language) is a human-readable format for Fabric semantic models. Create realistic fixture files.

```text
/// spool/spool-index/tests/fixtures/sales_model/model.tmdl
model Model
    culture: en-US
    defaultPowerBIDataSourceVersion: powerBI_V3
    sourceQueryCulture: en-US
```

```text
/// spool/spool-index/tests/fixtures/sales_model/tables/FactSales.tmdl
table FactSales

    measure 'Total Revenue' =
        SUM(FactSales[Revenue])
        formatString: #,##0.00
        description: "Sum of all revenue"

    measure 'QoQ Revenue' =
        CALCULATE([Total Revenue], DATEADD(DimDate[Date], -1, QUARTER))
        formatString: #,##0.00
        description: "Quarter-over-quarter revenue comparison"

    column Revenue
        dataType: decimal
        summarizeBy: sum
        sourceColumn: Revenue

    column DateKey
        dataType: int64
        isHidden
        summarizeBy: none
        sourceColumn: DateKey

    column ProductKey
        dataType: int64
        isHidden
        summarizeBy: none
        sourceColumn: ProductKey

    partition FactSales = m
        mode: import
        source
            let Source = Sql.Database("server", "db")
            in Source
```

```text
/// spool/spool-index/tests/fixtures/sales_model/tables/DimDate.tmdl
table DimDate

    column DateKey
        dataType: int64
        isKey
        summarizeBy: none
        sourceColumn: DateKey

    column CalendarYear
        dataType: int64
        summarizeBy: none
        sourceColumn: CalendarYear

    column CalendarQuarter
        dataType: string
        summarizeBy: none
        sourceColumn: CalendarQuarter

    column MonthName
        dataType: string
        summarizeBy: none
        sourceColumn: MonthName

    partition DimDate = m
        mode: import
        source
            let Source = Sql.Database("server", "db")
            in Source
```

**Step 2: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn fixtures_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("sales_model")
    }

    #[test]
    fn parse_model_file() {
        let content = std::fs::read_to_string(fixtures_dir().join("model.tmdl")).unwrap();
        let model_info = parse_model_header(&content).unwrap();
        assert_eq!(model_info.name, "Model");
        assert_eq!(model_info.culture, Some("en-US".into()));
    }

    #[test]
    fn parse_table_with_measures_and_columns() {
        let content = std::fs::read_to_string(
            fixtures_dir().join("tables").join("FactSales.tmdl"),
        )
        .unwrap();
        let table = parse_table(&content).unwrap();
        assert_eq!(table.name, "FactSales");
        assert_eq!(table.columns.len(), 3);
        assert_eq!(table.measures.len(), 2);

        // Check first measure
        let total_rev = &table.measures[0];
        assert_eq!(total_rev.name, "Total Revenue");
        assert!(total_rev.expression.contains("SUM(FactSales[Revenue])"));
        assert_eq!(total_rev.format_string, Some("#,##0.00".into()));
        assert_eq!(total_rev.description, Some("Sum of all revenue".into()));

        // Check columns
        let rev_col = table.columns.iter().find(|c| c.name == "Revenue").unwrap();
        assert_eq!(rev_col.data_type, "decimal");
        assert!(!rev_col.is_hidden);

        let dk_col = table.columns.iter().find(|c| c.name == "DateKey").unwrap();
        assert_eq!(dk_col.data_type, "int64");
        assert!(dk_col.is_hidden);
    }

    #[test]
    fn parse_table_without_measures() {
        let content = std::fs::read_to_string(
            fixtures_dir().join("tables").join("DimDate.tmdl"),
        )
        .unwrap();
        let table = parse_table(&content).unwrap();
        assert_eq!(table.name, "DimDate");
        assert_eq!(table.columns.len(), 4);
        assert!(table.measures.is_empty());

        let key_col = table.columns.iter().find(|c| c.name == "DateKey").unwrap();
        assert!(key_col.is_key);
    }

    #[test]
    fn parse_model_directory() {
        let parsed = parse_model_directory(&fixtures_dir()).unwrap();
        assert_eq!(parsed.model_name, "Model");
        assert_eq!(parsed.tables.len(), 2);
        assert_eq!(parsed.measures.len(), 2);

        let table_names: Vec<&str> = parsed.tables.iter().map(|t| t.name.as_str()).collect();
        assert!(table_names.contains(&"FactSales"));
        assert!(table_names.contains(&"DimDate"));
    }
}
```

**Step 2b: Run test to verify it fails**

Run: `cd spool && cargo test -p spool-index -- tmdl`
Expected: FAIL

**Step 3: Write the implementation**

```rust
// spool/spool-index/src/tmdl.rs
use crate::error::IndexError;
use std::path::Path;

/// Parsed model header information.
#[derive(Debug, Clone)]
pub struct ModelInfo {
    pub name: String,
    pub culture: Option<String>,
}

/// Parsed table from a TMDL file.
#[derive(Debug, Clone)]
pub struct ParsedTable {
    pub name: String,
    pub columns: Vec<ParsedColumn>,
    pub measures: Vec<ParsedMeasure>,
}

/// Parsed column from a TMDL table.
#[derive(Debug, Clone)]
pub struct ParsedColumn {
    pub name: String,
    pub data_type: String,
    pub is_hidden: bool,
    pub is_key: bool,
}

/// Parsed measure from a TMDL table.
#[derive(Debug, Clone)]
pub struct ParsedMeasure {
    pub name: String,
    pub expression: String,
    pub format_string: Option<String>,
    pub description: Option<String>,
}

/// Complete parsed model from a TMDL directory.
#[derive(Debug, Clone)]
pub struct ParsedModel {
    pub model_name: String,
    pub tables: Vec<ParsedTable>,
    pub measures: Vec<ParsedMeasure>,
}

/// Parse the model header from model.tmdl content.
pub fn parse_model_header(content: &str) -> Result<ModelInfo, IndexError> {
    let mut name = String::new();
    let mut culture = None;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("model ") {
            name = trimmed.strip_prefix("model ").unwrap_or("").trim().to_string();
        } else if trimmed.starts_with("culture:") {
            culture = Some(
                trimmed
                    .strip_prefix("culture:")
                    .unwrap_or("")
                    .trim()
                    .to_string(),
            );
        }
    }

    if name.is_empty() {
        return Err(IndexError::TmdlParse(
            "no model name found in model.tmdl".into(),
        ));
    }

    Ok(ModelInfo { name, culture })
}

/// Parse a table TMDL file into a ParsedTable.
pub fn parse_table(content: &str) -> Result<ParsedTable, IndexError> {
    let mut table_name = String::new();
    let mut columns = Vec::new();
    let mut measures = Vec::new();

    #[derive(Debug, PartialEq)]
    enum ParseState {
        Top,
        InMeasure,
        InColumn,
        InPartition,
    }

    let mut state = ParseState::Top;
    let mut current_measure_name = String::new();
    let mut current_measure_expression = String::new();
    let mut current_measure_format = None;
    let mut current_measure_description = None;
    let mut current_col_name = String::new();
    let mut current_col_type = String::new();
    let mut current_col_hidden = false;
    let mut current_col_key = false;

    let flush_measure =
        |name: &mut String,
         expr: &mut String,
         fmt: &mut Option<String>,
         desc: &mut Option<String>,
         measures: &mut Vec<ParsedMeasure>| {
            if !name.is_empty() {
                measures.push(ParsedMeasure {
                    name: std::mem::take(name),
                    expression: std::mem::take(expr).trim().to_string(),
                    format_string: fmt.take(),
                    description: desc.take(),
                });
            }
        };

    let flush_column =
        |name: &mut String,
         dtype: &mut String,
         hidden: &mut bool,
         key: &mut bool,
         columns: &mut Vec<ParsedColumn>| {
            if !name.is_empty() {
                columns.push(ParsedColumn {
                    name: std::mem::take(name),
                    data_type: std::mem::take(dtype),
                    is_hidden: *hidden,
                    is_key: *key,
                });
                *hidden = false;
                *key = false;
            }
        };

    for line in content.lines() {
        let trimmed = line.trim();

        // Table name
        if trimmed.starts_with("table ") && table_name.is_empty() {
            table_name = trimmed
                .strip_prefix("table ")
                .unwrap_or("")
                .trim()
                .to_string();
            continue;
        }

        // Detect new top-level block
        if trimmed.starts_with("measure ") && trimmed.contains('=') {
            // Flush any pending measure or column
            flush_measure(
                &mut current_measure_name,
                &mut current_measure_expression,
                &mut current_measure_format,
                &mut current_measure_description,
                &mut measures,
            );
            flush_column(
                &mut current_col_name,
                &mut current_col_type,
                &mut current_col_hidden,
                &mut current_col_key,
                &mut columns,
            );

            // Parse measure header: measure 'Name' = or measure 'Name' =\n expression
            let after_measure = trimmed.strip_prefix("measure ").unwrap_or("");
            let (name_part, expr_part) = if let Some(eq_idx) = after_measure.find('=') {
                let name_raw = after_measure[..eq_idx].trim();
                let expr_raw = after_measure[eq_idx + 1..].trim();
                (name_raw, expr_raw)
            } else {
                (after_measure.trim(), "")
            };

            // Strip quotes from name
            current_measure_name = name_part
                .trim_matches('\'')
                .trim_matches('"')
                .to_string();
            current_measure_expression = expr_raw_to_string(expr_part);
            state = ParseState::InMeasure;
            continue;
        }

        if trimmed.starts_with("column ") && state != ParseState::InPartition {
            flush_measure(
                &mut current_measure_name,
                &mut current_measure_expression,
                &mut current_measure_format,
                &mut current_measure_description,
                &mut measures,
            );
            flush_column(
                &mut current_col_name,
                &mut current_col_type,
                &mut current_col_hidden,
                &mut current_col_key,
                &mut columns,
            );

            current_col_name = trimmed
                .strip_prefix("column ")
                .unwrap_or("")
                .trim()
                .trim_matches('\'')
                .trim_matches('"')
                .to_string();
            state = ParseState::InColumn;
            continue;
        }

        if trimmed.starts_with("partition ") {
            flush_measure(
                &mut current_measure_name,
                &mut current_measure_expression,
                &mut current_measure_format,
                &mut current_measure_description,
                &mut measures,
            );
            flush_column(
                &mut current_col_name,
                &mut current_col_type,
                &mut current_col_hidden,
                &mut current_col_key,
                &mut columns,
            );
            state = ParseState::InPartition;
            continue;
        }

        // Process content within blocks
        match state {
            ParseState::InMeasure => {
                if trimmed.starts_with("formatString:") {
                    current_measure_format = Some(
                        trimmed
                            .strip_prefix("formatString:")
                            .unwrap_or("")
                            .trim()
                            .to_string(),
                    );
                } else if trimmed.starts_with("description:") {
                    let desc_raw = trimmed
                        .strip_prefix("description:")
                        .unwrap_or("")
                        .trim();
                    current_measure_description = Some(
                        desc_raw
                            .trim_matches('"')
                            .trim_matches('\'')
                            .to_string(),
                    );
                } else if !trimmed.is_empty() && current_measure_expression.is_empty() {
                    // Continuation of expression on next line
                    current_measure_expression = trimmed.to_string();
                }
            }
            ParseState::InColumn => {
                if trimmed.starts_with("dataType:") {
                    current_col_type = trimmed
                        .strip_prefix("dataType:")
                        .unwrap_or("")
                        .trim()
                        .to_string();
                } else if trimmed == "isHidden" {
                    current_col_hidden = true;
                } else if trimmed == "isKey" {
                    current_col_key = true;
                }
            }
            _ => {}
        }
    }

    // Flush any remaining items
    flush_measure(
        &mut current_measure_name,
        &mut current_measure_expression,
        &mut current_measure_format,
        &mut current_measure_description,
        &mut measures,
    );
    flush_column(
        &mut current_col_name,
        &mut current_col_type,
        &mut current_col_hidden,
        &mut current_col_key,
        &mut columns,
    );

    if table_name.is_empty() {
        return Err(IndexError::TmdlParse("no table name found".into()));
    }

    Ok(ParsedTable {
        name: table_name,
        columns,
        measures,
    })
}

fn expr_raw_to_string(s: &str) -> String {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        String::new()
    } else {
        trimmed.to_string()
    }
}

/// Parse an entire model directory structure into a ParsedModel.
/// Expects:
///   dir/model.tmdl
///   dir/tables/*.tmdl
pub fn parse_model_directory(dir: &Path) -> Result<ParsedModel, IndexError> {
    // Parse model header
    let model_path = dir.join("model.tmdl");
    let model_content = std::fs::read_to_string(&model_path).map_err(|e| {
        IndexError::TmdlParse(format!("failed to read model.tmdl: {e}"))
    })?;
    let model_info = parse_model_header(&model_content)?;

    // Parse all table files
    let tables_dir = dir.join("tables");
    let mut tables = Vec::new();
    let mut all_measures = Vec::new();

    if tables_dir.exists() {
        let mut entries: Vec<_> = std::fs::read_dir(&tables_dir)
            .map_err(|e| IndexError::TmdlParse(format!("failed to read tables dir: {e}")))?
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .is_some_and(|ext| ext == "tmdl")
            })
            .collect();
        entries.sort_by_key(|e| e.file_name());

        for entry in entries {
            let content = std::fs::read_to_string(entry.path()).map_err(|e| {
                IndexError::TmdlParse(format!(
                    "failed to read {}: {e}",
                    entry.path().display()
                ))
            })?;
            let table = parse_table(&content)?;
            all_measures.extend(table.measures.clone());
            tables.push(table);
        }
    }

    Ok(ParsedModel {
        model_name: model_info.name,
        tables,
        measures: all_measures,
    })
}

// tests at bottom (from Step 1)
```

**Step 4: Run test to verify it passes**

Run: `cd spool && cargo test -p spool-index -- tmdl`
Expected: 4 tests PASS

**Step 5: Commit**

```bash
git add spool/spool-index/
git commit -m "feat(spool-index): TMDL parser with fixture files — parses models, tables, columns, measures from TMDL format"
```

---

## Task 9: Tier 1 Generation From Parsed TMDL

**Files:**

- Create: `spool/spool-index/src/tier1_gen.rs`

**Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::tmdl::parse_model_directory;
    use std::path::PathBuf;

    fn fixtures_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("sales_model")
    }

    #[test]
    fn generate_tier1_from_parsed_model() {
        let parsed = parse_model_directory(&fixtures_dir()).unwrap();
        let tier1 = generate_tier1(&parsed, "model_sales");

        assert_eq!(tier1.schema_version, "1.0.0");
        assert_eq!(tier1.models.len(), 1);

        let model = &tier1.models[0];
        assert_eq!(model.model_id, "model_sales");
        assert_eq!(model.model_name, "Model");
        assert_eq!(model.tables.len(), 2);
        assert_eq!(model.measures.len(), 2);
    }

    #[test]
    fn generated_tables_have_correct_columns() {
        let parsed = parse_model_directory(&fixtures_dir()).unwrap();
        let tier1 = generate_tier1(&parsed, "model_sales");
        let model = &tier1.models[0];

        let fact_table = model
            .tables
            .iter()
            .find(|t| t.table_name == "FactSales")
            .unwrap();
        assert_eq!(fact_table.columns.len(), 3);

        let rev_col = fact_table
            .columns
            .iter()
            .find(|c| c.column_name == "Revenue")
            .unwrap();
        assert_eq!(rev_col.data_type, "decimal");
        assert!(!rev_col.is_hidden);

        let dk_col = fact_table
            .columns
            .iter()
            .find(|c| c.column_name == "DateKey")
            .unwrap();
        assert!(dk_col.is_hidden);
    }

    #[test]
    fn generated_measures_have_expressions() {
        let parsed = parse_model_directory(&fixtures_dir()).unwrap();
        let tier1 = generate_tier1(&parsed, "model_sales");
        let model = &tier1.models[0];

        let total_rev = model
            .measures
            .iter()
            .find(|m| m.measure_name == "Total Revenue")
            .unwrap();
        assert!(total_rev.expression.contains("SUM"));
        assert_eq!(total_rev.table_name, "FactSales");
        assert_eq!(total_rev.format_string, Some("#,##0.00".into()));
        assert_eq!(total_rev.description, Some("Sum of all revenue".into()));
    }

    #[test]
    fn generated_ids_are_stable_and_unique() {
        let parsed = parse_model_directory(&fixtures_dir()).unwrap();
        let tier1 = generate_tier1(&parsed, "model_sales");
        let model = &tier1.models[0];

        // All table IDs should be unique
        let table_ids: Vec<&str> = model.tables.iter().map(|t| t.table_id.as_str()).collect();
        let unique_table_ids: std::collections::HashSet<&&str> = table_ids.iter().collect();
        assert_eq!(table_ids.len(), unique_table_ids.len());

        // All measure IDs should be unique
        let measure_ids: Vec<&str> = model.measures.iter().map(|m| m.measure_id.as_str()).collect();
        let unique_measure_ids: std::collections::HashSet<&&str> = measure_ids.iter().collect();
        assert_eq!(measure_ids.len(), unique_measure_ids.len());

        // IDs should be deterministic
        let tier1_again = generate_tier1(&parsed, "model_sales");
        for (a, b) in model.tables.iter().zip(tier1_again.models[0].tables.iter()) {
            assert_eq!(a.table_id, b.table_id);
        }
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cd spool && cargo test -p spool-index -- tier1_gen`
Expected: FAIL

**Step 3: Write the implementation**

```rust
// spool/spool-index/src/tier1_gen.rs
use crate::tmdl::{ParsedModel, ParsedTable};
use spool_knowledge::tier1::{
    ColumnSchema, MeasureSchema, SemanticModelSchema, TableSchema, Tier1SchemaPackage,
};

/// Generate a Tier 1 schema package from a parsed TMDL model.
/// IDs are derived deterministically from model_id + artifact names
/// to ensure stability across regenerations.
pub fn generate_tier1(parsed: &ParsedModel, model_id: &str) -> Tier1SchemaPackage {
    let tables: Vec<TableSchema> = parsed
        .tables
        .iter()
        .map(|t| generate_table_schema(model_id, t))
        .collect();

    let measures: Vec<MeasureSchema> = parsed
        .tables
        .iter()
        .flat_map(|t| {
            t.measures.iter().map(move |m| MeasureSchema {
                measure_id: stable_id(model_id, "measure", &m.name),
                measure_name: m.name.clone(),
                table_name: t.name.clone(),
                expression: m.expression.clone(),
                format_string: m.format_string.clone(),
                description: m.description.clone(),
                aliases: Vec::new(), // aliases come from Tier 2
                is_hidden: false,
            })
        })
        .collect();

    Tier1SchemaPackage {
        schema_version: "1.0.0".into(),
        models: vec![SemanticModelSchema {
            model_id: model_id.into(),
            model_name: parsed.model_name.clone(),
            tables,
            measures,
            relationships: Vec::new(), // relationships parsed from separate files in full impl
        }],
    }
}

fn generate_table_schema(model_id: &str, table: &ParsedTable) -> TableSchema {
    let columns: Vec<ColumnSchema> = table
        .columns
        .iter()
        .map(|c| ColumnSchema {
            column_id: stable_id(model_id, "column", &format!("{}_{}", table.name, c.name)),
            column_name: c.name.clone(),
            data_type: c.data_type.clone(),
            is_hidden: c.is_hidden,
        })
        .collect();

    TableSchema {
        table_id: stable_id(model_id, "table", &table.name),
        table_name: table.name.clone(),
        columns,
        is_hidden: false,
    }
}

/// Generate a stable, deterministic ID from components.
/// Uses a simple convention: {model_id}_{type}_{sanitized_name}
fn stable_id(model_id: &str, artifact_type: &str, name: &str) -> String {
    let sanitized = name
        .to_lowercase()
        .replace(' ', "_")
        .replace('\'', "")
        .replace('"', "");
    format!("{model_id}_{artifact_type}_{sanitized}")
}

// tests at bottom (from Step 1)
```

**Step 4: Run test to verify it passes**

Run: `cd spool && cargo test -p spool-index -- tier1_gen`
Expected: 4 tests PASS

**Step 5: Commit**

```bash
git add spool/spool-index/src/tier1_gen.rs
git commit -m "feat(spool-index): Tier 1 generation from parsed TMDL — deterministic IDs, tables, columns, measures"
```

---

## Task 10: Bundle Builder — Assembles Tier 1 + Tier 2 Into a Validated Bundle

**Files:**

- Create: `spool/spool-index/src/bundle_builder.rs`

**Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::tier1_gen::generate_tier1;
    use crate::tmdl::parse_model_directory;
    use spool_knowledge::recipe::{Recipe, RecipeStep, WorkedExample, RecipeInput, RecipeInputType};
    use spool_knowledge::tier2::*;
    use std::path::PathBuf;

    fn fixtures_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("sales_model")
    }

    fn sample_tier2() -> Tier2Package {
        Tier2Package {
            contexts: vec![ContextArtifact {
                id: "ctx_revenue_model".into(),
                name: "revenue_model".into(),
                aliases: vec!["revenue data model".into()],
                description: "How revenue data is structured in the Sales Model".into(),
                body: "Revenue flows through FactSales with date dimension joins.".into(),
                key_relationships: vec!["FactSales joins to DimDate on DateKey".into()],
                usage_notes: vec!["Always filter by calendar year for YTD".into()],
            }],
            metrics: vec![MetricArtifact {
                id: "metric_total_revenue".into(),
                name: "total_revenue".into(),
                aliases: vec!["revenue".into(), "total rev".into()],
                description: "Sum of all revenue".into(),
                measure_refs: vec!["FactSales.Total Revenue".into()],
                formula: Some("SUM(FactSales[Revenue])".into()),
                disambiguation: vec![],
            }],
            rules: vec![RuleArtifact {
                id: "rule_date_filter".into(),
                name: "date_filter_selection".into(),
                aliases: vec!["date filtering".into()],
                description: "Which date filter to apply for revenue queries".into(),
                decision_logic: "IF YTD: filter CalendarYear = current year\nIF QoQ: use DATEADD".into(),
                why_it_matters: "Wrong date filter gives wrong totals".into(),
                examples: vec!["YTD revenue uses CalendarYear filter".into()],
            }],
            patterns: vec![PatternArtifact {
                id: "pattern_measure_validation".into(),
                name: "measure_validation".into(),
                aliases: vec!["measure check".into()],
                description: "Validate a measure by comparing DAX result to warehouse query".into(),
                when_to_apply: "When a measure value is disputed".into(),
                approach: vec![
                    "Run the measure DAX directly".into(),
                    "Run equivalent warehouse SQL".into(),
                    "Compare results".into(),
                ],
                anti_patterns: vec!["Do not assume measure is wrong without checking filters".into()],
            }],
            recipe_ids: vec!["recipe_report_number_mismatch".into()],
        }
    }

    fn sample_recipe() -> Recipe {
        Recipe {
            id: "recipe_report_number_mismatch".into(),
            name: "report_number_mismatch".into(),
            intent: "Diagnose report number mismatch".into(),
            problem_class: "number_mismatch".into(),
            applies_when: vec!["report number is disputed".into()],
            required_inputs: vec![RecipeInput {
                name: "report_reference".into(),
                description: "Report to investigate".into(),
                input_type: RecipeInputType::ArtifactReference,
                required: true,
            }],
            recommended_artifact_targets: vec!["report".into(), "measure".into()],
            expected_evidence_classes: vec![
                "report_metadata".into(),
                "measure_definition".into(),
                "dax_query_result".into(),
            ],
            validation_expectations: vec!["compare against direct query".into()],
            steps: vec![
                RecipeStep {
                    order: 1,
                    description: "Resolve report".into(),
                    evidence_class: Some("report_metadata".into()),
                    artifact_target: Some("report".into()),
                },
                RecipeStep {
                    order: 2,
                    description: "Run diagnostic DAX".into(),
                    evidence_class: Some("dax_query_result".into()),
                    artifact_target: None,
                },
            ],
            anti_patterns: vec!["Do not assume warehouse is always right".into()],
            worked_examples: vec![WorkedExample {
                title: "Q1 revenue mismatch".into(),
                scenario: "Report shows $12.4M, warehouse shows $13.1M".into(),
                walkthrough: vec!["Checked measure definition".into(), "Found implicit filter".into()],
                outcome: "Confirmed filter discrepancy".into(),
            }],
        }
    }

    #[test]
    fn build_valid_bundle() {
        let parsed = parse_model_directory(&fixtures_dir()).unwrap();
        let tier1 = generate_tier1(&parsed, "model_sales");
        let tier2 = sample_tier2();
        let recipes = vec![sample_recipe()];

        let result = build_bundle(
            "finance",
            "Finance LOB Bundle",
            "model_sales",
            tier1,
            tier2,
            recipes,
        );

        assert!(result.is_ok());
        let bundle = result.unwrap();
        assert_eq!(bundle.manifest.lob_id, "finance");
        assert_eq!(bundle.manifest.declared_recipe_ids.len(), 1);
        assert_eq!(bundle.manifest.declared_artifact_classes.len(), 4);
        assert!(bundle.validation_result.is_valid());
    }

    #[test]
    fn build_bundle_with_validation_errors() {
        let parsed = parse_model_directory(&fixtures_dir()).unwrap();
        let tier1 = generate_tier1(&parsed, "model_sales");
        let tier2 = sample_tier2();
        // Missing recipe — declared in tier2 but not provided
        let recipes: Vec<Recipe> = vec![];

        let result = build_bundle(
            "finance",
            "Finance LOB Bundle",
            "model_sales",
            tier1,
            tier2,
            recipes,
        );

        assert!(result.is_ok());
        let bundle = result.unwrap();
        assert!(!bundle.validation_result.is_valid());
    }

    #[test]
    fn build_bundle_manifest_has_correct_artifact_classes() {
        let parsed = parse_model_directory(&fixtures_dir()).unwrap();
        let tier1 = generate_tier1(&parsed, "model_sales");
        let tier2 = sample_tier2();
        let recipes = vec![sample_recipe()];

        let bundle = build_bundle(
            "finance",
            "Finance LOB Bundle",
            "model_sales",
            tier1,
            tier2,
            recipes,
        )
        .unwrap();

        let classes = &bundle.manifest.declared_artifact_classes;
        assert!(classes.contains(&"context".to_string()));
        assert!(classes.contains(&"metric".to_string()));
        assert!(classes.contains(&"rule".to_string()));
        assert!(classes.contains(&"pattern".to_string()));
    }

    #[test]
    fn built_bundle_serializes_to_json() {
        let parsed = parse_model_directory(&fixtures_dir()).unwrap();
        let tier1 = generate_tier1(&parsed, "model_sales");
        let tier2 = sample_tier2();
        let recipes = vec![sample_recipe()];

        let bundle = build_bundle(
            "finance",
            "Finance LOB Bundle",
            "model_sales",
            tier1,
            tier2,
            recipes,
        )
        .unwrap();

        let json = serde_json::to_string_pretty(&bundle).unwrap();
        let restored: BuiltBundle = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.manifest.bundle_id, bundle.manifest.bundle_id);
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cd spool && cargo test -p spool-index -- bundle_builder`
Expected: FAIL

**Step 3: Write the implementation**

```rust
// spool/spool-index/src/bundle_builder.rs
use chrono::Utc;
use serde::{Deserialize, Serialize};

use spool_knowledge::bundle::BundleManifest;
use spool_knowledge::recipe::Recipe;
use spool_knowledge::tier1::Tier1SchemaPackage;
use spool_knowledge::tier2::Tier2Package;
use spool_knowledge::validation::{self, BundleValidationResult};

use crate::error::IndexError;

/// A fully assembled bundle with validation results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuiltBundle {
    pub manifest: BundleManifest,
    pub tier1: Tier1SchemaPackage,
    pub tier2: Tier2Package,
    pub recipes: Vec<Recipe>,
    pub validation_result: BundleValidationResult,
}

/// Build a complete LOB bundle from Tier 1, Tier 2, and recipes.
/// Runs validation and returns the bundle with results attached.
pub fn build_bundle(
    lob_id: &str,
    display_name: &str,
    model_id: &str,
    tier1: Tier1SchemaPackage,
    tier2: Tier2Package,
    recipes: Vec<Recipe>,
) -> Result<BuiltBundle, IndexError> {
    let declared_artifact_classes = compute_artifact_classes(&tier2);
    let declared_recipe_ids: Vec<String> = tier2.recipe_ids.clone();

    let manifest = BundleManifest {
        bundle_id: format!("bundle_{lob_id}_{}", Utc::now().format("%Y%m%d%H%M%S")),
        lob_id: lob_id.into(),
        version: "1.0.0".into(),
        display_name: display_name.into(),
        default_workspace_scope: None,
        tier1_schema_version: tier1.schema_version.clone(),
        tier2_bundle_version: "1.0.0".into(),
        build_timestamp: Utc::now(),
        source_summary: format!("Generated from model {model_id} with curated Tier 2 knowledge"),
        declared_artifact_classes,
        declared_recipe_ids,
    };

    let validation_result = validation::validate_bundle(&manifest, &tier1, &tier2, &recipes);

    Ok(BuiltBundle {
        manifest,
        tier1,
        tier2,
        recipes,
        validation_result,
    })
}

fn compute_artifact_classes(tier2: &Tier2Package) -> Vec<String> {
    let mut classes = Vec::new();
    if !tier2.contexts.is_empty() {
        classes.push("context".into());
    }
    if !tier2.metrics.is_empty() {
        classes.push("metric".into());
    }
    if !tier2.rules.is_empty() {
        classes.push("rule".into());
    }
    if !tier2.patterns.is_empty() {
        classes.push("pattern".into());
    }
    classes
}

// tests at bottom (from Step 1)
```

**Step 4: Run test to verify it passes**

Run: `cd spool && cargo test -p spool-index -- bundle_builder`
Expected: 4 tests PASS

**Step 5: Commit**

```bash
git add spool/spool-index/src/bundle_builder.rs
git commit -m "feat(spool-index): bundle builder assembles Tier 1 + Tier 2 into validated bundle with manifest generation"
```

---

## Task 11: Fixture LOB Bundle — End-to-End Integration Test

**Files:**

- Create: `spool/spool-index/tests/integration_bundle.rs`
- Create: `spool/spool-index/tests/fixtures/finance_tier2/contexts/revenue_model.yml`
- Create: `spool/spool-index/tests/fixtures/finance_tier2/metrics/total_revenue.yml`
- Create: `spool/spool-index/tests/fixtures/finance_tier2/rules/date_filter_selection.yml`
- Create: `spool/spool-index/tests/fixtures/finance_tier2/patterns/measure_validation.yml`
- Create: `spool/spool-index/tests/fixtures/finance_tier2/recipes/report_number_mismatch.yml`

**Step 1: Create fixture Tier 2 YAML files**

```yaml
# spool/spool-index/tests/fixtures/finance_tier2/contexts/revenue_model.yml
id: ctx_revenue_model
name: revenue_model
aliases:
  - revenue data model
description: "How revenue data is structured in the Sales Model"
body: "Revenue flows through FactSales with date dimension joins."
key_relationships:
  - "FactSales joins to DimDate on DateKey"
usage_notes:
  - "Always filter by calendar year for YTD"
```

```yaml
# spool/spool-index/tests/fixtures/finance_tier2/metrics/total_revenue.yml
id: metric_total_revenue
name: total_revenue
aliases:
  - revenue
  - total rev
description: "Sum of all revenue"
measure_refs:
  - "FactSales.Total Revenue"
formula: "SUM(FactSales[Revenue])"
disambiguation: []
```

```yaml
# spool/spool-index/tests/fixtures/finance_tier2/rules/date_filter_selection.yml
id: rule_date_filter
name: date_filter_selection
aliases:
  - date filtering
description: "Which date filter to apply for revenue queries"
decision_logic: |
  IF YTD: filter CalendarYear = current year
  IF QoQ: use DATEADD
why_it_matters: "Wrong date filter gives wrong totals"
examples:
  - "YTD revenue uses CalendarYear filter"
```

```yaml
# spool/spool-index/tests/fixtures/finance_tier2/patterns/measure_validation.yml
id: pattern_measure_validation
name: measure_validation
aliases:
  - measure check
description: "Validate a measure by comparing DAX result to warehouse query"
when_to_apply: "When a measure value is disputed"
approach:
  - "Run the measure DAX directly"
  - "Run equivalent warehouse SQL"
  - "Compare results"
anti_patterns:
  - "Do not assume measure is wrong without checking filters"
```

```yaml
# spool/spool-index/tests/fixtures/finance_tier2/recipes/report_number_mismatch.yml
id: recipe_report_number_mismatch
name: report_number_mismatch
intent: "Diagnose why a report visual does not match expected totals"
problem_class: number_mismatch
applies_when:
  - "report number is disputed"
  - "semantic model and warehouse validation may both be relevant"
required_inputs:
  - name: report_reference
    description: "Report URL, name, or ID"
    input_type: artifact_reference
    required: true
  - name: expected_value
    description: "The number the user expects to see"
    input_type: user_provided
    required: false
recommended_artifact_targets:
  - report
  - semantic_model
  - measure
expected_evidence_classes:
  - report_metadata
  - measure_definition
  - dax_query_result
  - warehouse_query_result
validation_expectations:
  - "compare report output against at least one direct query-based validation"
steps:
  - order: 1
    description: "Resolve report, page, and visual"
    evidence_class: report_metadata
    artifact_target: report
  - order: 2
    description: "Identify backing semantic model object"
    evidence_class: semantic_model_metadata
    artifact_target: semantic_model
  - order: 3
    description: "Inspect relevant measure definitions and filters"
    evidence_class: measure_definition
    artifact_target: measure
  - order: 4
    description: "Run diagnostic DAX query"
    evidence_class: dax_query_result
    artifact_target: null
  - order: 5
    description: "Run warehouse validation query when appropriate"
    evidence_class: warehouse_query_result
    artifact_target: null
  - order: 6
    description: "Classify likely mismatch source"
    evidence_class: null
    artifact_target: null
anti_patterns:
  - "Assume warehouse mismatch before checking measure logic"
  - "Rely on report screenshot alone"
worked_examples:
  - title: "Q1 revenue mismatch between report and warehouse"
    scenario: "User reports Executive Revenue Report shows $12.4M for Q1 but warehouse shows $13.1M"
    walkthrough:
      - "Resolved report to workspace Executive BI, page Revenue Overview"
      - "Identified backing model: Sales Model"
      - "Inspected Total Revenue measure: SUM(FactSales[Revenue])"
      - "DAX query returned $12.4M — matches report"
      - "Warehouse query returned $13.1M — mismatch confirmed"
      - "Root cause: measure has an implicit date filter excluding late-arriving transactions"
    outcome: "Confirmed mismatch due to implicit date filter on measure"
```

**Step 2: Write the integration test**

```rust
// spool/spool-index/tests/integration_bundle.rs

use spool_index::bundle_builder::build_bundle;
use spool_index::tier1_gen::generate_tier1;
use spool_index::tmdl::parse_model_directory;
use spool_knowledge::bundle::BundleLoadStatus;
use spool_knowledge::loading::{BundleSource, LoadingPolicy};
use spool_knowledge::recipe::Recipe;
use spool_knowledge::tier2::*;
use std::path::PathBuf;

fn tmdl_fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("sales_model")
}

fn tier2_fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("finance_tier2")
}

fn load_fixture_context(name: &str) -> ContextArtifact {
    let path = tier2_fixtures_dir().join("contexts").join(format!("{name}.yml"));
    let content = std::fs::read_to_string(path).unwrap();
    serde_yaml::from_str(&content).unwrap()
}

fn load_fixture_metric(name: &str) -> MetricArtifact {
    let path = tier2_fixtures_dir().join("metrics").join(format!("{name}.yml"));
    let content = std::fs::read_to_string(path).unwrap();
    serde_yaml::from_str(&content).unwrap()
}

fn load_fixture_rule(name: &str) -> RuleArtifact {
    let path = tier2_fixtures_dir().join("rules").join(format!("{name}.yml"));
    let content = std::fs::read_to_string(path).unwrap();
    serde_yaml::from_str(&content).unwrap()
}

fn load_fixture_pattern(name: &str) -> PatternArtifact {
    let path = tier2_fixtures_dir().join("patterns").join(format!("{name}.yml"));
    let content = std::fs::read_to_string(path).unwrap();
    serde_yaml::from_str(&content).unwrap()
}

fn load_fixture_recipe(name: &str) -> Recipe {
    let path = tier2_fixtures_dir().join("recipes").join(format!("{name}.yml"));
    let content = std::fs::read_to_string(path).unwrap();
    serde_yaml::from_str(&content).unwrap()
}

#[test]
fn end_to_end_bundle_build_from_fixtures() {
    // Step 1: Parse TMDL fixtures
    let parsed_model = parse_model_directory(&tmdl_fixtures_dir()).unwrap();
    assert_eq!(parsed_model.model_name, "Model");
    assert_eq!(parsed_model.tables.len(), 2);

    // Step 2: Generate Tier 1
    let tier1 = generate_tier1(&parsed_model, "model_sales");
    assert_eq!(tier1.models[0].measures.len(), 2);

    // Step 3: Load Tier 2 fixtures
    let tier2 = Tier2Package {
        contexts: vec![load_fixture_context("revenue_model")],
        metrics: vec![load_fixture_metric("total_revenue")],
        rules: vec![load_fixture_rule("date_filter_selection")],
        patterns: vec![load_fixture_pattern("measure_validation")],
        recipe_ids: vec!["recipe_report_number_mismatch".into()],
    };

    // Step 4: Load recipe fixtures
    let recipes = vec![load_fixture_recipe("report_number_mismatch")];

    // Step 5: Build the bundle
    let built = build_bundle(
        "finance",
        "Finance LOB Bundle",
        "model_sales",
        tier1,
        tier2,
        recipes,
    )
    .unwrap();

    // Step 6: Verify bundle integrity
    assert_eq!(built.manifest.lob_id, "finance");
    assert!(built.validation_result.is_valid());
    assert_eq!(built.manifest.declared_artifact_classes.len(), 4);
    assert_eq!(built.manifest.declared_recipe_ids.len(), 1);
    assert_eq!(built.recipes.len(), 1);
    assert_eq!(built.recipes[0].steps.len(), 6);
    assert_eq!(built.recipes[0].anti_patterns.len(), 2);
    assert_eq!(built.recipes[0].worked_examples.len(), 1);
}

#[test]
fn end_to_end_loading_policy_fully_loaded() {
    let parsed_model = parse_model_directory(&tmdl_fixtures_dir()).unwrap();
    let tier1 = generate_tier1(&parsed_model, "model_sales");

    let tier2 = Tier2Package {
        contexts: vec![load_fixture_context("revenue_model")],
        metrics: vec![load_fixture_metric("total_revenue")],
        rules: vec![load_fixture_rule("date_filter_selection")],
        patterns: vec![load_fixture_pattern("measure_validation")],
        recipe_ids: vec!["recipe_report_number_mismatch".into()],
    };

    let recipes = vec![load_fixture_recipe("report_number_mismatch")];

    let source = BundleSource {
        manifest: spool_knowledge::bundle::BundleManifest {
            bundle_id: "bundle_finance".into(),
            lob_id: "finance".into(),
            version: "1.0.0".into(),
            display_name: "Finance LOB Bundle".into(),
            default_workspace_scope: None,
            tier1_schema_version: "1.0.0".into(),
            tier2_bundle_version: "1.0.0".into(),
            build_timestamp: chrono::Utc::now(),
            source_summary: "fixture test".into(),
            declared_artifact_classes: vec!["context".into(), "metric".into(), "rule".into(), "pattern".into()],
            declared_recipe_ids: vec!["recipe_report_number_mismatch".into()],
        },
        tier1: Some(tier1),
        tier2: Some(tier2),
        recipes,
    };

    let policy = LoadingPolicy {
        selected_lob: "finance".into(),
    };

    let loaded = policy.load(&[source]).unwrap();
    assert_eq!(loaded.load_status, BundleLoadStatus::FullyLoaded);
    assert!(loaded.tier1.is_some());
    assert!(loaded.tier2.is_some());
    assert_eq!(loaded.manifest.lob_id, "finance");
}

#[test]
fn end_to_end_cold_start_with_tier1_only() {
    let parsed_model = parse_model_directory(&tmdl_fixtures_dir()).unwrap();
    let tier1 = generate_tier1(&parsed_model, "model_sales");

    let source = BundleSource {
        manifest: spool_knowledge::bundle::BundleManifest {
            bundle_id: "bundle_finance_cold".into(),
            lob_id: "finance".into(),
            version: "1.0.0".into(),
            display_name: "Finance LOB Bundle (cold)".into(),
            default_workspace_scope: None,
            tier1_schema_version: "1.0.0".into(),
            tier2_bundle_version: "0.0.0".into(),
            build_timestamp: chrono::Utc::now(),
            source_summary: "cold start test".into(),
            declared_artifact_classes: vec![],
            declared_recipe_ids: vec![],
        },
        tier1: Some(tier1),
        tier2: None,
        recipes: vec![],
    };

    let policy = LoadingPolicy {
        selected_lob: "finance".into(),
    };

    let loaded = policy.load(&[source]).unwrap();
    assert_eq!(loaded.load_status, BundleLoadStatus::Tier1Only);
    assert!(loaded.tier1.is_some());
    assert!(loaded.tier2.is_none());
    assert!(!loaded.validation_warnings.is_empty());
}

#[test]
fn end_to_end_recipe_yaml_round_trip() {
    let recipe = load_fixture_recipe("report_number_mismatch");
    assert_eq!(recipe.id, "recipe_report_number_mismatch");
    assert_eq!(recipe.steps.len(), 6);
    assert_eq!(recipe.required_inputs.len(), 2);
    assert!(recipe.required_inputs[0].required);
    assert!(!recipe.required_inputs[1].required);
    assert_eq!(recipe.worked_examples.len(), 1);
    assert_eq!(recipe.worked_examples[0].walkthrough.len(), 6);

    // Re-serialize and re-parse to verify round-trip
    let yaml = serde_yaml::to_string(&recipe).unwrap();
    let restored: Recipe = serde_yaml::from_str(&yaml).unwrap();
    assert_eq!(restored.id, recipe.id);
    assert_eq!(restored.steps.len(), recipe.steps.len());
}

#[test]
fn end_to_end_validation_catches_broken_bundle() {
    let parsed_model = parse_model_directory(&tmdl_fixtures_dir()).unwrap();
    let tier1 = generate_tier1(&parsed_model, "model_sales");

    // Create tier2 with a broken measure ref and a duplicate ID
    let tier2 = Tier2Package {
        contexts: vec![ContextArtifact {
            id: "metric_total_revenue".into(), // deliberate duplicate with metric below
            name: "bad_context".into(),
            aliases: vec![],
            description: "bad".into(),
            body: "bad".into(),
            key_relationships: vec![],
            usage_notes: vec![],
        }],
        metrics: vec![MetricArtifact {
            id: "metric_total_revenue".into(),
            name: "total_revenue".into(),
            aliases: vec![],
            description: "".into(),
            measure_refs: vec!["NonExistent.GhostMeasure".into()],
            formula: None,
            disambiguation: vec![],
        }],
        rules: vec![],
        patterns: vec![],
        recipe_ids: vec!["recipe_missing".into()],
    };

    let built = build_bundle(
        "finance",
        "Finance LOB Bundle (broken)",
        "model_sales",
        tier1,
        tier2,
        vec![], // no recipes — recipe_missing will fail validation
    )
    .unwrap();

    assert!(!built.validation_result.is_valid());

    // Should have errors for: duplicate ID + missing recipe
    assert!(built.validation_result.errors.len() >= 2);

    // Should have warning for: unresolved measure ref
    assert!(!built.validation_result.warnings.is_empty());
}
```

**Step 2: Run tests to verify they fail**

Run: `cd spool && cargo test --test integration_bundle`
Expected: FAIL (fixtures not yet created, or compilation errors)

**Step 3: Ensure all fixture files and modules are properly created and exported**

Verify all fixture YAML files exist at the paths specified in Step 1.

Verify `spool/spool-index/src/lib.rs` exports all modules:

```rust
pub mod tmdl;
pub mod tier1_gen;
pub mod bundle_builder;
pub mod error;
```

Verify `spool/spool-knowledge/src/lib.rs` exports all modules:

```rust
pub mod bundle;
pub mod tier1;
pub mod tier2;
pub mod recipe;
pub mod validation;
pub mod loading;
pub mod error;
```

**Step 4: Run tests to verify they pass**

Run: `cd spool && cargo test --test integration_bundle`
Expected: 5 tests PASS

Then run the full test suite:

Run: `cd spool && cargo test`
Expected: all tests PASS (spool-knowledge + spool-index + integration)

**Step 5: Commit**

```bash
git add spool/spool-index/tests/ spool/spool-index/src/lib.rs spool/spool-knowledge/src/lib.rs
git commit -m "feat(spool-index): end-to-end integration tests — fixture LOB bundle with TMDL parsing, Tier 1 generation, Tier 2 loading, validation, and loading policy"
```

---

## Summary

| Task | What it proves | Test count |
|------|---------------|------------|
| 1 | Workspace builds with spool-knowledge + spool-index | 0 (build check) |
| 2 | Bundle manifest schema | 3 |
| 3 | Tier 1 schema package types | 5 |
| 4 | Tier 2 curated knowledge types | 6 |
| 5 | Recipe schema with selection and deviation | 7 |
| 6 | Bundle validation across both tiers | 7 |
| 7 | Loading policy with cold-start fallback | 8 |
| 8 | TMDL parser with fixture files | 4 |
| 9 | Tier 1 generation from parsed TMDL | 4 |
| 10 | Bundle builder with validation | 4 |
| 11 | End-to-end integration with fixture bundle | 5 |
| **Total** | | **53** |
