#!/usr/bin/env python3
"""Independently verify test-cases/official/<v> JSON suites against the source XML."""
import glob
import json
import os
import sys
import xml.etree.ElementTree as ET

NS = "{http://hl7.org/fhirpath/tests}"
ATTR_MAP = {  # XML test attribute -> JSON key
    "name": "name", "description": "description", "predicate": "predicate",
    "skipStaticCheck": "skipStaticCheck", "ordered": "ordered", "mode": "mode",
    "version": "version", "versionTo": "versionTo", "reference": "reference",
    "testing": "testing", "context": "context", "contextType": "contextType",
    "disabled": "disabled",
}
BOOLS = {"predicate", "skipStaticCheck", "ordered", "disabled"}


def all_text(e):
    return "".join(e.itertext())


def main(root):
    problems = []
    total = 0
    for vdir in sorted(glob.glob(os.path.join(root, "r*"))):
        v = os.path.basename(vdir)
        xml = ET.parse(os.path.join(vdir, f"tests-fhir-{v}.xml")).getroot()
        js = {}
        for p in glob.glob(os.path.join(vdir, "*.json")):
            suite = json.load(open(p, encoding="utf-8"))
            for t in suite["tests"]:
                if t["name"] in js:
                    problems.append(f"{v}: duplicate JSON test {t['name']}")
                js[t["name"]] = (suite, t)
        seen = set()
        for g in xml.iter(NS + "group"):
            for x in g.iter(NS + "test"):
                total += 1
                name = x.get("name")
                seen.add(name)
                if name not in js:
                    problems.append(f"{v}: {name} missing in JSON")
                    continue
                suite, t = js[name]
                if suite["name"] != g.get("name"):
                    problems.append(f"{v}: {name} in wrong suite {suite['name']}")
                for a, k in ATTR_MAP.items():
                    want = x.get(a)
                    if a == "mode" and want is None:
                        want = g.get("mode")
                    if want is not None and a in BOOLS:
                        want = want == "true"
                    if t.get(k) != want:
                        problems.append(f"{v}: {name}.{k}: {t.get(k)!r} != {want!r}")
                inp = x.get("inputfile")
                want_inp = inp[:-4] + ".json" if inp and inp.endswith(".xml") else inp
                if t.get("inputfile") != want_inp:
                    problems.append(f"{v}: {name}.inputfile {t.get('inputfile')} != {want_inp}")
                if want_inp and not os.path.exists(os.path.join(vdir, "input", want_inp)):
                    problems.append(f"{v}: {name}: input file {want_inp} not present")
                e = x.find(NS + "expression")
                if t["expression"] != all_text(e).strip():
                    problems.append(f"{v}: {name}.expression {t['expression']!r} != {all_text(e).strip()!r}")
                if t.get("invalidKind") != e.get("invalid") or bool(t.get("expectError")) != (e.get("invalid") is not None):
                    problems.append(f"{v}: {name}.invalid mismatch")
                outs = [(o.get("type"), all_text(o)) for o in x.findall(NS + "output")]
                got = [(o.get("type"), o["value"]) for o in t.get("outputs", [])]
                if outs != got:
                    problems.append(f"{v}: {name}.outputs {got} != {outs}")
                if len(t["expected"]) != len(outs):
                    problems.append(f"{v}: {name}.expected length {len(t['expected'])} != {len(outs)}")
                for (ty, raw), val in zip(outs, t["expected"]):
                    raw = raw.strip()
                    ok = {
                        "boolean": lambda: val == (raw == "true"),
                        "integer": lambda: val == int(raw),
                        "decimal": lambda: abs(val - float(raw)) <= 1e-12 * max(1, abs(float(raw))),
                        "date": lambda: val == raw.lstrip("@"),
                        "dateTime": lambda: val == raw.lstrip("@"),
                        "time": lambda: val == raw.lstrip("@"),
                    }.get(ty, lambda: val == raw)()
                    if not ok:
                        problems.append(f"{v}: {name}.expected {val!r} vs {ty} {raw!r}")
                if x.find(NS + "notes") is not None and t.get("notes") != all_text(x.find(NS + "notes")):
                    problems.append(f"{v}: {name}.notes mismatch")
        for extra in set(js) - seen:
            problems.append(f"{v}: JSON has {extra} not in XML")
        print(f"{v}: {len(seen)} XML tests, {len(js)} JSON tests")
    for p in problems[:50]:
        print("PROBLEM", p)
    print(f"checked {total} tests, {len(problems)} problems")
    return 1 if problems else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1]))
