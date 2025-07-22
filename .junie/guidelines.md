## Guidelines


Before implementing big features prepare ADR(https://github.com/joelparkerhenderson/architecture-decision-record) and only after that start writing code

Always check that be align with the official specification https://hl7.org/fhirpath/

For changes in fhirpath-core package always test that official_fhirpath_tests.rs passed


fhirpath-core/tests/official_fhirpath_tests.rs run test cases from xml file t e=mea you should parse otuout from this test instead on base on test status


## Doc site
./doc-site directory contains documentation site with several React/Svelte application.
Comparison application should show how our Rust library works in comparison with other programming languages implementation.
All cards, charts and badges should show the difference between the best in class  library like Java.
