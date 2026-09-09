# FHIRPath performance release notes

Measured 2026-09-09 on aarch64-apple-darwin, Rust 1.97.1, release builds.
Baseline: fdf70795f76f872c2cc429ca9b16a712c4f6c7b0 (0.4.54).
Version remains 0.4.54: the manual release workflow performs the next patch bump.
Publication is left to the release workflow; no remote changes are made here.

## Original plan: implemented scope and boundaries

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

Workspace dependency audit retains lru 0.16.4 (RUSTSEC-2026-0253) through
canonical-manager 0.2.2 -> fhirschema -> CLI/dev tools. It is outside the published
library's normal dependency graph. Fixing this chain requires a canonical-manager
release that uses lru >=0.18.2; adding a direct lru dependency to FHIRPath would
not replace the transitive 0.16 requirement. No advisory suppression, vendored
fork or unpublished path override is included in the FHIRPath release.
