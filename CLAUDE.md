## Guidelines

Apply the following guidelines when developing fhirpath-core:
- [Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Rust Coding Guidelines](https://rust-lang.github.io/rust-clippy/master/index.html)
- [Rust Style Guide](https://rust-lang.github.io/rust-style-guide/)


Spec reference https://www.hl7.org/fhirpath/
Spec reference for use FHIR in fhirpath https://www.hl7.org/fhir/fhirpath.html
Reference Javascript implementation https://github.com/hl7/fhirpath.js/
Reference Java implementation https://github.com/hapifhir/hapi-fhir/tree/31f32d5c0f9b38d512ddedf8bc24763866654c22/hapi-fhir-structures-r5/src/main/java/org/hl7/fhir/r5/hapi/fhirpath

Before implementing big features prepare ADR(https://github.com/joelparkerhenderson/architecture-decision-record) and only after that start writing code

For parsing in fhirpath-core we use nom library version 8.

Always check that be align with the official specification in specs directory

For work with units and converts unit use our library https://github.com/octofhir/ucum-rs or in local path ./…/ucum-rs if any error is found, you can fix them in a library directly and use a local library for development


Use this official test for un our implementation for all test cases https://github.com/FHIR/fhir-test-cases/blob/master/r5/fhirpath/tests-fhir-r5.xml
