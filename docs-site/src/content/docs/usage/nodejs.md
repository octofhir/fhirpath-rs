---
title: Node.js Integration
description: Complete guide to using OctoFHIR FHIRPath in Node.js and JavaScript applications
---

The OctoFHIR FHIRPath Node.js bindings provide native performance for JavaScript and TypeScript applications.

## Installation

Install the package using npm or yarn:

```bash
# Using npm
npm install fhirpath-node

# Using yarn
yarn add fhirpath-node
```

## Basic Usage

### CommonJS (require)

```javascript
const { evaluateExpression, validateExpression } = require('fhirpath-node');

const patient = {
    resourceType: "Patient",
    name: [
        {
            given: ["John", "Michael"],
            family: "Smith"
        }
    ],
    gender: "male"
};

// Evaluate FHIRPath expression
const result = evaluateExpression("name.given", patient);
console.log("Given names:", result);

// Validate expression syntax
const isValid = validateExpression("name.family");
console.log("Expression is valid:", isValid);
```

### ES Modules (import)

```javascript
import { evaluateExpression, validateExpression } from 'fhirpath-node';

const patient = {
    resourceType: "Patient",
    name: [{ given: ["Jane"], family: "Doe" }]
};

const familyName = evaluateExpression("name.family", patient);
console.log("Family name:", familyName);
```

### TypeScript

```typescript
import { evaluateExpression, validateExpression, FhirPathResult } from 'fhirpath-node';

interface Patient {
    resourceType: "Patient";
    name?: Array<{
        given?: string[];
        family?: string;
        use?: string;
    }>;
    gender?: string;
}

const patient: Patient = {
    resourceType: "Patient",
    name: [
        {
            given: ["Alice"],
            family: "Johnson",
            use: "official"
        }
    ],
    gender: "female"
};

const result: FhirPathResult = evaluateExpression("name.given", patient);
console.log("Result:", result);
```

## Advanced Usage

### Error Handling

```javascript
const { evaluateExpression } = require('fhirpath-node');

function safeEvaluate(expression, resource) {
    try {
        const result = evaluateExpression(expression, resource);
        return { success: true, data: result };
    } catch (error) {
        return { 
            success: false, 
            error: error.message,
            type: error.name
        };
    }
}

const patient = { resourceType: "Patient" };

// Valid expression
const result1 = safeEvaluate("resourceType", patient);
console.log("Valid result:", result1);

// Invalid expression
const result2 = safeEvaluate("invalid..syntax", patient);
console.log("Invalid result:", result2);
```

### Working with Complex Resources

```javascript
const { evaluateExpression } = require('fhirpath-node');

const bundle = {
    resourceType: "Bundle",
    entry: [
        {
            resource: {
                resourceType: "Patient",
                name: [{ given: ["John"] }]
            }
        },
        {
            resource: {
                resourceType: "Observation",
                valueQuantity: {
                    value: 120,
                    unit: "mmHg"
                }
            }
        }
    ]
};

// Extract all resource types
const resourceTypes = evaluateExpression("entry.resource.resourceType", bundle);
console.log("Resource types:", resourceTypes);

// Extract patient names
const patientNames = evaluateExpression(
    "entry.resource.where(resourceType = 'Patient').name.given",
    bundle
);
console.log("Patient names:", patientNames);

// Extract observation values
const obsValues = evaluateExpression(
    "entry.resource.where(resourceType = 'Observation').valueQuantity.value",
    bundle
);
console.log("Observation values:", obsValues);
```

### Async/Await Pattern

```javascript
const { evaluateExpression } = require('fhirpath-node');

async function processPatient(patient) {
    try {
        // FHIRPath evaluation is synchronous, but you can wrap it for async workflows
        const name = await Promise.resolve(evaluateExpression("name.given", patient));
        const gender = await Promise.resolve(evaluateExpression("gender", patient));
        
        return {
            name: name[0],
            gender: gender[0]
        };
    } catch (error) {
        throw new Error(`Failed to process patient: ${error.message}`);
    }
}

// Usage
const patient = {
    resourceType: "Patient",
    name: [{ given: ["Sarah"] }],
    gender: "female"
};

processPatient(patient)
    .then(result => console.log("Processed:", result))
    .catch(error => console.error("Error:", error));
```

## Integration Patterns

### Express.js API

```javascript
const express = require('express');
const { evaluateExpression, validateExpression } = require('fhirpath-node');

const app = express();
app.use(express.json());

// Evaluate FHIRPath expression endpoint
app.post('/fhirpath/evaluate', (req, res) => {
    const { expression, resource } = req.body;
    
    if (!expression || !resource) {
        return res.status(400).json({
            error: 'Both expression and resource are required'
        });
    }
    
    try {
        const result = evaluateExpression(expression, resource);
        res.json({ result });
    } catch (error) {
        res.status(400).json({
            error: error.message,
            type: error.name
        });
    }
});

// Validate FHIRPath expression endpoint
app.post('/fhirpath/validate', (req, res) => {
    const { expression } = req.body;
    
    if (!expression) {
        return res.status(400).json({
            error: 'Expression is required'
        });
    }
    
    try {
        const isValid = validateExpression(expression);
        res.json({ valid: isValid });
    } catch (error) {
        res.json({ 
            valid: false, 
            error: error.message 
        });
    }
});

app.listen(3000, () => {
    console.log('FHIRPath API server running on port 3000');
});
```

### React Component

```jsx
import React, { useState } from 'react';
import { evaluateExpression } from 'fhirpath-node';

function FhirPathEvaluator() {
    const [expression, setExpression] = useState('');
    const [resource, setResource] = useState('');
    const [result, setResult] = useState(null);
    const [error, setError] = useState(null);

    const handleEvaluate = () => {
        try {
            const parsedResource = JSON.parse(resource);
            const evalResult = evaluateExpression(expression, parsedResource);
            setResult(evalResult);
            setError(null);
        } catch (err) {
            setError(err.message);
            setResult(null);
        }
    };

    return (
        <div>
            <h2>FHIRPath Evaluator</h2>
            <div>
                <label>Expression:</label>
                <input
                    type="text"
                    value={expression}
                    onChange={(e) => setExpression(e.target.value)}
                    placeholder="e.g., name.given"
                />
            </div>
            <div>
                <label>FHIR Resource (JSON):</label>
                <textarea
                    value={resource}
                    onChange={(e) => setResource(e.target.value)}
                    placeholder='{"resourceType": "Patient", ...}'
                />
            </div>
            <button onClick={handleEvaluate}>Evaluate</button>
            
            {result && (
                <div>
                    <h3>Result:</h3>
                    <pre>{JSON.stringify(result, null, 2)}</pre>
                </div>
            )}
            
            {error && (
                <div style={{ color: 'red' }}>
                    <h3>Error:</h3>
                    <p>{error}</p>
                </div>
            )}
        </div>
    );
}

export default FhirPathEvaluator;
```

### Batch Processing

```javascript
const { evaluateExpression } = require('fhirpath-node');

class FhirPathProcessor {
    constructor() {
        this.results = [];
    }

    processResources(resources, expression) {
        return resources.map((resource, index) => {
            try {
                const result = evaluateExpression(expression, resource);
                return {
                    index,
                    success: true,
                    result
                };
            } catch (error) {
                return {
                    index,
                    success: false,
                    error: error.message
                };
            }
        });
    }

    async processResourcesAsync(resources, expression) {
        const promises = resources.map(async (resource, index) => {
            return new Promise((resolve) => {
                try {
                    const result = evaluateExpression(expression, resource);
                    resolve({ index, success: true, result });
                } catch (error) {
                    resolve({ index, success: false, error: error.message });
                }
            });
        });

        return Promise.all(promises);
    }
}

// Usage
const processor = new FhirPathProcessor();
const patients = [
    { resourceType: "Patient", name: [{ given: ["John"] }] },
    { resourceType: "Patient", name: [{ given: ["Jane"] }] }
];

const results = processor.processResources(patients, "name.given");
console.log("Batch results:", results);
```

## Testing

### Jest Tests

```javascript
const { evaluateExpression, validateExpression } = require('fhirpath-node');

describe('FHIRPath Node.js Integration', () => {
    const samplePatient = {
        resourceType: "Patient",
        name: [
            {
                given: ["John"],
                family: "Doe"
            }
        ],
        gender: "male"
    };

    test('should evaluate simple expressions', () => {
        const result = evaluateExpression("name.given", samplePatient);
        expect(result).toEqual(["John"]);
    });

    test('should validate expressions', () => {
        expect(validateExpression("name.given")).toBe(true);
        expect(validateExpression("invalid..syntax")).toBe(false);
    });

    test('should handle errors gracefully', () => {
        expect(() => {
            evaluateExpression("invalid..syntax", samplePatient);
        }).toThrow();
    });
});
```

## Performance Tips

1. **Reuse expressions**: If evaluating the same expression multiple times, consider caching the validation result
2. **Batch processing**: Process multiple resources in batches for better throughput
3. **Error handling**: Always wrap evaluations in try-catch blocks
4. **Memory management**: For large datasets, process resources in chunks

## Common Patterns

### Configuration-driven evaluation

```javascript
const { evaluateExpression } = require('fhirpath-node');

const extractionConfig = {
    patientName: "name.given[0]",
    patientFamily: "name.family[0]",
    patientGender: "gender",
    patientBirthDate: "birthDate"
};

function extractPatientData(patient) {
    const extracted = {};
    
    for (const [key, expression] of Object.entries(extractionConfig)) {
        try {
            const result = evaluateExpression(expression, patient);
            extracted[key] = result.length === 1 ? result[0] : result;
        } catch (error) {
            extracted[key] = null;
        }
    }
    
    return extracted;
}
```

## Next Steps

- Explore more [usage examples](/examples/usage-examples/)
- Learn about [CLI usage](/usage/cli/)
- Check out [Rust integration](/usage/rust/)
- Read the [API reference](/reference/api/)
