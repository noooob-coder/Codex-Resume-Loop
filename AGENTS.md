# AGENTS.md

## Skills

### Available skills in this project

- `gemini-frontend-review-gate`
  - Description: Use Gemini as the primary UI/frontend analyst and implementer, then use Codex for summary, review, gap analysis, and acceptance.
  - File: [./.agents/skills/gemini-frontend-review-gate/SKILL.md](E:/project/run_spider/.agents/skills/gemini-frontend-review-gate/SKILL.md)

## Trigger rules

- For UI, UX, frontend discovery, component creation, page design, layout/styling refactors, dashboard/admin/settings/onboarding surfaces, or other user-facing interface work, use `gemini-frontend-review-gate` first unless the user explicitly asks to bypass Gemini.

## Local environment note

- `gemini` CLI is available on this machine and was verified with `gemini --version`.
