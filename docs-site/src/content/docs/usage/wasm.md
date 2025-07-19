---
title: WebAssembly Integration
description: Complete guide to using OctoFHIR FHIRPath in web browsers with WebAssembly
---

The OctoFHIR FHIRPath WebAssembly bindings provide high-performance FHIRPath evaluation directly in web browsers without requiring a server.

## Installation

### Using npm

```bash
# Install the WASM package
npm install fhirpath-wasm

# Or install from local build
cd fhirpath-wasm
npm install
npm run build
```

### Direct Usage

You can also use the WASM module directly by including the generated files in your web project:

- `fhirpath_wasm.js` - JavaScript bindings
- `fhirpath_wasm_bg.wasm` - WebAssembly binary
- `fhirpath_wasm.d.ts` - TypeScript definitions (if using TypeScript)

## Basic Usage

### Module Initialization

```javascript
import init, {
    evaluate_fhirpath,
    validate_fhirpath,
    get_fhirpath_version
} from './pkg/fhirpath_wasm.js';

// Initialize the WASM module
async function initFHIRPath() {
    try {
        await init();
        console.log('FHIRPath WASM initialized');
        console.log('Version:', get_fhirpath_version());
    } catch (error) {
        console.error('Failed to initialize WASM:', error);
    }
}

// Call initialization
initFHIRPath();
```

### Expression Evaluation

```javascript
// Sample FHIR Patient resource
const patient = {
    resourceType: "Patient",
    name: [
        {
            given: ["John", "Michael"],
            family: "Smith"
        }
    ],
    gender: "male",
    birthDate: "1990-01-15"
};

// Convert to JSON string (required by WASM interface)
const resourceJson = JSON.stringify(patient);

// Evaluate FHIRPath expression
try {
    const result = evaluate_fhirpath("Patient.name.given", resourceJson);
    const parsedResult = JSON.parse(result);
    
    if (parsedResult.error) {
        console.error('Evaluation error:', parsedResult.error);
    } else {
        console.log('Given names:', parsedResult);
        // Output: ["John", "Michael"]
    }
} catch (error) {
    console.error('Unexpected error:', error);
}
```

### Expression Validation

```javascript
// Validate FHIRPath expression syntax
try {
    const result = validate_fhirpath("Patient.name.family");
    const validation = JSON.parse(result);
    
    if (validation.valid) {
        console.log('✅ Expression is valid');
    } else {
        console.log('❌ Expression is invalid:', validation.error);
    }
} catch (error) {
    console.error('Validation error:', error);
}
```

## Advanced Usage

### Complete Web Application Example

```html
<!DOCTYPE html>
<html>
<head>
    <title>FHIRPath WASM Demo</title>
</head>
<body>
    <div>
        <h1>FHIRPath Evaluator</h1>
        <textarea id="resource" placeholder="Enter FHIR resource JSON..."></textarea>
        <input id="expression" placeholder="Enter FHIRPath expression..." />
        <button onclick="evaluateExpression()">Evaluate</button>
        <pre id="result"></pre>
    </div>

    <script type="module">
        import init, { evaluate_fhirpath, get_fhirpath_version } from './pkg/fhirpath_wasm.js';
        
        let wasmInitialized = false;
        
        async function initializeWasm() {
            try {
                await init();
                wasmInitialized = true;
                console.log('WASM initialized, version:', get_fhirpath_version());
            } catch (error) {
                console.error('WASM initialization failed:', error);
            }
        }
        
        window.evaluateExpression = function() {
            if (!wasmInitialized) {
                document.getElementById('result').textContent = 'WASM not initialized yet';
                return;
            }
            
            const resource = document.getElementById('resource').value;
            const expression = document.getElementById('expression').value;
            
            try {
                const result = evaluate_fhirpath(expression, resource);
                const parsed = JSON.parse(result);
                
                if (parsed.error) {
                    document.getElementById('result').textContent = 'Error: ' + parsed.error;
                } else {
                    document.getElementById('result').textContent = JSON.stringify(parsed, null, 2);
                }
            } catch (error) {
                document.getElementById('result').textContent = 'Error: ' + error.message;
            }
        };
        
        // Initialize on page load
        initializeWasm();
    </script>
</body>
</html>
```

### Error Handling

```javascript
function safeEvaluate(expression, resourceJson) {
    try {
        // Validate JSON first
        JSON.parse(resourceJson);
        
        // Evaluate expression
        const result = evaluate_fhirpath(expression, resourceJson);
        const parsed = JSON.parse(result);
        
        if (parsed.error) {
            return {
                success: false,
                error: parsed.error,
                type: 'evaluation'
            };
        }
        
        return {
            success: true,
            data: parsed
        };
    } catch (error) {
        if (error instanceof SyntaxError) {
            return {
                success: false,
                error: 'Invalid JSON in resource',
                type: 'json'
            };
        }
        
        return {
            success: false,
            error: error.message,
            type: 'unexpected'
        };
    }
}
```

### TypeScript Support

```typescript
// Type definitions for the WASM module
declare module 'fhirpath-wasm' {
    export default function init(): Promise<void>;
    export function evaluate_fhirpath(expression: string, resource: string): string;
    export function validate_fhirpath(expression: string): string;
    export function get_fhirpath_version(): string;
}

// Usage with types
import init, { evaluate_fhirpath, validate_fhirpath } from 'fhirpath-wasm';

interface EvaluationResult {
    success: boolean;
    data?: any;
    error?: string;
}

async function evaluateWithTypes(expression: string, resource: object): Promise<EvaluationResult> {
    try {
        const resourceJson = JSON.stringify(resource);
        const result = evaluate_fhirpath(expression, resourceJson);
        const parsed = JSON.parse(result);
        
        if (parsed.error) {
            return { success: false, error: parsed.error };
        }
        
        return { success: true, data: parsed };
    } catch (error) {
        return { success: false, error: (error as Error).message };
    }
}
```

## Common FHIRPath Expressions

Here are some commonly used FHIRPath expressions you can try:

### Patient Resource
```javascript
// Get patient's given names
"Patient.name.given"

// Get patient's family name
"Patient.name.family"

// Get patient's gender
"Patient.gender"

// Get patient's birth date
"Patient.birthDate"

// Get patient's phone numbers
"Patient.telecom.where(system='phone').value"

// Get patient's email addresses
"Patient.telecom.where(system='email').value"

// Get patient's city
"Patient.address.city"
```

### Observation Resource
```javascript
// Get observation value
"Observation.value"

// Get observation code
"Observation.code.coding.code"

// Get observation status
"Observation.status"

// Get observation date
"Observation.effectiveDateTime"
```

## Performance Considerations

- **Initialization**: WASM module initialization is asynchronous and should be done once per application
- **Memory**: Large FHIR resources may require more memory; consider chunking for very large datasets
- **Caching**: Cache frequently used expressions and resources when possible
- **Error Handling**: Always handle both WASM errors and FHIRPath evaluation errors

## Browser Compatibility

The WASM module works in all modern browsers that support:
- WebAssembly (WASM)
- ES6 Modules
- Async/Await

Supported browsers:
- Chrome 61+
- Firefox 60+
- Safari 11+
- Edge 16+

## Deployment

When deploying your application:

1. Ensure WASM files are served with correct MIME types
2. Configure your web server to serve `.wasm` files with `application/wasm` content type
3. Consider using a CDN for better performance
4. Enable gzip compression for WASM files

### Example server configuration (nginx):
```nginx
location ~* \.wasm$ {
    add_header Content-Type application/wasm;
    add_header Cache-Control "public, max-age=31536000";
}
```

## Interactive Playground

Try the [FHIRPath Playground](/playground) to experiment with expressions interactively in your browser.
