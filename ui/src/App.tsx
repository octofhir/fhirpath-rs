import { Component, createSignal } from 'solid-js';
import styles from './App.module.css';
import FileUpload from './components/FileUpload';
import FileList from './components/FileList';
import ExpressionInput from './components/ExpressionInput';
import ResultsDisplay from './components/ResultsDisplay';
import { FhirVersion, OperationType, EvaluationResult, AnalysisResult } from './services/types';
import { api } from './services/api';

const App: Component = () => {
  const [fhirVersion, setFhirVersion] = createSignal<FhirVersion>('r4');
  const [operation, setOperation] = createSignal<OperationType>('evaluate');
  const [expression, setExpression] = createSignal('');
  const [selectedFile, setSelectedFile] = createSignal<string>('');
  const [loading, setLoading] = createSignal(false);
  const [results, setResults] = createSignal<EvaluationResult | AnalysisResult | null>(null);
  const [error, setError] = createSignal<string>('');
  const [fileListKey, setFileListKey] = createSignal(0);

  const handleExecute = async () => {
    if (!expression().trim()) {
      setError('Please enter a FHIRPath expression');
      return;
    }

    setLoading(true);
    setError('');
    setResults(null);

    try {
      if (operation() === 'evaluate') {
        const result = await api.evaluate(
          fhirVersion(),
          expression(),
          undefined,
          selectedFile() || undefined
        );
        
        // Check if the server returned an error in the response
        if (!result.success && result.error) {
          setError(result.error.message || 'Evaluation failed');
          setResults(null);
        } else {
          setResults(result);
          setError('');
        }
      } else {
        const result = await api.analyze(fhirVersion(), expression());
        
        // Check if the server returned an error in the response
        if (!result.success && result.error) {
          setError(result.error.message || 'Analysis failed');
          setResults(null);
        } else {
          setResults(result);
          setError('');
        }
      }
    } catch (err) {
      // Network or other errors
      setError(err instanceof Error ? err.message : 'An error occurred');
      setResults(null);
    } finally {
      setLoading(false);
    }
  };

  const handleFileUploaded = () => {
    // Trigger refresh of file list by updating the key
    setFileListKey(prev => prev + 1);
  };

  const handleClear = () => {
    setExpression('');
    setResults(null);
    setError('');
  };

  return (
    <div class={styles.app}>
      <header class={styles.header}>
        <div class={styles.headerContent}>
          <div class={styles.brandSection}>
            <img src="/octofhir.png" alt="OctoFHIR Logo" class={styles.logo} />
            <div class={styles.titleSection}>
              <h1 class={styles.title}>FHIRPath Evaluator</h1>
              <p class={styles.subtitle}>
                Interactive expression evaluation and analysis
              </p>
            </div>
          </div>
          <div class={styles.topControls}>
            <div class={styles.versionSelector}>
              <label class={styles.versionLabel} for="fhir-version">
                FHIR Version:
              </label>
              <select 
                id="fhir-version"
                class="select"
                value={fhirVersion()}
                onChange={(e) => setFhirVersion(e.currentTarget.value as FhirVersion)}
              >
                <option value="r4">R4</option>
                <option value="r4b">R4B</option>
                <option value="r5">R5</option>
                <option value="r6">R6</option>
              </select>
            </div>
            <div class={styles.operationSwitch}>
              <div class={styles.switchContainer}>
                <span class={`${styles.switchLabel} ${operation() === 'evaluate' ? styles.active : ''}`}>
                  📊 Evaluate
                </span>
                <label class={styles.switch}>
                  <input
                    type="checkbox"
                    checked={operation() === 'analyze'}
                    onChange={() => setOperation(operation() === 'evaluate' ? 'analyze' : 'evaluate')}
                  />
                  <span class={styles.slider}></span>
                </label>
                <span class={`${styles.switchLabel} ${operation() === 'analyze' ? styles.active : ''}`}>
                  🔍 Analyze
                </span>
              </div>
            </div>
            <div class={styles.actionButtons}>
              <button 
                class="button button-secondary"
                onClick={handleClear}
                disabled={loading()}
              >
                Clear
              </button>
              <button 
                class="button button-primary"
                onClick={handleExecute}
                disabled={loading() || !expression().trim()}
              >
                {loading() ? 'Processing...' : operation() === 'evaluate' ? 'Evaluate' : 'Analyze'}
              </button>
            </div>
          </div>
        </div>
      </header>

      <main class={styles.main}>
        <div class={styles.leftPanel}>
          <section class={styles.section}>
            <h2 class={styles.sectionTitle}>File Management</h2>
            <FileUpload onFileUploaded={handleFileUploaded} />
            <FileList 
              selectedFile={selectedFile()}
              onFileSelect={setSelectedFile}
              refreshTrigger={fileListKey()}
            />
          </section>

          <section class={styles.section}>
            <h2 class={styles.sectionTitle}>FHIRPath Expression</h2>
            <ExpressionInput 
              expression={expression()}
              onExpressionChange={setExpression}
              disabled={loading()}
            />
          </section>
        </div>

        <div class={styles.rightPanel}>
          <section class={styles.section}>
            <h2 class={styles.sectionTitle}>
              {operation() === 'evaluate' ? 'Evaluation Results' : 'Analysis Results'}
            </h2>
            <ResultsDisplay 
              results={results()}
              error={error()}
              loading={loading()}
              operation={operation()}
            />
          </section>
        </div>
      </main>
    </div>
  );
};

export default App;