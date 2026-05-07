# Git Tooling Guide

Git state management utilities for Codex CLI using "ghost commits" - invisible Git snapshots that don't affect repository history.

## 🚀 Quick Start

## 🚀 Quick Start (TL;DR)

**What it does**: Creates invisible Git snapshots for undo functionality.

**How it works**:
```rust
// 1. Create snapshot (before AI changes)
let options = CreateGhostCommitOptions::new(repo_path);
let ghost = create_ghost_commit(&options)?; 

// 2. Undo when needed  
restore_ghost_commit(repo_path, &ghost)?;
```

**Where snapshots live**: `.git/objects/` as loose Git objects (invisible to `git log`)

**Requirements**: ⚠️ **Git repository required** - feature auto-disables in non-Git directories

**In Codex CLI**: 
- 🔄 Auto-snapshots before each AI interaction
- ⚡ `/undo` command restores last snapshot  
- 📚 Tracks up to 20 snapshots in memory

**Key insight**: Uses `git commit-tree` to create commits without updating branches/refs.

**What it does**: Creates invisible Git snapshots for undo functionality.

```rust
// Create snapshot (before AI changes)
let ghost = create_ghost_commit(&CreateGhostCommitOptions::new(repo_path))?;

// Undo when needed  
restore_ghost_commit(repo_path, &ghost)?;
```

**Storage**: `.git/objects/` as loose Git objects (invisible to `git log`)

**Requirements**: ⚠️ **Git repository required** - auto-disables in non-Git directories

**In Codex CLI**: 
- 🔄 Auto-snapshots before each AI interaction
- ⚡ `/undo` command restores last snapshot  
- 📚 Tracks up to 20 snapshots in memory

## Core Types

### `GhostCommit`
```rust
pub struct GhostCommit {
    id: String,           // Commit SHA
    parent: Option<String>, // Parent commit SHA
}
```

### `CreateGhostCommitOptions<'a>`
```rust
pub struct CreateGhostCommitOptions<'a> {
    pub repo_path: &'a Path,        // Repository path
    pub message: Option<&'a str>,   // Custom commit message
    pub force_include: Vec<PathBuf>, // Include ignored files
}
```

**Lifetime rationale**: Borrows lightweight references (`repo_path`, `message`) for efficiency, owns complex collection (`force_include`) for flexibility.

## Core Functions

### `create_ghost_commit(options) -> GhostCommit`
Creates invisible snapshot using `git commit-tree` (doesn't update branches/refs).

### `restore_ghost_commit(repo_path, ghost) -> ()`
Restores working tree using `git restore --source=<ghost-sha>`.

### `restore_to_commit(repo_path, commit_id) -> ()`
Restores from any commit SHA.

## Usage Examples

### Basic Usage
```rust
let options = CreateGhostCommitOptions::new(repo);
let ghost = create_ghost_commit(&options)?;
// ... make changes ...
restore_ghost_commit(repo, &ghost)?;
```

### Include Ignored Files
```rust
let options = CreateGhostCommitOptions::new(repo)
    .force_include(vec![".env".into(), "logs/debug.log".into()]);
let ghost = create_ghost_commit(&options)?;
```

### Custom Message
```rust
let ghost = create_ghost_commit(
    &CreateGhostCommitOptions::new(repo).message("Before AI changes")
)?;
```

### Subdirectory Operations
```rust
let workspace = repo.join("src");
let ghost = create_ghost_commit(&CreateGhostCommitOptions::new(&workspace))?;
// Only affects src/ directory
```

## How It Works

1. **Snapshot Creation**:
   - Creates temporary Git index
   - Stages all files with `git add --all`
   - Optionally force-includes ignored files with `git add --force`
   - Creates tree with `git write-tree`
   - Creates commit with `git commit-tree` (no ref updates)

2. **Restoration**:
   - Uses `git restore --source=<ghost-sha>` to restore files
   - Preserves ignored files not in snapshot

## Error Handling

```rust
pub enum GitToolingError {
    GitCommand { command: String, status: ExitStatus, stderr: String },
    NotAGitRepository { path: PathBuf },
    NonRelativePath { path: PathBuf },
    PathEscapesRepository { path: PathBuf },
    // ... other variants
}
```

## Security Features

- **Path validation**: Prevents directory traversal attacks
- **Repository boundaries**: Cannot escape repository root
- **Command safety**: No shell injection vulnerabilities

## Platform Support

- **Unix/Linux/macOS**: Full support
- **Windows**: Full support with platform-specific symlink handling
- **Dependencies**: `tempfile`, `thiserror`, `walkdir`

## Integration Details

### Codex CLI Integration
- **TUI**: Auto-creates snapshots before AI interactions
- **Slash command**: `/undo` restores last snapshot
- **Error handling**: Gracefully disables in non-Git directories
- **Limit**: Tracks up to 20 ghost commits in memory

### Key Benefits
- **Non-destructive**: No impact on Git history
- **Complete**: Captures tracked, untracked, and optionally ignored files
- **Safe**: Path traversal protection and security validation
- **Efficient**: Leverages Git's object storage and deduplication