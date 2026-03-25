# Gemini Handoff Examples

## New page example

```text
You are the primary UI and frontend implementer for this task.

Goal:
Create a pricing page for the existing SaaS app.

Product and interface summary:
- users: prospective customers comparing plans
- surfaces to implement: pricing page with plan cards, faq section, and sticky mobile cta
- required features:
  - monthly and annual plan presentation
  - feature comparison bullets
  - faq section
  - clear cta entry points
- required states:
  - responsive mobile and desktop layouts
  - hover and focus states for plan cards and buttons
  - empty-safe rendering if faq content is absent

Read these files first:
- app/layout.tsx
- app/page.tsx
- components/ui/*
- tailwind.config.ts
- app/globals.css

Constraints:
- match existing spacing, typography, and card patterns
- reuse current button, badge, and container components where possible
- keep the page responsive from mobile to desktop
- maintain accessible heading order and button labels

Task:
Inspect the listed code, then directly implement the pricing page in code. Do not stop at a plan. Edit the relevant files and produce the final code changes.

Deliverable:
- updated code
- brief summary of changed files
- any assumptions that materially affected implementation
```

## Example Gemini CLI launch

### macOS or Linux

```bash
gemini --yolo -p "$(cat /tmp/gemini-ui-handoff.txt)"
```

### Windows PowerShell

```powershell
$getPrompt = Get-Content -Raw .\gemini-ui-handoff.txt
gemini --yolo -p $getPrompt
```

## Review request example for Codex

```text
Gemini has completed the implementation. Review the changed files only for:
- scope adherence
- consistency with nearby patterns
- required pages, features, and states
- accessibility basics
- responsive behavior risks
- maintainability and obvious regressions

Respond with one of:
- accepted
- accepted with follow-up suggestions
- changes required

If changes are required, provide a short actionable review note that can be sent back to Gemini.
```
