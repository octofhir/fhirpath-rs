//! Exercise input and output boundaries, including auto-detected NDJSON pipes.
use assert_cmd::Command;

#[test]
fn pipe_and_json_output_preserve_decimal_tokens() {
    let input = r#"{"resourceType":"Observation","valueDecimal":0.10000000000000000001}"#;
    let mut pipe = Command::new(assert_cmd::cargo::cargo_bin!("octofhir-fhirpath"));
    pipe.args(["--quiet", "evaluate", "value"])
        .write_stdin(format!("{input}\n"));
    pipe.assert().success().stdout("0.10000000000000000001\n");
    let mut json = Command::new(assert_cmd::cargo::cargo_bin!("octofhir-fhirpath"));
    json.args([
        "--quiet",
        "--output-format",
        "json",
        "evaluate",
        "value",
        "--input",
        input,
    ]);
    json.assert()
        .success()
        .stdout(predicates::str::contains("0.10000000000000000001"));
}
