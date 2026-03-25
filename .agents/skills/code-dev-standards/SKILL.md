---
name: code-dev-standards
description: "enforce reusable coding standards and output templates for code-related tasks. use this skill when chatgpt is asked to write code, modify code, generate scripts, refactor code, or write tests. apply the user's preferred engineering workflow as a soft constraint: read project documentation first, prefer test-first validation in test_code, consider performance during design and implementation, clean temporary or redundant artifacts after finishing, protect secrets, use utf-8, keep code maintainable, and present implementation with a consistent template that includes approach, code, tests, cleanup notes, and usage notes when helpful."
---

# Code development standards

Apply these rules only for code-related tasks such as writing code, modifying code, generating scripts, refactoring, or writing tests.

Treat all requirements in this skill as defaults and soft constraints. Follow them unless the user explicitly asks to override part of them.

## Default workflow

1. Understand the request and infer the smallest useful change.
2. If the task touches an existing project, first inspect `README.md` and nearby docs before proposing edits.
3. During design and implementation, actively consider performance:
   - choose a simple solution first, but avoid obvious inefficiencies
   - reduce unnecessary loops, repeated I/O, repeated parsing, and duplicated computation
   - prefer appropriate data structures and clear complexity tradeoffs
   - mention performance-sensitive assumptions when they affect the design
4. Prefer a test-first path for new logic:
   - add or sketch validation in `test_code/`
   - verify behavior there first
   - then move proven code into the main codebase
5. After code changes, check whether documentation or README text should be updated to match the implementation.
6. Before finalizing, run a self-check checklist and reflect the result in the response when useful:
   - performance: check for repeated expensive work, unnecessary scans, nested loops, blocking I/O, large copies, and avoidable allocations
   - cleanup: remove temporary debug code, throwaway scripts, commented-out code, stale helpers, dead branches, and unused variables/imports
   - maintainability: reduce duplication, large functions, deep nesting, and unclear naming
   - security: verify no secrets, hard-coded credentials, or unsafe defaults were introduced
   - validation: confirm tests or validation steps cover the changed behavior
   - docs: check whether README or nearby docs should be updated
7. Do not keep temporary verification files unless the user asked to preserve them.
8. When relevant, remind the user to `git pull` before starting and to commit validated changes after finishing.

## Output template

Use this structure by default, but compress it for very small tasks.

### For implementation tasks

```markdown
## 思路
- 用 2 到 4 句说明实现方案、边界、关键取舍，以及主要性能考虑。

## 代码
```language
# production-ready code
```

## 测试
```language
# focused tests or validation script
```

## 清理与优化
- 说明已清理的临时代码、冗余逻辑、未使用项，或说明无需清理。
- 如果有性能敏感点，说明已经做的优化或暂不优化的原因。

## Self-check checklist
- 性能：已检查的热点、复杂度、I/O、内存或重复计算问题。
- 清理：已移除或确认不存在的临时代码、废代码、未使用项。
- 安全：已确认未引入密钥、敏感信息或危险默认值。
- 测试：已执行或建议执行的验证步骤。
- 文档：已更新或确认无需更新 README / 注释 / 使用说明。

## 使用说明
- 说明如何运行、依赖、环境变量、输入输出。
```

### For refactor tasks

```markdown
## 重构目标
- 说明当前问题、目标，以及性能或维护性收益。

## 重构后的代码
```language
# refactored code
```

## 验证方式
- 给出测试点、回归检查点、兼容性注意事项。

## 清理与优化
- 说明移除了哪些残余和冗余。
- 标出复杂度、I/O、内存或热点路径上的改进点。

## Self-check checklist
- 性能 / 清理 / 安全 / 验证 / 文档 五项分别给出一句结论。
```

### For small fixes

Use a shorter format:

```markdown
## 修改点
- 1 到 3 条，包含必要的性能或清理说明。

## 代码
```language
...
```

## 验证
- 1 到 3 条

## Self-check checklist
- 至少用 3 条简短结论覆盖性能、清理、验证；必要时补充安全或文档。
```

## Coding rules

Follow the detailed standards in `references/coding_rules.md`.

When a task involves algorithm selection, interview-style problem solving, or performance-sensitive logic with classic patterns such as linked lists, stacks, queues, hash maps, heaps, trees, tries, union-find, two pointers, sliding windows, binary search, dynamic programming, greedy, backtracking, BFS, DFS, Dijkstra, or topological sort, also consult `references/data_structures_algorithms.md`. Use it to choose an appropriate approach, explain why it fits, and mention the rough complexity when helpful.

When a task is specifically in Python, Go, or Rust, also consult `references/language_notes_python_go_rust.md`. Use the matching language section to choose idiomatic containers, error-handling style, performance habits, cleanup expectations, and common pitfalls for that language.

When a task is an algorithm problem or classic coding pattern in Python, Go, or Rust, also consult `references/problem_type_mapping_python_go_rust.md`. Use it to map arrays, strings, linked lists, stacks, queues, trees, graphs, dynamic programming, greedy, and shortest-path style problems to idiomatic language-specific containers and implementation patterns.

When the user wants a reusable script, CLI, service, parser, module skeleton, or small project in Python, Go, or Rust, also consult `references/engineering_templates_python_go_rust.md`. Use it to propose a lightweight project layout, place core logic in the right files, include focused tests, and preserve the existing repo structure when the project already has one.

The highest-priority rules are:

- prefer readable code over clever code
- keep responsibilities small and isolated
- avoid duplication and hard-coded secrets
- use utf-8 for source and documentation files
- use 4 spaces instead of tabs
- use `PascalCase` for classes and `snake_case` for functions and variables unless the language ecosystem strongly requires another convention
- keep functions small, with fewer than 5 parameters when practical
- add comments for design intent, risks, and non-obvious tradeoffs, not for obvious line-by-line narration
- avoid obvious performance waste in hot paths, loops, I/O, allocations, or repeated computation
- remove temporary, dead, duplicate, or redundant code before finalizing

## Safety, performance, and maintainability checks

Before finalizing code, run this default self-check checklist and check for these issues:

- hidden credentials or secrets
- duplicated logic that should be extracted
- overly large functions or classes
- unnecessary boolean flags controlling many branches
- deep nesting that should be simplified
- mismatch between code behavior and docs
- repeated expensive operations that can be cached, batched, streamed, or computed once
- avoidable full scans, nested loops, blocking I/O, or unnecessary memory copies in likely hot paths
- unused imports, stale helpers, dead branches, debug prints, or commented-out code

## Repository hygiene reminders

When the task involves repository changes, do not suggest committing these paths unless the user explicitly wants otherwise:

- `test_code/`
- `.env`
- `.claude`
- `outputs/`
- `data/vectorstore/`

## Response expectation for self-check

For medium or large code tasks, include a short `Self-check checklist` section in the final answer by default.

For very small fixes, compress it into one line, but still mention whether performance, cleanup, and validation were checked.

## User override rule

If the user explicitly requests a different structure, style, naming convention, workflow, or optimization tradeoff, follow the user's instruction and treat this skill as guidance rather than a blocker.
