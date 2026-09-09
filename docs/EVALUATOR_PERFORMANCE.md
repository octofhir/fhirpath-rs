# FHIRPath performance release notes

Measured 2026-09-09 on aarch64-apple-darwin, Rust 1.97.1, release builds.
Baseline: fdf70795f76f872c2cc429ca9b16a712c4f6c7b0 (0.4.54).
Version remains 0.4.54: the manual release workflow performs the next patch bump.
Publication is left to the release workflow; no remote changes are made here.

## Coordinated follow-up batch (in progress)

The results below describe the earlier evaluator optimization commit. Subsequent
unreleased work now connects the existing sibling fhir-model and fhirschema
repositories through a prepared validation API:

- fhir-model exposes opaque ExecutableExpression, ConstraintId, PreparedResource
  and resource-local NodeId handles. Existing evaluators retain a default
  string/shared-context compatibility path.
- FHIRPath prepares the root and variable environment once per validation run.
  Node groups share FhirNode containers and execute retained plans. Foreign engine
  handles, foreign JSON allocations and changed variable Arc identities are
  rejected. Typed primitive roots and child nodes retain the legacy conversion
  semantics; each constraint gets an independent variable scope.
- fhirschema retains up to 4096 compiled expressions in a coalescing cache. Each
  validation run retains its handles locally, avoiding repeated global-cache
  lookups for every node. Successful booleans are memoized by node, constraint
  identity and declared type; diagnostics retain their original multiplicity.
- Concurrent schema misses coalesce. Schema-set invalidation rejects both stale
  successes and stale errors, including waiters arriving after invalidation.
  Nested compilations do not await each other's root cache entries. Inline
  expansion is limited to 128 element levels; finite nested BackboneElement
  overlays remain supported.

Normal evaluation now uses the stack-machine executor described below. As of
2026-09-10, the coordinated dependency releases are published: fhir-model 0.1.17,
canonical-manager 0.2.3 and FHIRSchema 0.3.29. Consumer minimums require those
versions and sibling path overrides have been removed. FHIRPath's own version
remains 0.4.54; the release workflow performs the next bump. The cache invalidation
follow-up below is implemented. Longer stable-host HTTP qualification remains a
separate performance acceptance check, not a claim implied by dependency updates.

The new Divan `prepared_constraints` benchmark compares both model-trait paths
using the same engine, three invariants per node and a fresh resource per
iteration. Source cloning and native preparation/indexing are timed; engine
construction and expression compilation are outside the timer. This measures
node-level validation, not full HTTP throughput.

Two sequential Divan runs, each 100 samples x 5 iterations, no concurrent builds:

| Nodes | Legacy batch API median | Prepared API median | Approx. speedup |
| --- | ---: | ---: | ---: |
| 10 | 83.72–85.48 us | 49.27–49.51 us | 1.7x |
| 100 | 782.5–787.8 us | 438.1–440.5 us | 1.8x |
| 1,000 | 12.96–12.99 ms | 4.287–4.330 ms | 3.0x |

At 1,000 nodes, the first long run's median allocated bytes fall from 117.1 MB
to 26.75 MB per validation (about 77% less allocation traffic). These are
allocations, **not peak RSS**. Native preparation adds request-local O(N) node
indexes. Longer runs exercise the bounded root cache's eviction behavior; the
initial 30-iteration check gave 7.945 ms vs 4.359 ms, so short and sustained
measurements must not be conflated. Both paths here use the same new engine,
including the constraint-scope isolation fix.

The server was also rebuilt from the three existing sibling repositories and
tested with PostgreSQL 18 in Docker. Read-only HTTP $validate, 100 Patient.contact
nodes, 8 VUs x 15 seconds, forward/reverse pairs: 1584–1595 -> 4768–4829 RPS;
p99 17.15–18.31 -> 8.67–9.22 ms. Zero HTTP errors and all result checks passed,
including rejection of an invalid contact by pat-1. These numbers compare the
previous saved evaluator-optimized binary with this coordinated integration,
not with the older baseline used in the tables below.

Local verification: FHIRPath 666 tests plus 1176/1176 compliance, fhir-model 23
tests, fhirschema 142 tests; strict Clippy passed in all three repositories.
These checks used the then-local coordinated dependencies. Registry-only release
verification is recorded separately below.

## Stack-machine follow-up: first pass

Normal compiled evaluation no longer falls back to recursive AST execution.
Context-sensitive expressions, lazy arguments, operators and provider boundaries
use explicit frames. Async/custom functions retain their callback API: a
wake-driven child-task queue supports concurrent callbacks (including join/select)
and cancellation without recursively polling a chain of evaluator futures.
Compilation traverses the general program iteratively and shares the original
owned AST between fast and general regions, without cloning deep subtrees.

Callback-free predicates run on a local inline frame stack without a child-task
channel or queue locks. Classification follows bound implementations, not function
names; custom registry overrides remain effective. Literal regex arguments are
prepared inside lambdas too. The zero-argument exists specialization is an
explicit opt-in from its lazy implementation. Heavy provider/VM futures are
allocated only at their execution boundary, keeping pure-plan futures small.

Boundaries: metadata-aware evaluation retains the reference AST implementation.
The borrowed evaluate_ast API still clones its input tree; retained compiled
handles avoid that cost. Parsing and destruction of the public Box-based AST are
not made arbitrarily deep-stack-safe by this executor. Cooperative dispatch
budgets do not preempt CPU work inside a single builtin/custom function.

Preserved pre-VM binaries vs the first VM candidate, sequential forward/reverse
Divan runs with 200 samples x 20 iterations, no concurrent builds:

| Warm expression | Before VM median | Current median |
| --- | ---: | ---: |
| 1 + 2 | 279.6 ns | 275.4–276.5 ns |
| {}.exists() | 444.9–455.4 ns | 267.1 ns |
| Literal matches | 379.9 ns | 375.8–380.0 ns |
| Literal matchesFull | 512.4–513.4 ns | 509.3–510.3 ns |
| Literal replaceMatches | 517.4–525.7 ns | 519.4–520.5 ns |
| item.where(active).id | 58.67–59.70 us | 53.63–53.98 us |
| item.where(id.matches('^item-[0-9]+$')).count() | 90.55–91.40 us | 75.42–75.44 us |

In the forward run, where(active) allocation traffic falls from about 343 KB to
203 KB per evaluation; where(regex) from about 543 KB to 353 KB. These are Divan
allocation measurements, not peak RSS. The 1,000-node prepared validation
benchmark remains effectively unchanged: 4.305–4.362 ms before vs 4.285–4.370 ms
after (100 samples x 5 iterations, forward/reverse runs). Its earlier cross-library
speedup remains; the VM change does not add a further material gain there.

First-pass workspace verification: 678 passed, 7 ignored; strict all-features/all-targets
Clippy passed. Eight VM tests cover reference parity, 5,000 binary nodes (including
inline callbacks), 1,024 nested lazy calls, custom overrides, join/select,
provider cancellation and dynamically constructed ASTs.

Specification verification is the actual `just test-coverage` recipe:
**1176/1176, 100.0%**. Its runner now exits unsuccessfully on failures, errors,
skips, missing/empty suites or insufficient total tests; the recipe requires at
least 1176 tests. Four gate tests include rounded-percentage and lost-test cases.
Manual CLI negative controls returned exit 1 for an intentionally wrong expected
result and for an all-passing but too-small suite. No specification fixtures or
expectations were relaxed. Temporary raw logs were subsequently removed during
the requested disk cleanup; the recorded results remain in this document.

## Contention follow-up

The first VM candidate showed a reproducible HTTP regex slowdown after a
validation workload (about 7.4–7.5k vs 8.9k RPS). This was not visible in the
single-thread microbenchmark. Raw binaries and logs were subsequently removed
during the requested disk cleanup; the measurements, including regressions, are
recorded here. Do not accept this candidate on compliance tests alone.

The follow-up replaces frame-owned Arc<Program> clones with numeric node IDs.
Scheduled tasks own their program once; inline callbacks borrow it. Inline
property names and synchronous operators are borrowed as well. Prepared regex
calls borrow argument collections and bound regexes; custom pure evaluators
retain the owned compatibility default. Clearing a reused task releases its
dynamic program.

Current sequential warm comparison (200 samples x 20 iterations; no builds):

| Expression | Pre-VM median | Current median |
| --- | ---: | ---: |
| 1 + 2 | 277.5 ns | 275.4 ns |
| {}.exists() | 442.9 ns | 271.3 ns |
| Literal matches | 382.0 ns | 325.0 ns |
| Literal matchesFull | 510.3 ns | 483.6 ns |
| Literal replaceMatches | 523.7 ns | 474.9 ns |
| where(active) | 59.34 us | 53.54 us |
| where(regex) | 90.77 us | 71.26 us |

The new shared-engine parallel_lambda benchmark compares the first VM with the
contention follow-up, not the pre-VM baseline. At eight threads, 400-sample
forward/reverse runs went from 524–554 us to 483–485 us; the final property-name
borrowing check gave 554 vs 484 us. Thirty-two-thread timings were too variable
to claim a stable gain. Divan allocation profiling remains enabled.
The 1,000-node prepared validation median was 4.234 ms (100 samples x 5
iterations), vs earlier pre-VM paired measurements of 4.305–4.362 ms; this small
difference does not establish a material further gain.

Current verification: 681 passed, 7 ignored; strict all-features/all-targets
Clippy and rustdoc passed; actual just test-coverage is still 1176/1176 (100.0%).
Eleven VM tests now also cover frame ownership, task-slot reuse, dynamic-program
release, and borrowed regex argument validation.
CI's stable Linux job and the release Linux build now run the same strict
specification gate before dependent publication jobs. Workflow YAML was syntax
checked locally; no workflow was launched.

### Remaining before the coordinated release batch

The final release binary was rebuilt and checked with PostgreSQL 18 in Docker.
In before → current → current → before phases (15s validation then 30s regex,
8 VUs), the larger regex regression did not recur. All eight runs had zero HTTP
failures and 100% result checks. Current regex throughput ranged from about -4%
to +2% vs paired controls; validation p99 was slightly higher. This does not
establish a whole-server gain or exclude smaller regressions. The full table and
binary hashes are in server-rs/k6/benchmarks/VALIDATION_RESULTS.md. Temporary raw
HTTP logs were subsequently removed during the requested disk cleanup.

- Run a longer stable-host HTTP soak during final release qualification, after
  the remaining cache work; do not substitute single-thread gains for it.

- Completed: generation-aware element-type invalidation and retained-context
  protection, described below. This is separate from fhirschema compiler-cache
  invalidation.
- Completed in published canonical-manager 0.2.3: updated lru and strictly bounded
  oldest-write-first search-cache eviction without cloning all cached results.
- Published model/schema/canonical minimums and registry-only resolution are now
  configured. Final verification uses these published dependencies, not sibling
  path overrides.

## Schema-cache invalidation follow-up (2026-09-10)

The engine's shared element-type entries carry a schema generation. Warm hits
remain lock-free: an atomic generation load and a Papaya lookup. A short
publication read lock covers only the post-I/O generation check and insertion;
invalidation takes the corresponding write lock, advances the generation and
clears entries. No lock is retained across provider I/O. A stale in-flight
success, absence or error is discarded. Each lookup uses its context's generation,
not a new generation sampled after invalidation: even an old parent TypeInfo
cannot seed the new cache. The caller must prepare a new context before retrying.

Only successful provider results (including authoritative absence) are cached.
Errors retain the existing optional-lookup fallback but disable descendants
memoization in that context, preventing a fallback-typed traversal from poisoning
later evaluations after provider recovery.

Engine-prepared contexts, sessions and model-trait resources are generation-bound.
Calling clear_element_type_cache after replacing the provider's schemas makes
existing prepared contexts invalid: callers must prepare a new context/resource.
Evaluation checks generation before and after execution, including the metadata
path; preparation also checks across asynchronous root typing. This deliberately
returns an explicit error rather than mixing old root/type/descendant caches with
new schema data. Compiled expression handles and schema-independent JSON trees
remain reusable. Caller-owned maps passed to the legacy public context
constructor remain externally managed; replace the map and contexts on reload.

Eight deterministic regressions cover stale success/absence/error with and
without a newer waiter, authoritative negative caching, provider recovery,
cancellation, invalidation during root preparation and evaluation, and retained
contexts/sessions/resources, including late lookups using stale parent metadata.
Semaphore handshakes control interleavings; no sleeps
or timing-based race assertions are used.

### Registry-only verification and measurements

Final local checks on 2026-09-10, against published model 0.1.17,
canonical-manager 0.2.3 and FHIRSchema 0.3.29:

- `just release-prep` passed, including the actual specification runner:
  **1176/1176, 100.0%**, with no skipped or weakened specification tests.
- Workspace all-features tests: **689 passed, 7 ignored**.
- Strict workspace/all-targets/all-features Clippy and rustdoc passed.
- `cargo publish -p octofhir-fhirpath --dry-run --allow-dirty --locked` passed.
  The expected existing-version warning is intentional: version 0.4.54 stays
  unchanged locally; the release workflow bumps it before the real publication.
- Both workspace and packaged-lockfile audits passed with `--deny warnings`
  against the freshly fetched RustSec database (1243 advisories). Packaged source
  and benchmarks were compared with the working tree and match the final code.
- Release workflow dry-run allows its own version changes and uses the resolved
  lockfile; the workflow's successful release commit also includes Cargo.lock.
  No remote workflow, publication, commit or push was performed in this check.

Warm cache hits now construct the key with exact capacity. Divan reports one
12-byte allocation and **no growth**, replacing one 8-byte allocation plus an
8-byte growth for `Patient.name`. The managed-cache median changed from 174.6 ns
in the initial generation-aware candidate to 139.6 ns in the final candidate
(400 samples x 50 iterations). A separate 200 x 20 run measured 140.8 ns.
This compares the two local candidates, not the earlier published release.
Final caller-owned and managed-cache medians were 141.2 vs 139.6 ns at one thread;
eight-thread results varied (3.317 vs 3.640 us in the repeat, 3.197 vs 2.999 us
in the first run), so no parallel throughput gain is claimed.

The prepared-constraint benchmark (100 samples x 5 iterations) measured 12.90 ms
for the legacy batch API vs 4.326 ms for the prepared API at 1,000 nodes; the
initial candidate's prepared median was 4.290 ms. The earlier prepared-path gain
is retained, not a new 3x improvement from the invalidation fix.
Two final warm-evaluator repeats (400 x 20) measured where(active) at
52.91-53.47 us and where(regex) at 71.05-71.06 us. The first final run had a
56.70 us where(active) median; that slowdown did not reproduce in the repeats.
All benchmark runs were sequential without concurrent compilation. These are
local microbenchmarks, not evidence of whole-server throughput or a substitute
for the longer stable-host HTTP qualification noted above.

Raw logs remain in the repository's ignored `target/release-checks` directory.
Generated debug/release binaries, package verification trees and rustdoc output
were removed after verification to reclaim disk space.

## Earlier evaluator-only batch: scope and boundaries

| Plan item | Implementation | Boundary |
| --- | --- | --- |
| Prepare a resource once per validation | Engine-local prepared-root cache; prepare_context and ValidationSession; contexts at JSON Pointer share object nodes; existing shared-context model trait benefits automatically | No fhir-model/fhirschema interface change or cross-library NodeId/ConstraintId API in this release. Arrays/scalars selected by session still use typed conversion. |
| Compact collections and scopes | Empty/One/Many storage; empty has no allocation, singleton one allocation; fixed this/index/total slots; short RwLock-protected named map | COW snapshots and public owned iteration retained; no general arena allocator |
| Semantic distinct/union | Stable hashed candidate sets with equality checks, shared by distinct/isDistinct/union/operator union; no Debug-string keys | Temporal and UCUM quantity values use semantic fallback and can remain quadratic |
| Regex reuse | Bounded shared dynamic cache with coalesced compilation; constant patterns bound to supported compiled call instructions | Complex AST fallback uses dynamic cache; no universal regex lowering inside lambda ASTs |
| Synchronous CPU core | 128 built-in implementations expose synchronous evaluation; async/custom evaluator interfaces remain compatible; 23 built-ins accept borrowed AST argument slices | Context/provider-dependent AST evaluation is still async |
| Executable plan handles and cache | compile_plan/evaluate_plan; bounded coalescing engine cache; retained handles survive eviction; explicit stack for lowered expression regions | Not a universal bytecode VM: unsupported/context-sensitive nodes fall back to existing AST evaluator. Plans reject execution by another engine. Registry identity is engine-local; schema lookups remain dynamic. |

Prepared roots retain both the original Arc<serde_json::Value> and its immutable
FhirNode tree. Pointer identity is checked against the retained strong Arc;
Arc::make_mut produces a new identity, avoiding stale data or cross-resource reuse.
The weighted cache has a 32 MiB accounting budget, not a strict RSS limit.
Large entries remain evaluable even when they cannot be retained.
Sessions must not be shared between different resources. Session constraint
expressions get separate variable scopes.

Deduplication uses FHIRPath equality, including numeric Integer/Decimal equivalence,
rather than formatting. Decimal equality is exact (the former epsilon comparison
was non-transitive). Hash collisions always use full equality checks; UCUM
conversion and partial temporal semantics are not approximated.
See the [FHIRPath specification](https://hl7.org/fhirpath/N1/index.html).

## Public API usage

```rust,ignore
let session = engine.validation_session(resource_arc).await?;
let context = session.context_at("/name/0", Some("HumanName")).await?;
let plan = engine.compile_plan("family.exists()")?;
let result = engine.evaluate_plan(&plan, &context).await?;
let constraints = session
    .evaluate_constraints("/name/0", Some("HumanName"), &["family.exists()"])
    .await?;
```

Retain the engine and plan across requests; create a validation session for each
resource. Existing string-based evaluate and model-evaluator trait APIs continue
to work. The new methods do not require downstream source changes.

## Microbenchmarks

Same Divan harness and allocation profiler on both executables. 200 samples,
10 evaluations per sample (32-thread case rounds to 224 samples). Fixtures,
runtime construction and initial expression compilation are outside timed loops.
Validation includes one initial JSON clone per request on both sides and repeats
groups against that same Arc; root fixture has 1,000 names. Warm where fixtures
contain 100 synthetic items. No provider I/O in these microbenchmarks.
Final runs were sequential without concurrent compilation.

| Case | Baseline median | Candidate median | Ratio |
| --- | ---: | ---: | ---: |
| distinct, 100 integers | 59.15 us | 11.17 us | 5.30x |
| distinct, 1,000 integers | 619.7 us | 117.3 us | 5.28x |
| distinct, 10,000 integers | 6.125 ms | 1.137 ms | 5.39x |
| 10 validation groups | 867.2 us | 231.2 us | 3.75x |
| 100 validation groups | 8.298 ms | 651.9 us | 12.73x |
| Constant matches | 14.00 us | 337.2 ns | 41.52x |
| Constant matchesFull | 14.01 us | 470.1 ns | 29.80x |
| Constant replaceMatches | 33.03 us | 496.1 ns | 66.58x |
| 1 + 2 | 472.5 ns | 263.1 ns | 1.80x |
| item.where(active).id | 104.9 us | 56.88 us | 1.84x |
| where + regex, 100 items | 1.464 ms | 89.02 us | 16.45x |
| {}.exists() | 388.2 ns | 435.4 ns | 0.89x (12% slower) |
| Shared regex, 8 threads | 63.17 us | 4.801 us | 13.16x |
| Shared regex, 32 threads | 76.82 us | 14.64 us | 5.25x |

The tiny empty-exists case regresses by 47 ns with the plan/fallback dispatch.
This remains a follow-up optimization, not an omitted result.
Singleton construction: two allocations become one; empty construction: one
becomes zero. Empty timing is below useful timer resolution: no ratio claimed.
These synthetic speedups must not be extrapolated to overall server capacity.

```sh
cargo bench -p octofhir-fhirpath --bench evaluation_hot_paths --locked -- \
  --sample-count 200 --sample-size 10
# Direct executable invocation requires --bench.
```

## Verification

- Workspace all-features tests: 662 passed, 7 ignored, zero failures, including
  library 572, CLI 60, integration 12 and doctests 18.
- Compliance runner: 1176/1176; fixes the two previous repeat/repeatAll $this
  failures instead of changing test expectations.
- Strict workspace/all-features/all-targets Clippy passed.
- Documentation built with RUSTDOCFLAGS=-D warnings; cargo package verified the
  actual publishable library successfully.
- Audit of the packaged crate's generated Cargo.lock: zero vulnerabilities and
  zero warnings. The workspace-only CLI/dev-tools advisory is described below.
- Regression coverage: COW collections, scope inheritance/isolation, regex errors,
  replacement captures, numeric and UCUM distinct, prepared-root identity and
  mutation, typed temporal focus, plan/AST parity, short-circuiting, concurrent
  compilation, eviction and foreign-engine rejection.
- CLI configuration tests disable auto-save and no longer touch user preferences.
- Crate package includes use crate-relative tests/benches paths.
- Cargo.lock updates yanked chacha20 0.10.1 to 0.10.2.
- Diagnostic table construction no longer uses tabled_derive; removes
  proc-macro-error2/proc-macro-error-attr2 and the future-incompatibility warning.
- Prepared JSON inputs preserve the existing typed/untyped model conversion
  contract; non-object inputs bypass prepared-root retention.

The earlier workspace dependency audit retained lru 0.16.4 (RUSTSEC-2026-0253) through
canonical-manager 0.2.2 -> fhirschema -> CLI/dev tools. It is outside the published
library's normal dependency graph. Fixing this chain requires a canonical-manager
release that uses lru >=0.18.2; adding a direct lru dependency to FHIRPath would
not replace the transitive 0.16 requirement. This chain is resolved by the
published canonical-manager 0.2.3 and FHIRSchema 0.3.29 dependency updates.
No advisory suppression, vendored fork or unpublished path override is included.
