import init, {
    evaluate_fhirpath,
    validate_fhirpath,
    get_fhirpath_version
} from '../pkg/fhirpath_wasm.js';

class FHIRPathDemo {
    constructor() {
        this.wasmModule = null;
        this.isInitialized = false;
    }

    async init() {
        try {
            // Initialize the WASM module
            this.wasmModule = await init();
            this.isInitialized = true;

            // Update version info
            this.updateVersionInfo();

            // Set up event listeners
            this.setupEventListeners();

            console.log('FHIRPath WASM module initialized successfully');
        } catch (error) {
            console.error('Failed to initialize WASM module:', error);
            this.showError('Failed to initialize FHIRPath engine. Please refresh the page.');
        }
    }

    updateVersionInfo() {
        try {
            const version = get_fhirpath_version();
            const versionElement = document.getElementById('version-info');
            versionElement.textContent = `FHIRPath Spec Version: ${version}`;
        } catch (error) {
            console.error('Failed to get version info:', error);
        }
    }

    setupEventListeners() {
        // Evaluate button
        const evaluateBtn = document.getElementById('evaluate-btn');
        evaluateBtn.addEventListener('click', () => this.evaluateExpression());

        // Validate button
        const validateBtn = document.getElementById('validate-btn');
        validateBtn.addEventListener('click', () => this.validateExpression());

        // Example buttons
        const exampleButtons = document.querySelectorAll('.example-btn');
        exampleButtons.forEach(btn => {
            btn.addEventListener('click', (e) => {
                const expression = e.target.getAttribute('data-expression');
                document.getElementById('expression-input').value = expression;
                this.evaluateExpression();
            });
        });

        // Enter key support for expression input
        const expressionInput = document.getElementById('expression-input');
        expressionInput.addEventListener('keypress', (e) => {
            if (e.key === 'Enter') {
                this.evaluateExpression();
            }
        });

        // Auto-resize textarea
        const resourceInput = document.getElementById('resource-input');
        resourceInput.addEventListener('input', this.autoResizeTextarea);
    }

    autoResizeTextarea(e) {
        const textarea = e.target;
        textarea.style.height = 'auto';
        textarea.style.height = Math.min(textarea.scrollHeight, 400) + 'px';
    }

    async evaluateExpression() {
        if (!this.isInitialized) {
            this.showError('WASM module not initialized yet. Please wait...');
            return;
        }

        const expression = document.getElementById('expression-input').value.trim();
        const resourceJson = document.getElementById('resource-input').value.trim();

        if (!expression) {
            this.showError('Please enter a FHIRPath expression');
            return;
        }

        if (!resourceJson) {
            this.showError('Please enter a FHIR resource JSON');
            return;
        }

        this.showLoading(true);

        try {
            // Validate JSON first
            JSON.parse(resourceJson);

            // Evaluate the expression
            const result = evaluate_fhirpath(expression, resourceJson);
            const parsedResult = JSON.parse(result);

            if (parsedResult.error) {
                this.showError(`Evaluation Error: ${parsedResult.error}`);
            } else {
                this.showResult(JSON.stringify(parsedResult, null, 2), 'success');
            }
        } catch (error) {
            if (error instanceof SyntaxError) {
                this.showError('Invalid JSON in FHIR resource');
            } else {
                this.showError(`Unexpected error: ${error.message}`);
            }
        } finally {
            this.showLoading(false);
        }
    }

    async validateExpression() {
        if (!this.isInitialized) {
            this.showError('WASM module not initialized yet. Please wait...');
            return;
        }

        const expression = document.getElementById('expression-input').value.trim();

        if (!expression) {
            this.showError('Please enter a FHIRPath expression to validate');
            return;
        }

        this.showLoading(true);

        try {
            const result = validate_fhirpath(expression);
            const parsedResult = JSON.parse(result);

            if (parsedResult.valid) {
                this.showResult('✅ Expression is valid', 'success');
            } else {
                this.showResult(`❌ Expression is invalid: ${parsedResult.error}`, 'error');
            }
        } catch (error) {
            this.showError(`Validation error: ${error.message}`);
        } finally {
            this.showLoading(false);
        }
    }

    showResult(result, type = 'success') {
        const resultOutput = document.getElementById('result-output');
        resultOutput.textContent = result;
        resultOutput.className = type;
    }

    showError(message) {
        this.showResult(`❌ Error: ${message}`, 'error');
    }

    showLoading(show) {
        const loading = document.getElementById('loading');
        const resultOutput = document.getElementById('result-output');

        if (show) {
            loading.classList.remove('hidden');
            resultOutput.style.opacity = '0.5';
        } else {
            loading.classList.add('hidden');
            resultOutput.style.opacity = '1';
        }
    }
}

// Initialize the demo when the page loads
document.addEventListener('DOMContentLoaded', async () => {
    const demo = new FHIRPathDemo();
    await demo.init();
});

// Handle any unhandled promise rejections
window.addEventListener('unhandledrejection', (event) => {
    console.error('Unhandled promise rejection:', event.reason);
    event.preventDefault();
});

// Export for debugging
window.FHIRPathDemo = FHIRPathDemo;
