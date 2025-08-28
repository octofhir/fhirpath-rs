import { Component, Show, For } from 'solid-js';
import styles from './ResultsDisplay.module.css';
import { EvaluationResult, AnalysisResult, OperationType, ValidationErrorInfo } from '../services/types';

interface ResultsDisplayProps {
  results: EvaluationResult | AnalysisResult | null;
  error: string;
  loading: boolean;
  operation: OperationType;
}

const ResultsDisplay: Component<ResultsDisplayProps> = (props) => {
  const copyToClipboard = async (text: string) => {
    try {
      await navigator.clipboard.writeText(text);
    } catch (err) {
      console.error('Failed to copy to clipboard:', err);
    }
  };

  const downloadResults = () => {
    if (!props.results) return;
    
    const dataStr = JSON.stringify(props.results, null, 2);
    const dataUri = 'data:application/json;charset=utf-8,' + encodeURIComponent(dataStr);
    
    const exportFileDefaultName = `fhirpath-${props.operation}-results.json`;
    
    const linkElement = document.createElement('a');
    linkElement.setAttribute('href', dataUri);
    linkElement.setAttribute('download', exportFileDefaultName);
    linkElement.click();
  };

  const formatResultsForDisplay = (results: EvaluationResult | AnalysisResult) => {
    if (props.operation === 'evaluate') {
      const evalResults = results as EvaluationResult;
      return JSON.stringify(evalResults.result, null, 2);
    } else {
      const analysisResults = results as AnalysisResult;
      return JSON.stringify(analysisResults.analysis, null, 2);
    }
  };

  const getDiagnosticIcon = (level: string) => {
    switch (level) {
      case 'error': return '❌';
      case 'warning': return '⚠️';
      case 'info': return 'ℹ️';
      default: return '📝';
    }
  };

  const getDiagnosticClass = (level: string) => {
    switch (level) {
      case 'error': return styles.diagnosticError;
      case 'warning': return styles.diagnosticWarning;
      case 'info': return styles.diagnosticInfo;
      default: return styles.diagnosticInfo;
    }
  };

  const getOptimizationImpactClass = (impact: string) => {
    switch (impact) {
      case 'high': return styles.impactHigh;
      case 'medium': return styles.impactMedium;
      case 'low': return styles.impactLow;
      default: return styles.impactLow;
    }
  };

  const renderEvaluationResults = (results: EvaluationResult) => {
    const resultArray = Array.isArray(results.result) ? results.result : (results.result ? [results.result] : []);
    
    return (
      <div class={styles.results}>
        <div class={styles.resultsHeader}>
          <div class={styles.resultsTitle}>
            📊 Evaluation Results ({resultArray.length} item{resultArray.length !== 1 ? 's' : ''})
          </div>
          <div class={styles.resultsActions}>
            <button 
              class={styles.actionButton}
              onClick={() => copyToClipboard(formatResultsForDisplay(results))}
              title="Copy results to clipboard"
            >
              📋 Copy
            </button>
            <button 
              class={styles.actionButton}
              onClick={downloadResults}
              title="Download results as JSON"
            >
              💾 Download
            </button>
          </div>
        </div>

        <div class={styles.resultsContent}>
          <pre class={styles.jsonOutput}>
            {formatResultsForDisplay(results)}
          </pre>
        </div>

        <div class={styles.executionStats}>
          <div class={styles.stat}>
            <span class={styles.statIcon}>⏱️</span>
            <span>Execution Time: {results.metadata.execution_time_ms}ms</span>
          </div>
          <div class={styles.stat}>
            <span class={styles.statIcon}>📏</span>
            <span>Result Count: {resultArray.length}</span>
          </div>
          <div class={styles.stat}>
            <span class={styles.statIcon}>🏗️</span>
            <span>AST Nodes: {results.metadata.ast_nodes}</span>
          </div>
          <div class={styles.stat}>
            <span class={styles.statIcon}>🔄</span>
            <span>Engine Reused: {results.metadata.engine_reused ? 'Yes' : 'No'}</span>
          </div>
        </div>
      </div>
    );
  };

  const renderAnalysisResults = (results: AnalysisResult) => {
    const analysisData = results.analysis;
    const isValid = results.success && (!analysisData?.validation_errors || analysisData.validation_errors.length === 0);
    
    return (
      <div class={styles.results}>
        <div class={styles.resultsHeader}>
          <div class={styles.resultsTitle}>
            🔍 Analysis Results {isValid ? '✅' : '❌'}
          </div>
          <div class={styles.resultsActions}>
            <button 
              class={styles.actionButton}
              onClick={() => copyToClipboard(formatResultsForDisplay(results))}
              title="Copy analysis to clipboard"
            >
              📋 Copy
            </button>
            <button 
              class={styles.actionButton}
              onClick={downloadResults}
              title="Download analysis as JSON"
            >
              💾 Download
            </button>
          </div>
        </div>

        <div class={styles.resultsContent}>
          <div class={styles.analysisResults}>
            <Show when={analysisData?.validation_errors && analysisData.validation_errors.length > 0}>
              <div class={styles.diagnostics}>
                <div class={styles.diagnosticsTitle}>
                  📋 Validation Errors ({analysisData!.validation_errors.length})
                </div>
                <For each={analysisData!.validation_errors}>
                  {(error) => (
                    <div class={`${styles.diagnostic} ${getDiagnosticClass(error.severity)}`}>
                      <div class={styles.diagnosticIcon}>
                        {getDiagnosticIcon(error.severity)}
                      </div>
                      <div class={styles.diagnosticContent}>
                        <div class={styles.diagnosticMessage}>
                          {error.message}
                        </div>
                        <Show when={error.location}>
                          <div class={styles.diagnosticLocation}>
                            Line {error.location!.line}, Column {error.location!.column}
                          </div>
                        </Show>
                      </div>
                    </div>
                  )}
                </For>
              </div>
            </Show>

            <Show when={analysisData}>
              <div class={styles.executionStats}>
                <div class={styles.stat}>
                  <span class={styles.statIcon}>⏱️</span>
                  <span>Execution Time: {results.metadata.execution_time_ms}ms</span>
                </div>
                <div class={styles.stat}>
                  <span class={styles.statIcon}>🎯</span>
                  <span>Type Annotations: {analysisData!.type_annotations}</span>
                </div>
                <div class={styles.stat}>
                  <span class={styles.statIcon}>📞</span>
                  <span>Function Calls: {analysisData!.function_calls}</span>
                </div>
                <div class={styles.stat}>
                  <span class={styles.statIcon}>🔗</span>
                  <span>Union Types: {analysisData!.union_types}</span>
                </div>
              </div>
            </Show>
          </div>
        </div>
      </div>
    );
  };

  return (
    <div class={styles.resultsDisplay}>
      <Show when={props.loading}>
        <div class={styles.loading}>
          <div class={styles.spinner}></div>
          <div class={styles.loadingText}>
            {props.operation === 'evaluate' ? 'Evaluating Expression' : 'Analyzing Expression'}
          </div>
          <div class={styles.loadingSubtext}>
            Processing {props.operation === 'evaluate' ? 'FHIRPath evaluation' : 'syntax analysis'}...
          </div>
        </div>
      </Show>

      <Show when={props.error && !props.loading}>
        <div class={styles.error}>
          <div class={styles.errorContent}>
            <span class={styles.errorIcon}>⚠️</span>
            <div>
              <div class={styles.errorMessage}>
                Operation Failed
              </div>
              <div class={styles.errorDetails}>
                {props.error}
              </div>
            </div>
          </div>
          <div class={styles.errorActions}>
            <button class={styles.errorAction} onClick={() => window.location.reload()}>
              🔄 Retry
            </button>
            <button class={styles.errorAction} onClick={() => copyToClipboard(props.error)}>
              📋 Copy Error
            </button>
          </div>
        </div>
      </Show>

      <Show when={props.results && !props.loading && !props.error}>
        {props.operation === 'evaluate' 
          ? renderEvaluationResults(props.results as EvaluationResult)
          : renderAnalysisResults(props.results as AnalysisResult)
        }
      </Show>

      <Show when={!props.results && !props.loading && !props.error}>
        <div class={styles.placeholder}>
          <div class={styles.placeholderIcon}>
            {props.operation === 'evaluate' ? '📊' : '🔍'}
          </div>
          <div class={styles.placeholderText}>
            {props.operation === 'evaluate' ? 'No results yet' : 'No analysis yet'}
          </div>
          <div class={styles.placeholderSubtext}>
            Enter a FHIRPath expression and click "
            {props.operation === 'evaluate' ? 'Evaluate' : 'Analyze'}" to see results here.
          </div>
        </div>
      </Show>
    </div>
  );
};

export default ResultsDisplay;