# AGENTS.md

## Repository Grounding

- Before editing any tracked file, read `README.md` and `REPO_MAP.md`.
- Treat `REPO_MAP.md` as the canonical ownership map for the repo. Every code or file change must align to an existing entry there.
- Any change that adds, removes, renames, or repurposes a maintained file or directory must update `REPO_MAP.md` in the same change.
- Generated output directories are not sources of truth. Do not hand-edit build artifacts in `dist/`, `target/`, or any `dist-target*` tree.

## Project Skills

### Repo-owned skills

- `gemini-frontend-review-gate`
  - Use for UI, UX, layout, styling, or other user-facing interface work.
  - File: [./.agents/skills/gemini-frontend-review-gate/SKILL.md](E:/project/run_spider/.agents/skills/gemini-frontend-review-gate/SKILL.md)

- `code-dev-standards`
  - Use for code writing, code edits, refactors, scripts, and test-bearing implementation work.
  - File: [./.agents/skills/code-dev-standards/SKILL.md](E:/project/run_spider/.agents/skills/code-dev-standards/SKILL.md)

### Machine-available skills used in this project

- `bug-fix-first`
  - Use first for bugs, regressions, crashes, wrong results, flaky behavior, UI defects, API defects, and performance issues.
  - File: [bug-fix-first](C:/Users/shcem/.codex/skills/bug-fix-first/SKILL.md)

- `testing-verifier`
  - Use when adding or strengthening tests, regression coverage, stress checks, or verification plans.
  - File: [testing-verifier](C:/Users/shcem/.codex/skills/testing-verifier/SKILL.md)

- `document-consistency-editor`
  - Use when updating docs that affect related docs or root navigation files.
  - File: [document-consistency-editor](C:/Users/shcem/.codex/skills/document-consistency-editor/SKILL.md)

## Trigger Rules

- For any code write, code edit, refactor, script change, or implementation task, use `code-dev-standards`.
- For any bug, regression, performance issue, or incorrect behavior, use `bug-fix-first` first, then use `code-dev-standards` for the implementation pass.
- For any test-heavy change or when new regression coverage is needed, use `testing-verifier` after the implementation path is known.
- For any UI or frontend work, use `gemini-frontend-review-gate` first, then `code-dev-standards` for validation, cleanup, and acceptance.
- For any documentation change that affects multiple docs or repo navigation docs, use `document-consistency-editor`.

## Required Workflow

- Start from the smallest maintained subsystem that matches `REPO_MAP.md`.
- Do not create new maintained files casually. If a new maintained file is necessary, add it to `REPO_MAP.md` immediately.
- Prefer updating existing tests before adding new helper scripts.
- Remove temporary debug code, one-off migration leftovers, stale commented-out code, and redundant build artifacts before finishing.
- When changing packaging or installation behavior, verify both the code path and the produced installer/archive.

## Repository-Specific Notes

- `gemini` CLI is available on this machine and was verified with `gemini --version`.
