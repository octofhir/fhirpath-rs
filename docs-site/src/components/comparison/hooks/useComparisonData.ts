/**
 * Custom hook for loading and managing comparison data
 */

import { useState, useEffect } from 'react';
import type {
  TestResultSet,
  BenchmarkResultSet,
  LanguageInfo,
  ComparisonData
} from '../types/comparison';
import {
  getAllTestResults,
  getAllBenchmarkResults,
  createSampleData,
  getEnhancedLanguageInfo,
  LANGUAGE_INFO
} from '../services/resultLoader';
import { usePerformanceMetrics } from './usePerformanceMetrics';

export function useComparisonData(): ComparisonData {
  const [testResults, setTestResults] = useState<TestResultSet[]>([]);
  const [benchmarkResults, setBenchmarkResults] = useState<BenchmarkResultSet[]>([]);
  const [languageInfo, setLanguageInfo] = useState<Record<string, LanguageInfo>>(LANGUAGE_INFO);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  // Calculate performance metrics based on current data
  const metrics = usePerformanceMetrics(testResults, benchmarkResults);

  useEffect(() => {
    let isMounted = true;

    const loadData = async () => {
      try {
        console.log('[DEBUG_LOG] Starting data load...');
        setLoading(true);
        setError(null);

        // Load test and benchmark results in parallel
        console.log('[DEBUG_LOG] Calling getAllTestResults and getAllBenchmarkResults...');
        const [testData, benchmarkData] = await Promise.all([
          getAllTestResults(),
          getAllBenchmarkResults()
        ]);
        console.log('[DEBUG_LOG] Data loading completed:', { testDataLength: testData.length, benchmarkDataLength: benchmarkData.length });

        // Only update state if component is still mounted
        if (!isMounted) return;

        // If no data was loaded, use sample data for demonstration
        if (testData.length === 0 && benchmarkData.length === 0) {
          console.log('No data loaded, using sample data');
          const sampleData = createSampleData();
          setTestResults(sampleData.testResults);
          setBenchmarkResults(sampleData.benchmarkResults);
        } else {
          setTestResults(testData);
          setBenchmarkResults(benchmarkData);
        }

        // Enhance language info with version information
        const enhancedInfo = getEnhancedLanguageInfo(testData, benchmarkData);
        setLanguageInfo(enhancedInfo);

        console.log(`[DEBUG_LOG] Loaded ${testData.length} test result sets and ${benchmarkData.length} benchmark result sets`);
      } catch (err) {
        console.log('[DEBUG_LOG] Error occurred during data loading:', err);
        if (!isMounted) {
          console.log('[DEBUG_LOG] Component unmounted, skipping error handling');
          return;
        }

        const errorMessage = err instanceof Error ? err.message : 'Unknown error occurred';
        console.error('Error loading comparison data:', err);
        setError(errorMessage);

        // Use sample data on error
        console.log('[DEBUG_LOG] Using sample data due to error');
        const sampleData = createSampleData();
        setTestResults(sampleData.testResults);
        setBenchmarkResults(sampleData.benchmarkResults);
      } finally {
        console.log('[DEBUG_LOG] Finally block executing, isMounted:', isMounted);
        if (isMounted) {
          console.log('[DEBUG_LOG] Setting loading to false');
          setLoading(false);
        } else {
          console.log('[DEBUG_LOG] Component unmounted, not setting loading to false');
        }
      }
    };

    loadData();

    // Cleanup function to prevent state updates on unmounted component
    return () => {
      isMounted = false;
    };
  }, []);

  return {
    testResults,
    benchmarkResults,
    languageInfo,
    metrics,
    loading,
    error
  };
}
