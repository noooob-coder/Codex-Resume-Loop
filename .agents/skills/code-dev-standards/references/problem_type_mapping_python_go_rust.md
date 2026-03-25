# Problem-type mapping for Python / Go / Rust

Use this reference when the user asks for algorithmic code, interview-style solutions, or performance-sensitive implementation and the language is Python, Go, or Rust.

Goal:
- map common problem types to idiomatic containers and implementation patterns
- avoid fighting the language runtime or standard library
- keep explanations brief but language-aware

## How to use this reference

For each problem, decide in this order:
1. Which algorithmic pattern fits the problem shape?
2. Which built-in container or standard-library tool best supports that pattern in the target language?
3. Which language-specific pitfalls are worth mentioning?
4. What is the simplest implementation that still respects performance and cleanup?

## Arrays / strings / hash lookup

### Python
- Dynamic array: `list`
- Membership / dedupe: `set`
- Counting / grouping: `dict`, `Counter`, `defaultdict`
- Sorting-heavy tasks: sort once, avoid repeated sort in loops
- String building: accumulate pieces, then `"".join(parts)`
- Good fit for two-sum, anagram grouping, frequency stats, prefix maps

### Go
- Dynamic array: `[]T`
- Hash lookup: `map[K]V`
- String assembly: `strings.Builder` or `bytes.Buffer`
- Prefer explicit loops over over-abstracted helpers in hot paths
- Good fit for services or CLI logic that needs predictable memory and speed

### Rust
- Dynamic array: `Vec<T>`
- Hash lookup: `HashMap<K, V>`, `HashSet<T>`
- Ordered variants: `BTreeMap`, `BTreeSet`
- Prefer borrowed slices `&[T]` and `&str` in helper APIs
- Good fit when array-heavy logic is performance sensitive and should avoid hidden allocations

## Linked list problems

### Python
- Only use explicit node classes when the problem is truly about linked lists
- For general business code, `list` or `deque` is usually better
- Use dummy head and clear helper functions for reverse / merge / split logic

### Go
- Use struct nodes with pointers
- Keep pointer rewiring steps simple and local; avoid overly clever multi-assignment when it hurts readability
- For queues in production code, prefer slices or ring-buffer style structures over custom linked lists unless constant-time splicing matters

### Rust
- Linked lists are often intentionally awkward because ownership matters
- For algorithm exercises, keep node manipulation explicit and small-scope
- For production code, prefer `Vec`, `VecDeque`, or indexing unless true pointer-based behavior is required
- Be careful not to paper over ownership issues with unnecessary cloning

## Stack / queue / deque / monotonic structures

### Python
- Stack: `list.append` / `list.pop`
- Queue / deque: `collections.deque`
- Heap: `heapq`
- Monotonic stack/deque patterns are concise and idiomatic in Python

### Go
- Stack: `[]T`
- Queue: `[]T` with head index, or a small dedicated ring queue if needed
- Heap: `container/heap`
- For monotonic structures, store indexes in slices for cache-friendly scans

### Rust
- Stack: `Vec<T>`
- Queue / deque: `VecDeque<T>`
- Binary heap: `BinaryHeap<T>`
- Monotonic stacks are often easiest with `Vec<usize>` storing indexes

## Sliding window / two pointers

### Python
- Very strong default choice for string and array windows
- Use `dict`/`Counter` for counts; use `set` for uniqueness windows
- Watch out for repeated slicing inside the loop

### Go
- Good for byte slices, strings, and numeric arrays
- Prefer indexes and incremental state updates over substring copies
- Be careful with rune vs byte semantics for Unicode-heavy tasks

### Rust
- Excellent when working with slices and byte arrays
- Prefer indexes over owned substring creation
- For UTF-8 text, be explicit whether logic is byte-based or char-based

## Prefix sums / difference arrays / range queries

### Python
- Use lists for prefix sums; simple and expressive
- For many updates and queries under tight limits, note when Python may be slower and keep implementation minimal

### Go
- Prefix arrays in slices are straightforward and fast
- Good choice when the task mixes many queries with heavier input sizes

### Rust
- `Vec<i64>` or `Vec<usize>` style prefix arrays are natural
- Good fit when correctness and overflow awareness matter

## Heap / top-k / scheduling

### Python
- `heapq` for min-heap behavior
- For max-heap logic, invert keys or use tuples carefully
- Great for top-k, merge-k-sorted, task ordering

### Go
- `container/heap` is verbose but standard
- Wrap heap behavior in a small type and keep comparisons obvious
- Good when top-k and scheduling are part of a larger service or worker loop

### Rust
- `BinaryHeap` is max-heap by default; use `Reverse` for min-heap
- Keep custom ordering explicit and tested
- Good fit for Dijkstra, event scheduling, and bounded top-k

## Trees / recursion / DFS / BFS

### Python
- Clean for recursive tree logic and BFS with `deque`
- Mention recursion depth if trees or graphs can be very deep
- For very deep recursion, iterative traversal may be safer

### Go
- BFS/DFS with slices are clear and efficient
- Recursive DFS is fine for moderate depth, but watch stack depth on adversarial inputs
- Use structs for tree nodes and keep traversal helpers small

### Rust
- Tree ownership shape matters; avoid overengineering if indexing or arena-style storage is simpler
- BFS/DFS over adjacency lists with `Vec<Vec<usize>>` is often easier than pointer-heavy structures
- Prefer iterative graph traversal if borrow complexity starts obscuring the algorithm

## Dynamic programming

### Python
- Very good for explaining state transitions and memoization clearly
- Prefer list-based DP tables or `functools.lru_cache` for memo DFS
- Use rolling arrays when space matters; avoid oversized nested tables without need

### Go
- Strong option for iterative DP with predictable performance
- Preallocate DP arrays; use clear loop order and comments for state meaning
- Useful when constraints are too large for casual Python overhead

### Rust
- Great for iterative DP where ownership is simple and performance matters
- Use `Vec` and rolling buffers; prefer numeric types with explicit bounds awareness
- Keep transitions readable; do not let indexing tricks destroy clarity

## Greedy / sorting / interval problems

### Python
- Excellent for greedy + sort implementations
- Use tuple sorting and key functions, but sort once and keep scans linear afterward

### Go
- `sort.Slice` works well; keep comparator simple and deterministic
- Good choice when large input and repeated scheduling logic matter

### Rust
- `sort`, `sort_by`, `sort_unstable_by` depending on stability needs
- Great fit when sorting dominates runtime and memory behavior matters

## Graph shortest path / topology / union-find

### Python
- BFS, topo sort, union-find are concise and readable
- Dijkstra works but be mindful of `heapq` tuple churn on large inputs
- Suitable for moderate constraints and explanation-first solutions

### Go
- Strong for graph traversal and shortest path under larger constraints
- Use slices for adjacency lists and small structs for edges
- A good default when Python would be borderline on performance

### Rust
- Strong for large graph workloads when memory layout and speed matter
- Adjacency lists in `Vec<Vec<Edge>>` are usually cleanest
- Great fit for performance-sensitive shortest path or DAG DP code

## Backtracking / search / combinatorics

### Python
- Best for concise recursive search with pruning and explanation
- Use mutable path buffers and pop/backtrack cleanly
- Warn when exponential growth dominates and only pruning saves it

### Go
- Good when search tree is large enough that Python overhead starts to matter
- Keep state rollback explicit to avoid hidden allocations

### Rust
- Works well when mutable buffers and explicit ownership are kept simple
- Prefer reusable buffers over cloning full state each recursion step

## Engineering-focused defaults by language

### Python
- Best for automation scripts, data cleanup, integrations, and explanation-heavy solutions
- Default to readability, type hints on public functions, and small helper functions
- Mention when a CPU-heavy solution may want Go or Rust instead

### Go
- Best for concurrent services, CLI tools, network workers, and stable throughput
- Default to explicit error handling, context propagation, and preallocation where obvious
- Keep goroutine lifecycle and cleanup visible

### Rust
- Best for performance-critical modules, robust CLIs, parsers, and memory-sensitive components
- Default to borrow-first APIs, controlled allocation, and avoiding needless clones
- Keep cleanup focused on ownership, temporary allocations, and panic-prone code paths

## Output hint

When helpful, summarize in one line like this:
- Pattern: sliding window.
- Python mapping: `dict` + two indexes for O(n) scan.
- Go mapping: byte/rune indexes + map counts with no substring copying.
- Rust mapping: slice indexes + `HashMap` counts, borrow input where possible.
