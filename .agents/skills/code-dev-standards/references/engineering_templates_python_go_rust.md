# Engineering templates for Python / Go / Rust

Use this reference when the user asks for project code, scripts, CLIs, services, tests, or wants a reusable skeleton rather than only a single function.

Goal:
- provide lightweight default project shapes
- keep structure small and practical
- encourage performance checks, cleanup, tests, and documentation alignment

## Shared template rules

For all three languages:
- start with the smallest viable structure
- separate parsing / configuration / core logic / side effects
- include at least one focused test path when the task is non-trivial
- remove temporary debug code and unused dependencies before finalizing
- note how to run, test, and configure the result
- prefer standard library first unless a dependency clearly saves real work

## Python template

### Good defaults for
- automation scripts
- data processing tools
- small CLIs
- service helpers

### Suggested layout

```text
project/
├── README.md
├── requirements.txt        # only when needed
├── src/
│   └── app/
│       ├── __init__.py
│       ├── main.py
│       ├── config.py
│       ├── models.py
│       ├── services.py
│       └── utils.py
└── tests/
    └── test_main.py
```

### Implementation defaults
- entrypoint in `main.py`
- pure logic in `services.py` or domain-named modules
- config loaded from env vars or arguments, not hard-coded constants
- `logging` instead of `print`
- type hints on public functions
- context managers for files and network resources

### Testing defaults
- use `pytest` style when tests are requested
- isolate pure logic for fast unit tests
- if the task is script-heavy, add at least one smoke test or validation snippet

### Performance / cleanup defaults
- avoid repeated file reads, JSON parses, and regex compilation
- stream large inputs when possible
- remove debug prints, unused imports, one-off scripts, commented-out code

## Go template

### Good defaults for
- CLI tools
- HTTP services
- workers / consumers
- concurrent utilities

### Suggested layout

```text
project/
├── README.md
├── go.mod
├── cmd/
│   └── app/
│       └── main.go
├── internal/
│   ├── config/
│   │   └── config.go
│   ├── service/
│   │   └── service.go
│   └── transport/
│       └── http.go
└── internal/service/
    └── service_test.go
```

### Implementation defaults
- `cmd/.../main.go` only wires dependencies and starts execution
- core logic under `internal/`
- pass `context.Context` through request-scoped or cancellable work
- wrap errors with context
- keep interfaces small and justified by testing or multiple implementations

### Testing defaults
- table-driven tests for logic with multiple cases
- keep tests in `_test.go`
- for services, add one focused integration-like test only when it adds confidence

### Performance / cleanup defaults
- preallocate slices/maps when size is predictable
- use `strings.Builder` / `bytes.Buffer` for heavy string assembly
- check goroutine exit paths and resource closing
- remove unused structs, vars, logs, and experimental branches

## Rust template

### Good defaults for
- robust CLI tools
- parsers
- high-performance modules
- backend hotspots

### Suggested layout

```text
project/
├── Cargo.toml
├── README.md
├── src/
│   ├── main.rs
│   ├── lib.rs
│   ├── config.rs
│   ├── models.rs
│   └── service.rs
└── tests/
    └── integration_test.rs
```

### Implementation defaults
- keep business logic in `lib.rs` or focused modules, not packed into `main.rs`
- use `main.rs` for CLI / startup wiring only
- prefer `Result<T, E>` with meaningful errors
- prefer borrowed inputs in helper APIs when ownership is not needed
- add small structs and enums instead of loosely shaped tuples once semantics matter

### Testing defaults
- unit tests near modules with `#[cfg(test)]`
- integration tests under `tests/` when cross-module behavior matters
- for parser / CLI tasks, include at least one representative success case and one failure case

### Performance / cleanup defaults
- use `with_capacity` when size is known
- avoid unnecessary `clone()` and panic-prone `unwrap()` on external input paths
- keep temporary buffers scoped tightly
- remove unused modules, debug logging, and workaround code after the final shape is clear

## Response template guidance

When generating project code in these languages, a good default answer shape is:

```markdown
## 思路
- 说明模块划分、关键依赖、性能和清理考虑。

## 项目结构
```text
# tree or file list
```

## 代码
```language
# key files or full implementation
```

## 测试
```language
# focused tests
```

## 清理与优化
- 说明删掉了哪些临时项、做了哪些性能处理。

## Self-check checklist
- 性能：...
- 清理：...
- 安全：...
- 测试：...
- 文档：...

## 使用说明
- 运行、测试、配置方法。
```

## When to compress

Compress the template when:
- the user only wants one file
- the task is a tiny patch
- the existing repo structure is already fixed and should not be reinvented

In that case, still keep:
- approach
- code
- test/validation
- cleanup/performance note
- self-check conclusion
