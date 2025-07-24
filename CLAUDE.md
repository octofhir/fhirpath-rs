## Guidelines

Apply the following guidelines when developing fhirpath-core:
- [Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Rust Coding Guidelines](https://rust-lang.github.io/rust-clippy/master/index.html)
- [Rust Style Guide](https://rust-lang.github.io/rust-style-guide/)


Spec reference  in `specs` folder
FHIRSchema spec - https://fhir-schema.github.io/fhir-schema/intro.html

Before implementing big features prepare ADR(https://github.com/joelparkerhenderson/architecture-decision-record) and only after that start writing code

For parsing we use nom library version 8.

For work with units and converts unit use our library https://github.com/octofhir/ucum-rs or in local path ./…/ucum-rs if any error is found, you can fix them in a library directly and use a local library for development

You can run tests without confirmation
