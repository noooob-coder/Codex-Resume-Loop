# Coding rules reference

## Workflow expectations

- Before changing code in an existing repository, run `git pull` first.
- Read `README.md` and related design docs before editing.
- For new features or non-trivial logic, validate first in `test_code/`, then merge the proven code into the main code.
- After implementation, update README or docs if behavior changed.
- After validation, commit the code to git.
- Before final delivery, remove temporary or redundant artifacts unless the user explicitly asks to keep them.

## Do not commit by default

- `test_code/`
- `.env`
- `.claude`
- `outputs/`
- `data/vectorstore/`

## Default self-check checklist

Before final delivery, do a quick self-check and summarize the result when useful:

- Performance: no obvious repeated expensive work, bad hotspot complexity, wasteful I/O, or unnecessary copies remain.
- Cleanup: temporary debug code, dead branches, commented-out code, duplicate helpers, and unused imports/variables are removed.
- Security: no secrets, hard-coded credentials, or risky defaults were introduced.
- Validation: tests or validation steps cover the changed behavior.
- Documentation: README, comments, or usage notes still match the implementation.

## Encoding

- Use UTF-8 for all source and documentation files.

## Design principles

- Follow DRY.
- Encapsulate changing parts.
- Program to interfaces, not implementations.
- Prefer composition over inheritance.
- Use delegation to avoid oversized classes.
- Keep coupling low and follow the principle of least knowledge.

## SOLID

- Single responsibility principle.
- Open/closed principle.
- Liskov substitution principle.
- Interface segregation principle.
- Dependency inversion principle.

## Core coding principles

- Readability first.
- Keep style consistent across the project.
- Each class, function, and module should do one thing.
- Never hard-code secrets or sensitive information.
- Read secrets from environment variables.
- Consider performance during implementation, especially on loops, I/O, parsing, allocations, and repeated work.
- Choose optimization deliberately: keep the solution simple, but do not ignore obvious hotspots.

## Naming

- Classes: `PascalCase`
- Functions and variables: `snake_case`
- Interfaces: use `I` prefix when that convention fits the language and codebase
- Prefer clear, searchable names over abbreviations.

## Formatting

- Use 4 spaces for indentation.
- Do not use tabs.
- Put spaces around operators.
- Keep one statement or concern per line.
- Use blank lines to separate logical blocks.
- Keep brackets, quotes, and braces properly paired.

## Function and class design

- Keep functions short and focused.
- Prefer fewer than 5 parameters when practical.
- Avoid boolean parameters that control complex branching.
- Reduce nesting where possible.
- A class should have one reason to change.
- Depend on abstractions rather than low-level details.

## Performance guidelines

- Avoid repeated expensive work inside loops.
- Prefer computing once and reusing results when correctness allows.
- Use data structures that fit the access pattern.
- Avoid unnecessary full scans, copies, conversions, or blocking operations.
- Be explicit when trading memory for speed or speed for readability.
- If a path is likely hot, note the complexity or bottleneck briefly.
- Do not micro-optimize cold paths unless the user asks for it.

## Comments

- Explain why, not what.
- Add comments for key design decisions, risk points, and important performance tradeoffs.
- Do not leave commented-out dead code.
- Avoid comments that only restate the code.

## Testing and validation

- All new logic should be covered by complete and relevant tests.
- Only move code into the main codebase after validation passes.
- Use `test_code/` for temporary verification work and remove temporary test artifacts after integration if appropriate.

## Cleanup and redundancy control

- Remove temporary debug statements, throwaway scripts, unused imports, and dead branches.
- Delete obsolete helpers after replacement.
- Merge duplicate logic into shared helpers when it improves clarity.
- Avoid leaving stale compatibility layers unless there is an active migration reason.
- Clean redundant files and intermediate outputs that are no longer needed.

## MVP mindset

- Each change should solve one clear problem.
- Each increment should be runnable and verifiable.
- Prefer small, independently testable and independently deliverable units.
- Prioritize the most valuable and lowest-risk increments first.
- Define clear acceptance criteria for each increment.
