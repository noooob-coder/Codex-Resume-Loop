# Common data structures and algorithm patterns

Use this reference when a coding task involves algorithm selection, interview-style problems, performance-sensitive logic, or explaining why one approach is better than another.

## How to use this reference

When the user asks for code involving a known pattern, do not jump straight to code. First decide:

1. What are the input size and performance constraints?
2. What access pattern dominates: sequential scan, random lookup, updates in the middle, ordering, prefix/range query, shortest path, optimization over subproblems, or local-choice optimization?
3. What is the simplest structure or algorithm that meets the need?
4. Can you explain the tradeoff in one or two sentences?

Prefer a short explanation of why the chosen pattern fits.

## Selection quick map

- Frequent membership lookup / deduplication -> hash set or hash map
- Ordered retrieval / min-max / sorted iteration -> balanced tree, heap, or sorted list depending on update frequency
- FIFO processing -> queue
- LIFO backtracking / parsing / monotonic processing -> stack
- Many front insertions/removals with pointer operations -> linked list
- Prefix queries -> trie
- Interval aggregation / range query updates -> prefix sums, Fenwick tree, segment tree, sparse table
- Repeated optimal substructure -> dynamic programming
- Local best choice with provable structure -> greedy
- Relationship traversal -> graph BFS / DFS / Dijkstra / topological sort / union-find / MST
- Repeated contiguous subarray or substring scanning -> two pointers / sliding window
- Repeated string matching -> KMP / Z-function / rolling hash depending on requirements

## Data structures

### Array / dynamic array

Use when:
- index-based access is common
- iteration is dominant
- memory locality matters

Strengths:
- O(1) random access
- simple and cache-friendly

Costs:
- middle insert/delete is O(n)
- resizing can be expensive occasionally

Good fits:
- lookup by index
- DP tables
- sliding window
- prefix sums

### Linked list

Use when:
- the task truly benefits from pointer rewiring
- insert/delete at known node positions matters more than random access
- the user explicitly asks for linked list operations

Strengths:
- O(1) insert/delete after locating the node
- natural for queue implementations, LRU internals, and list-manipulation problems

Costs:
- O(n) access by position
- extra pointer memory
- worse cache locality than arrays

Common patterns:
- dummy head node to simplify edge cases
- fast/slow pointers for cycle detection or middle node
- in-place reversal with prev/curr/next
- merge two sorted linked lists

Watch-outs:
- for most business code, arrays/lists are simpler and faster unless middle rewiring is essential

### Stack

Use when:
- last-in-first-out behavior is natural
- parsing nested structures
- undo/backtracking
- monotonic stack problems

Common patterns:
- parentheses validation
- next greater element
- histogram / span problems
- DFS implemented iteratively

### Monotonic stack

Use when:
- each element needs the nearest greater/smaller value on one side
- you need to remove dominated candidates while scanning once

Common patterns:
- next greater element
- largest rectangle in histogram
- daily temperatures

### Queue / deque

Use when:
- breadth-first traversal
- FIFO scheduling
- sliding window extremes with deque

Common patterns:
- BFS shortest path in unweighted graph
- task processing pipelines
- monotonic deque for window max/min

### Monotonic deque

Use when:
- the window moves and you need max/min in O(1) amortized time
- a DP transition benefits from keeping best candidates in order

### Hash map / hash set

Use when:
- constant average-time lookup matters
- counting, indexing, grouping, or deduplication is needed

Strengths:
- average O(1) insert/find/delete

Common patterns:
- frequency count
- two-sum lookup
- visited tracking
- grouping by key

Watch-outs:
- unordered by default
- worst-case degradation exists, though uncommon in normal use

### Heap / priority queue

Use when:
- repeatedly extracting smallest/largest item
- scheduling by priority
- top-k problems

Strengths:
- O(log n) push/pop
- ideal for streaming top-k or greedy selection with best-next choice

Common patterns:
- merge k sorted lists
- Dijkstra
- task scheduling
- top-k frequency or closest points

### Tree / balanced search tree

Use when:
- ordered data and updates both matter
- range traversal or predecessor/successor is needed

Good fits:
- interval management
- ranking / ordering
- sorted maps and sets

### Binary tree / BST / balanced BST

Use when:
- recursive hierarchical structure is natural
- ordered insertion/search is part of the task

Common patterns:
- tree traversal and recursion
- lowest common ancestor
- validating BST properties

### Trie

Use when:
- prefix lookup or autocomplete is central
- dictionary-style prefix matching matters

Watch-outs:
- can be memory-heavy

### Union-find (disjoint set)

Use when:
- connectivity under merges matters
- grouping components dynamically

Common patterns:
- cycle detection in undirected graph
- Kruskal minimum spanning tree
- connected components merging

### Fenwick tree (binary indexed tree)

Use when:
- point updates and prefix/range sum queries are both needed
- coordinate-compressed frequency or inversion counting is involved

Good fits:
- dynamic prefix sums
- counting smaller elements to the right

### Segment tree

Use when:
- repeated range queries and updates both matter
- the operation is associative, such as sum/min/max/gcd

Good fits:
- range min/max/sum
- lazy propagation range updates

Watch-outs:
- heavier than prefix sums or Fenwick tree; use only when needed

### Sparse table

Use when:
- the array is static
- many idempotent range queries are needed, such as min/max/gcd

Strengths:
- O(1) query after O(n log n) preprocessing

### Graph representations

Choose based on density and access pattern:
- adjacency list -> sparse graphs, traversal, shortest path
- adjacency matrix -> dense graphs or constant-time edge existence checks
- edge list -> sorting edges, Kruskal, batch processing

## Algorithm patterns

### Two pointers

Use when:
- processing sorted arrays
- shrinking/expanding a range
- detecting pair relationships in linear time

Common patterns:
- remove duplicates from sorted array
- pair sum in sorted array
- partitioning
- fast/slow pointer on linked list

### Sliding window

Use when:
- dealing with contiguous subarray or substring problems
- maintaining a running constraint over a moving range

Common patterns:
- longest substring without repeats
- minimum window substring
- fixed-size average / max sum window

Checklist:
- what enters the window?
- what leaves the window?
- what invariant must hold?

### Prefix sums / difference arrays

Use when:
- many range-sum queries
- repeated cumulative calculations
- interval increment problems

Strengths:
- turns repeated O(n) segment sums into O(1) query after preprocessing

### Binary search

Use when:
- the search space is monotonic
- not just on arrays: also on answer space

Common patterns:
- first true / last false
- minimum feasible capacity
- threshold optimization problems

Checklist:
- define monotonic predicate clearly
- avoid infinite loops and overflow in midpoint logic

### Dynamic programming

Use when:
- the problem has overlapping subproblems and optimal substructure
- brute force recursion repeats work
- you can define a reusable state

How to reason:
1. Define state precisely.
2. Write transition.
3. Decide initialization.
4. Decide iteration order or memoized recursion.
5. Check whether state can be compressed.

Common DP families:
- linear DP: climbing stairs, house robber
- knapsack DP: choose under capacity constraints
- interval DP: merge/cut over ranges
- sequence DP: LIS, edit distance, LCS
- grid DP: path counting, min path sum
- bitmask DP: small-state combinatorial optimization
- tree DP: choose/include-exclude over subtrees
- digit DP: count numbers under prefix/limit constraints

When to avoid:
- if a greedy or direct mathematical observation solves it more simply
- if the state dimension is too large for constraints

Explain briefly:
- state meaning
- time/space complexity
- whether rolling array or memoization is used

### Greedy

Use when:
- a locally optimal choice can be shown to lead to a globally optimal answer
- the problem exposes exchange argument or staying-ahead reasoning

Common patterns:
- interval scheduling
- jump game variants
- Huffman-like merging
- choosing earliest finishing or smallest next feasible item

Watch-outs:
- if proof is unclear, prefer DP or search

### Backtracking / DFS search

Use when:
- you must enumerate combinations, permutations, or placements
- pruning can drastically reduce the search space

Common patterns:
- subsets / permutations / combinations
- N-Queens
- word search

Checklist:
- what state is mutated?
- how is it reverted?
- what pruning conditions are valid?

### BFS

Use when:
- shortest path in an unweighted graph matters
- level-order expansion is natural

Common patterns:
- minimum steps on grid
- multi-source spread
- tree level traversal

### DFS

Use when:
- reachability, components, recursion, or postorder processing matters
- state can be processed by exploring one branch fully

Common patterns:
- connected components
- cycle detection
- topological ordering by finish time

### Dijkstra

Use when:
- edge weights are non-negative
- shortest path from one or more sources is needed

Watch-outs:
- do not use with negative edges
- stale heap entries are normal; skip them safely

### Bellman-Ford / SPFA awareness

Use when:
- negative edges exist and constraints are small enough
- negative-cycle detection matters

Default note:
- mention that these are usually slower and only justified by edge conditions

### Floyd-Warshall awareness

Use when:
- all-pairs shortest path is needed
- n is small enough for O(n^3)

### Topological sort

Use when:
- dependencies form a DAG
- order respecting prerequisites is required

Common patterns:
- course schedule
- build order
- DP on DAG

### Minimum spanning tree

Use when:
- the task is to connect all nodes with minimal total edge cost

Common patterns:
- Kruskal + union-find
- Prim + heap

### KMP / Z-function / rolling hash awareness

Use when:
- repeated substring matching or border/prefix structure matters
- naive repeated matching is too slow

Choose simply:
- KMP for exact pattern matching in linear time
- Z-function for prefix-based string analysis
- rolling hash when probabilistic matching or substring hashing is acceptable

## Problem-shape hints

- Need O(1) lookup? Think hash map / hash set.
- Need best-so-far over a moving window? Think deque / heap.
- Need repeated range queries? Think prefix sums, Fenwick tree, segment tree, sparse table.
- Need choose-or-skip over sequence? Think DP.
- Need shortest unweighted path? Think BFS.
- Need shortest weighted path with non-negative edges? Think Dijkstra.
- Need merge connectivity? Think union-find.
- Need ordered scheduling by interval ends or costs? Think greedy + sorting.
- Need enumerate all feasible answers? Think backtracking with pruning.

## Complexity reminder style

When helpful, summarize the choice in one line like this:

- Data structure: hash map for O(1) average lookup.
- Algorithm: sliding window for O(n) scan with bounded extra space.
- Tradeoff: simpler than DP and avoids repeated rescans.
