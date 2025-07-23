## Guidelines

Apply the following guidelines when developing fhirpath-core:
- [Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)

For parsing in fhirpath-core we use nom library version 8.

Before implementing big features prepare ADR(https://github.com/joelparkerhenderson/architecture-decision-record) and only after that start writing code

Always check that be align with the official specification in specs directory

{{/*For changes in fhirpath-core package always test that official_fhirpath_tests.rs passed*/}}


fhirpath-core/tests/official_fhirpath_tests.rs run test cases from xml file t e=mea you should parse output from this test instead on base on test status

For work with units and converts unit use iur library https://github.com/octofhir/ucum-rs

## Doc site
./doc-site directory contains a documentation site with several React/Svelte applications.
Comparison application should show how our Rust library works in comparison with other programming languages implementation.
All cards, charts and badges should show the difference between the best in a class library like Java.
