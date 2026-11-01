# Official FHIRPath test suites

Unmodified copies of the HL7 test suites from
[fhir-test-cases](https://github.com/FHIR/fhir-test-cases), converted to the JSON
suite format the dev-tools runners read. The `source` field of every suite records the
upstream file and commit.

```
official/
  r5/  tests-fhir-r5.xml  testSchema.xsd  <group>.json ...  input/*.json
  r6/  tests-fhir-r6.xml  testSchema.xsd  <group>.json ...  input/*.json
```

- One JSON suite per XML `<group>`. Every attribute and element the test schema
  defines is carried over (`mode`, `context`, `contextType`, `ordered`, `testing`,
  `version`, ...), and `outputs` keeps each output's text as written.
- `input/` holds the resources the tests reference: upstream JSON as is, FHIR XML
  converted by `convert-fhir-xml` against that version's schemas. `ccda.json` is the
  hand-maintained CDA form from `test-cases/input`.

## Running

```
just test-official-r5    # report: TEST_COVERAGE_OFFICIAL_R5.md
just test-official-r6    # report: TEST_COVERAGE_OFFICIAL_R6.md
```

These runs are report-only. `just test-coverage` (the CI gate) does not include them.
Groups with `mode="custom"` (Java RI extension functions such as `documentFunctions`)
are skipped.

## Updating

```
git clone --depth 1 https://github.com/FHIR/fhir-test-cases.git /tmp/fhir-test-cases
just sync-official-tests /tmp/fhir-test-cases
```

`sync.sh` regenerates everything here and then runs `verify.py`. `verify.py` is an
independent check: it reads each `<test>` in the XML and compares it with the JSON
(attributes, expression, outputs), and it confirms that every referenced input file
exists.
