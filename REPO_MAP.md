# REPO_MAP.md

This file is the canonical ownership map for the `run_spider` repo. Treat it as the source of truth for where changes belong.

## Root Files

| Path | Role |
| --- | --- |
| [AGENTS.md](E:/project/run_spider/AGENTS.md) | Project-specific agent rules, skill routing, and repo workflow constraints. |
| [REPO_MAP.md](E:/project/run_spider/REPO_MAP.md) | Canonical map of maintained files and directories. Update this file whenever maintained structure changes. |
| [README.md](E:/project/run_spider/README.md) | User-facing product overview, install/use guidance, and release-facing behavior notes. |
| [Cargo.toml](E:/project/run_spider/Cargo.toml) | Rust package manifest, feature flags, binary targets, and dependency declarations. |
| [Cargo.lock](E:/project/run_spider/Cargo.lock) | Locked dependency graph for reproducible builds. |
| [build.rs](E:/project/run_spider/build.rs) | Build-time resource preparation, especially Slint compilation and Windows icon synchronization. |
| [.gitignore](E:/project/run_spider/.gitignore) | Ignore rules for generated outputs and local-only directories. |

## Repo-Owned Skills

| Path | Role |
| --- | --- |
| [SKILL.md](E:/project/run_spider/.agents/skills/gemini-frontend-review-gate/SKILL.md) | Gemini-first workflow for UI/frontend implementation and review. |
| [openai.yaml](E:/project/run_spider/.agents/skills/gemini-frontend-review-gate/agents/openai.yaml) | Agent wiring for the frontend Gemini workflow. |
| [gemini-handoff-examples.md](E:/project/run_spider/.agents/skills/gemini-frontend-review-gate/references/gemini-handoff-examples.md) | Handoff examples for Gemini-first UI tasks. |
| [SKILL.md](E:/project/run_spider/.agents/skills/code-dev-standards/SKILL.md) | Project-owned coding workflow for code changes, cleanup, tests, and self-checks. |
| [openai.yaml](E:/project/run_spider/.agents/skills/code-dev-standards/agents/openai.yaml) | Agent wiring for the coding standards skill. |
| [coding_rules.md](E:/project/run_spider/.agents/skills/code-dev-standards/references/coding_rules.md) | Detailed implementation and code hygiene rules. |
| [data_structures_algorithms.md](E:/project/run_spider/.agents/skills/code-dev-standards/references/data_structures_algorithms.md) | Algorithm/pattern guidance for performance-sensitive logic. |
| [engineering_templates_python_go_rust.md](E:/project/run_spider/.agents/skills/code-dev-standards/references/engineering_templates_python_go_rust.md) | Skeleton guidance for reusable scripts and small projects. |
| [language_notes_python_go_rust.md](E:/project/run_spider/.agents/skills/code-dev-standards/references/language_notes_python_go_rust.md) | Language-specific notes for Python, Go, and Rust. |
| [problem_type_mapping_python_go_rust.md](E:/project/run_spider/.agents/skills/code-dev-standards/references/problem_type_mapping_python_go_rust.md) | Data-structure/problem-type mapping guidance for Python, Go, and Rust. |

## Source Tree

### Library and Desktop Runtime

| Path | Role |
| --- | --- |
| [lib.rs](E:/project/run_spider/src/lib.rs) | Public module wiring for the shared Rust crate. |
| [main.rs](E:/project/run_spider/src/main.rs) | Desktop binary entrypoint; launches the Slint desktop app. |
| [desktop.rs](E:/project/run_spider/src/desktop.rs) | Desktop controller, UI synchronization, workspace/session state projection, action callbacks, and desktop-side tests/benchmarks. |
| [runtime.rs](E:/project/run_spider/src/runtime.rs) | Background process execution, output streaming, Codex output filtering, retry logic, and runtime event delivery. |
| [codex.rs](E:/project/run_spider/src/codex.rs) | Codex command construction, session discovery, history summarization, and command-launch abstraction. |
| [model.rs](E:/project/run_spider/src/model.rs) | In-memory workspace/session/output data model used by desktop and runtime layers. |
| [persistence.rs](E:/project/run_spider/src/persistence.rs) | Local app-state persistence and atomic config writes. |
| [diagnostics.rs](E:/project/run_spider/src/diagnostics.rs) | Panic hook and local diagnostic logging. |

### CLI

| Path | Role |
| --- | --- |
| [crl.rs](E:/project/run_spider/src/bin/crl.rs) | CLI entrypoint for resume loops, new conversation flow, install/uninstall, and CLI-side regression tests. |

## UI Assets

| Path | Role |
| --- | --- |
| [main.slint](E:/project/run_spider/ui/main.slint) | Slint desktop UI definition, callbacks, layouts, and widget wiring. |
| [crl-icon.svg](E:/project/run_spider/ui/assets/crl-icon.svg) | Canonical vector app logo. |
| [crl-icon.ico](E:/project/run_spider/ui/assets/crl-icon.ico) | Windows icon artifact generated from the SVG for exe/installer resources. |

## Packaging

### Windows Packaging

| Path | Role |
| --- | --- |
| [build-installer.ps1](E:/project/run_spider/packaging/windows/build-installer.ps1) | Builds the Windows release binaries and packages them with Inno Setup. |
| [crl.iss](E:/project/run_spider/packaging/windows/crl.iss) | Inno Setup installer script, including install/update migration and uninstall behavior. |
| [sync-icon.py](E:/project/run_spider/packaging/windows/sync-icon.py) | Regenerates the Windows `.ico` from the SVG logo before packaging. |
| [test-install-migration.ps1](E:/project/run_spider/packaging/windows/test-install-migration.ps1) | Validates install-time cleanup of legacy CRL installs, shims, and PATH entries. |
| [test-uninstall-e2e.ps1](E:/project/run_spider/packaging/windows/test-uninstall-e2e.ps1) | End-to-end uninstall validation for CLI uninstall flow and history cleanup. |

### Linux Packaging

| Path | Role |
| --- | --- |
| [build-cli.ps1](E:/project/run_spider/packaging/linux/build-cli.ps1) | Produces the Linux CLI archive from current source. |
| [install.sh](E:/project/run_spider/packaging/linux/install.sh) | Installs the Linux CLI archive into a directly runnable `crl` command. |

## Generated or Derived Outputs

These paths are not maintained source of truth and must not be hand-edited:

- `dist/`
  - Canonical local release artifacts only. Keep the latest installer/archive here; regenerate rather than edit.
- `target/`
  - Cargo build cache and compiled outputs.
- `dist-target*`
  - Temporary or historical build trees used for isolated packaging runs and experiments.
- `.slint/`
  - Slint-generated local state if present.

These directories are safe to delete when slimming the repo or resetting build state.

## Change Rules

- Every tracked code or file edit must map to a path documented above.
- If a change introduces a new maintained file or removes a maintained file, update this document in the same change.
- Do not place production logic in generated directories.
- Do not use build artifacts as evidence of implementation unless the corresponding source file above is also updated.
