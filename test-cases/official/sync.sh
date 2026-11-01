#!/usr/bin/env bash
# Sync the official FHIRPath test suites from a local fhir-test-cases checkout.
#
#   test-cases/official/sync.sh <path-to-fhir-test-cases> [r5 r6 ...]
#
# For each FHIR version this copies <ver>/fhirpath/tests-fhir-<ver>.xml and its schema,
# rebuilds <ver>/input from the resources the tests reference (upstream JSON as is,
# FHIR XML through convert-fhir-xml), regenerates the per-group JSON suites, and
# re-checks every test against the XML with verify.py.
set -euo pipefail

FTC=${1:?usage: sync.sh <path-to-fhir-test-cases> [versions...]}
shift
if [ $# -eq 0 ]; then VERSIONS=(r5 r6); else VERSIONS=("$@"); fi

HERE=$(cd "$(dirname "$0")" && pwd)
ROOT=$(cd "$HERE/../.." && pwd)
cd "$ROOT"

cargo build -q -p fhirpath-dev-tools --bin convert-fhir-xml --bin convert-r5-xml-to-json
BIN="$ROOT/target/debug"
COMMIT=${FTC_COMMIT:-$(git -C "$FTC" rev-parse --short HEAD)}

for v in "${VERSIONS[@]}"; do
  src="$FTC/$v/fhirpath"
  out="$HERE/$v"
  echo "== $v (fhir-test-cases @ $COMMIT)"
  mkdir -p "$out"
  rm -f "$out"/*.json
  rm -rf "$out/input"
  mkdir -p "$out/input"
  cp "$src/tests-fhir-$v.xml" "$src/testSchema.xsd" "$out/"

  # Every input file the suite references, by its XML/JSON name.
  inputs=$(grep -o 'inputfile="[^"]*"' "$out/tests-fhir-$v.xml" | sed 's/inputfile="//; s/"$//' | sort -u)
  for f in $inputs; do
    stem="${f%.*}"
    if [ -f "$FTC/$v/$stem.json" ]; then
      cp "$FTC/$v/$stem.json" "$out/input/$stem.json"
    elif [ "$stem" = "ccda" ]; then
      # CDA is not a FHIR resource; the CDA JSON form is maintained by hand.
      cp "$ROOT/test-cases/input/ccda.json" "$out/input/ccda.json"
    elif [ -f "$FTC/$v/$stem.xml" ]; then
      "$BIN/convert-fhir-xml" --fhir-version "$v" "$FTC/$v/$stem.xml" "$out/input/$stem.json" >/dev/null
    else
      echo "missing input $f for $v" >&2
      exit 1
    fi
  done

  "$BIN/convert-r5-xml-to-json" "$out/tests-fhir-$v.xml" --out "$out" --fhir-version "$v" \
    --source "fhir-test-cases $v/fhirpath/tests-fhir-$v.xml @ $COMMIT"
done

python3 -I "$HERE/verify.py" "$HERE"
