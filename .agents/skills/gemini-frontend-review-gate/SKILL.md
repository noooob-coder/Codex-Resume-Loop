---
name: gemini-frontend-review-gate
description: route ui, ux, and frontend design and implementation tasks through gemini first, then use codex only for requirements synthesis, review, gap analysis, risk checks, and acceptance. use when chatgpt is asked to design or implement any user-facing surface such as pages, screens, components, flows, layouts, styling, design systems, interaction behavior, or frontend code and the workflow requires gemini to directly read the codebase, edit files itself, and run in yolo mode so codex does not pause for per-change approval before gemini acts.
---

# Gemini Frontend Review Gate

Treat Gemini as the primary UI and frontend designer and implementer. Treat Codex as the requirements synthesizer and reviewer.

## Core rule

For any UI, UX, or frontend design or implementation task, follow this order:

1. Gather the minimum context needed from the repository and user request.
2. Summarize the current product and interface requirements into a Gemini handoff package.
3. Invoke Gemini from the local repository with `--yolo` so Gemini can inspect files and directly perform the required edits without interactive approval pauses.
4. Have Gemini directly read the related code and write the UI or frontend code changes itself.
5. Only after Gemini has produced code, let Codex review the result for correctness, consistency, regressions, accessibility, maintainability, and alignment with the request.
6. If fixes are needed, prefer asking Gemini to apply them. Use Codex for review comments, minimal patch suggestions, or acceptance checks.

Do not let Codex be the first author of new UI or frontend code unless the user explicitly overrides this workflow or Gemini is unavailable and the user accepts fallback behavior.

## What counts as a UI or frontend task

Apply this skill when the task involves any of the following:

- page or screen design
- component creation or refactoring
- layout, spacing, responsive behavior, or visual hierarchy
- css, tailwind, style systems, tokens, themes, or animations
- design-system alignment
- ui copy placement tied to layout
- flows such as onboarding, checkout, settings, dashboards, tables, forms, modals, drawers, toasts, and wizards
- frontend bug fixes that require code edits in components, pages, styles, assets, or client logic
- user-visible interaction details such as hover, focus, empty, loading, validation, disabled, and error states

Do not apply this skill to backend-only, infra-only, data-only, or purely non-visual tasks.

## Required operating model

Assume the environment has access to Gemini through the local Gemini CLI. The essential requirement is that Gemini must be the one that reads the relevant source files and writes the code.

When Codex invokes Gemini for implementation, use `--yolo` by default. This is the approved operating mode for this workflow because the user's intent is to avoid per-change confirmation interrupts while Gemini edits the repository.

Interpret this as:

- Codex is responsible for preparing the request and launching Gemini.
- Gemini is authorized to directly read, create, modify, rename, and delete files that are relevant to the requested UI or frontend implementation.
- Codex should not stop to ask for approval on each Gemini CLI file operation when launching Gemini in this workflow.
- Codex still performs review after Gemini finishes.

If local policy, sandboxing, or environment configuration prevents `--yolo` from taking effect, state that constraint clearly and continue with the least interruptive mode available.

## Workflow

### Step 1: collect only the context Gemini needs

Before handing work to Gemini, gather:

- user goal
- target users or audience when relevant
- required pages, screens, surfaces, or flows
- required features per page or flow
- required states such as loading, empty, error, disabled, validation, success, and responsive variants
- affected routes, components, assets, and style files
- framework and styling stack
- constraints such as "match existing design language", "do not break api contracts", "mobile-first", or "reuse current components"
- acceptance criteria if present

Do not over-specify visuals if the repository already contains patterns Gemini can infer from the codebase.

### Step 2: build the Gemini handoff package

Create a handoff that includes:

- the objective in one sentence
- a concise product and interface summary
- the pages, surfaces, or flows that must exist after implementation
- the features each page or flow needs
- the states Gemini must cover
- files or directories Gemini must inspect first
- implementation constraints
- expected deliverable
- a reminder that Gemini should edit code directly, not only describe a plan

Use this template and fill in the placeholders.

```text
You are the primary UI and frontend implementer for this task.

Goal:
[one-sentence goal]

Product and interface summary:
- users: [user type]
- surfaces to implement: [pages/screens/components/flows]
- required features: [bullets]
- required states: [bullets]

Read these files first:
[file list]

Constraints:
- preserve existing architecture and conventions
- match the current visual language and component patterns
- keep changes scoped to the request
- prefer reusing existing utilities/components before creating new ones
- maintain accessibility and responsive behavior
[add task-specific constraints]

Task:
Inspect the listed code, then directly implement the required UI and frontend changes in code. Do not stop at a plan. Edit the relevant files and produce the final code changes.

Deliverable:
- updated code
- brief summary of changed files
- any assumptions that materially affected implementation
```

### Step 3: invoke Gemini with yolo mode

Run Gemini from the repository root or the relevant package directory. Prefer repository-aware execution so Gemini can inspect nearby files and infer conventions.

Use one of these command patterns.

#### macOS or Linux

```bash
gemini --yolo -p "$(cat /tmp/gemini-ui-handoff.txt)"
```

#### Windows PowerShell

```powershell
$getPrompt = Get-Content -Raw .\gemini-ui-handoff.txt
gemini --yolo -p $getPrompt
```

If the handoff is short, inline form is acceptable.

#### macOS or Linux inline example

```bash
gemini --yolo -p "Read app/layout.tsx, app/dashboard/page.tsx, components/ui, and app/globals.css. Implement the dashboard redesign directly in code. Reuse existing patterns, cover loading, empty, and responsive states, then summarize changed files."
```

#### Windows PowerShell inline example

```powershell
gemini --yolo -p "Read app/layout.tsx, app/dashboard/page.tsx, components/ui, and app/globals.css. Implement the dashboard redesign directly in code. Reuse existing patterns, cover loading, empty, and responsive states, then summarize changed files."
```

### Step 4: Gemini writes the code

Require Gemini to do the implementation.

If Gemini returns only a design description or plan, ask Gemini again and explicitly require code edits.

If Gemini proposes broad rewrites, narrow the scope and restate the exact target files or screens.

### Step 5: Codex reviews after Gemini

After Gemini finishes, Codex should perform a structured review covering:

- request fulfillment
- visual consistency with the surrounding codebase
- component reuse and architectural fit
- correctness and obvious regressions
- accessibility basics such as semantics, keyboard flow, labels, contrast risks, and focus states when relevant
- responsive behavior
- code quality and maintainability
- scope control and file hygiene

Codex should default to one of these outputs:

- accepted
- accepted with follow-up suggestions
- changes required

When changes are required, Codex should produce a concise review note and send the fix back to Gemini when possible, instead of taking over authorship.

## Review checklist

Use this checklist in review outputs:

- Does the UI satisfy the user request?
- Does it cover the required pages, screens, or flows?
- Does it include the required feature set for each surface?
- Does it cover loading, empty, error, validation, disabled, and responsive states where relevant?
- Does it match existing patterns in nearby files?
- Are there duplicated components or styles that should have reused existing ones?
- Are props, state, and data flow consistent with project conventions?
- Are there likely accessibility issues?
- Are there responsive layout issues?
- Are there unnecessary risky changes outside scope?
- Are naming and file placement consistent?

## Fallback behavior

If Gemini is unavailable, blocked, or not configured:

1. State that the intended workflow requires Gemini to author the UI or frontend code first.
2. Ask for or infer the Gemini integration path if possible.
3. Only proceed with Codex-authored code if the user explicitly approves the fallback.

If Gemini is available but `--yolo` is blocked by environment policy, state that clearly and continue with the closest available mode while preserving the same Gemini-first authorship rule.

## Output style for the user

When reporting progress or results:

- clearly label what Codex summarized as requirements
- clearly label the exact Gemini invocation used
- clearly label what Gemini implemented
- clearly label what Codex reviewed
- separate implementation notes from review findings
- call out any remaining assumptions or risks

## Example trigger patterns

Use this skill for requests like:

- design and build a new dashboard page
- refactor this react component to match the existing design system
- update the checkout ui to be mobile friendly
- restyle this settings screen but keep current behavior
- build the interface for this feature, but have gemini write the code and codex only review it
- summarize the ui requirements, send them to gemini, let gemini implement with yolo, then review the result
