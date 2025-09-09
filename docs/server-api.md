# FHIRPath Lab HTTP API

The server is available through the CLI:

```
octofhir-fhirpath server --port 8080
```

## Endpoints

| Method | Path           | Description |
| ------ | -------------- | ----------- |
| GET    | `/health`      | Health check |
| GET    | `/version`     | Server version info |
| POST   | `/` or `/$fhirpath` | Evaluate FHIRPath against supplied resource (auto-detects FHIR version) |
| POST   | `/r4`          | Evaluate expression assuming FHIR R4 |

Requests use the [FHIR "Parameters" resource](https://hl7.org/fhir/parameters.html) format
as defined by the FHIRPath‑Lab project.

### Example

```bash
curl -s \
  -H 'Content-Type: application/fhir+json' \
  -d '{"resourceType":"Parameters","parameter":[{"name":"expression","valueString":"Patient.name.given"},{"name":"resource","resource":{"resourceType":"Patient","name":[{"given":["John"]}]}}]}' \
  http://localhost:8080/r4 | jq
```

### Diagnostics example

```bash
curl -s \
  -H 'Content-Type: application/fhir+json' \
  -d '{"resourceType":"Parameters","parameter":[{"name":"expression","valueString":"Patient.unknown"},{"name":"resource","resource":{"resourceType":"Patient"}}]}' \
  http://localhost:8080/r4 | jq
```
