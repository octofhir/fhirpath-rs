/**
 * URL State Management Hook
 * Manages comparison application state in URL for sharing and persistence
 */

import { useState, useEffect, useCallback } from 'react';
import type { FilterCriteria } from '../components/Common/AdvancedFilter';

export interface URLState {
  tab?: string;
  languages?: string[];
  performanceMin?: number;
  performanceMax?: number;
  successRateMin?: number;
  successRateMax?: number;
  executionTimeMin?: number;
  executionTimeMax?: number;
  minBenchmarks?: number;
  minTests?: number;
  excludeErrors?: boolean;
  onlyProductionReady?: boolean;
  sortBy?: string;
  sortOrder?: string;
  shared?: boolean;
}

export interface URLStateManager {
  state: URLState;
  updateState: (updates: Partial<URLState>) => void;
  generateShareableURL: () => string;
  resetState: () => void;
  isSharedView: boolean;
}

/**
 * Hook for managing comparison application state in URL
 */
export function useURLState(): URLStateManager {
  const [state, setState] = useState<URLState>({});
  const [isSharedView, setIsSharedView] = useState(false);

  // Parse URL parameters on mount
  useEffect(() => {
    const urlParams = new URLSearchParams(window.location.search);
    const parsedState: URLState = {};

    // Parse tab
    const tab = urlParams.get('tab');
    if (tab) parsedState.tab = tab;

    // Parse languages
    const languages = urlParams.get('languages');
    if (languages) parsedState.languages = languages.split(',');

    // Parse performance range
    const performanceMin = urlParams.get('performanceMin');
    const performanceMax = urlParams.get('performanceMax');
    if (performanceMin) parsedState.performanceMin = Number(performanceMin);
    if (performanceMax) parsedState.performanceMax = Number(performanceMax);

    // Parse success rate range
    const successRateMin = urlParams.get('successRateMin');
    const successRateMax = urlParams.get('successRateMax');
    if (successRateMin) parsedState.successRateMin = Number(successRateMin);
    if (successRateMax) parsedState.successRateMax = Number(successRateMax);

    // Parse execution time range
    const executionTimeMin = urlParams.get('executionTimeMin');
    const executionTimeMax = urlParams.get('executionTimeMax');
    if (executionTimeMin) parsedState.executionTimeMin = Number(executionTimeMin);
    if (executionTimeMax) parsedState.executionTimeMax = Number(executionTimeMax);

    // Parse minimum requirements
    const minBenchmarks = urlParams.get('minBenchmarks');
    const minTests = urlParams.get('minTests');
    if (minBenchmarks) parsedState.minBenchmarks = Number(minBenchmarks);
    if (minTests) parsedState.minTests = Number(minTests);

    // Parse boolean flags
    const excludeErrors = urlParams.get('excludeErrors');
    const onlyProductionReady = urlParams.get('onlyProductionReady');
    if (excludeErrors) parsedState.excludeErrors = excludeErrors === 'true';
    if (onlyProductionReady) parsedState.onlyProductionReady = onlyProductionReady === 'true';

    // Parse sorting
    const sortBy = urlParams.get('sortBy');
    const sortOrder = urlParams.get('sortOrder');
    if (sortBy) parsedState.sortBy = sortBy;
    if (sortOrder) parsedState.sortOrder = sortOrder;

    // Check if this is a shared view
    const shared = urlParams.get('shared');
    if (shared) {
      parsedState.shared = true;
      setIsSharedView(true);
    }

    setState(parsedState);
  }, []);

  // Update URL when state changes
  const updateURL = useCallback((newState: URLState) => {
    const url = new URL(window.location.href);
    const params = url.searchParams;

    // Clear existing comparison params
    const comparisonParams = [
      'tab', 'languages', 'performanceMin', 'performanceMax',
      'successRateMin', 'successRateMax', 'executionTimeMin', 'executionTimeMax',
      'minBenchmarks', 'minTests', 'excludeErrors', 'onlyProductionReady',
      'sortBy', 'sortOrder', 'shared'
    ];

    comparisonParams.forEach(param => params.delete(param));

    // Add new params
    Object.entries(newState).forEach(([key, value]) => {
      if (value !== undefined && value !== null) {
        if (Array.isArray(value)) {
          params.set(key, value.join(','));
        } else {
          params.set(key, String(value));
        }
      }
    });

    // Update URL without page reload
    window.history.replaceState({}, '', url.toString());
  }, []);

  // Update state and URL
  const updateState = useCallback((updates: Partial<URLState>) => {
    const newState = { ...state, ...updates };
    setState(newState);
    updateURL(newState);
  }, [state, updateURL]);

  // Generate shareable URL
  const generateShareableURL = useCallback(() => {
    const url = new URL(window.location.href);
    const params = url.searchParams;

    // Add shared flag and timestamp
    params.set('shared', 'true');
    params.set('timestamp', Date.now().toString());

    return url.toString();
  }, []);

  // Reset state
  const resetState = useCallback(() => {
    const newState: URLState = {};
    setState(newState);

    // Clear URL params
    const url = new URL(window.location.href);
    const comparisonParams = [
      'tab', 'languages', 'performanceMin', 'performanceMax',
      'successRateMin', 'successRateMax', 'executionTimeMin', 'executionTimeMax',
      'minBenchmarks', 'minTests', 'excludeErrors', 'onlyProductionReady',
      'sortBy', 'sortOrder', 'shared', 'timestamp'
    ];

    comparisonParams.forEach(param => url.searchParams.delete(param));
    window.history.replaceState({}, '', url.toString());

    setIsSharedView(false);
  }, []);

  return {
    state,
    updateState,
    generateShareableURL,
    resetState,
    isSharedView
  };
}

/**
 * Convert URL state to filter criteria
 */
export function urlStateToFilterCriteria(
  urlState: URLState,
  defaultCriteria: FilterCriteria
): Partial<FilterCriteria> {
  const criteria: Partial<FilterCriteria> = {};

  if (urlState.languages) criteria.languages = urlState.languages;

  if (urlState.performanceMin !== undefined && urlState.performanceMax !== undefined) {
    criteria.performanceRange = [urlState.performanceMin, urlState.performanceMax];
  }

  if (urlState.successRateMin !== undefined && urlState.successRateMax !== undefined) {
    criteria.successRateRange = [urlState.successRateMin, urlState.successRateMax];
  }

  if (urlState.executionTimeMin !== undefined && urlState.executionTimeMax !== undefined) {
    criteria.executionTimeRange = [urlState.executionTimeMin, urlState.executionTimeMax];
  }

  if (urlState.minBenchmarks !== undefined) criteria.minBenchmarks = urlState.minBenchmarks;
  if (urlState.minTests !== undefined) criteria.minTests = urlState.minTests;
  if (urlState.excludeErrors !== undefined) criteria.excludeErrors = urlState.excludeErrors;
  if (urlState.onlyProductionReady !== undefined) criteria.onlyProductionReady = urlState.onlyProductionReady;

  if (urlState.sortBy) criteria.sortBy = urlState.sortBy as FilterCriteria['sortBy'];
  if (urlState.sortOrder) criteria.sortOrder = urlState.sortOrder as FilterCriteria['sortOrder'];

  return criteria;
}

/**
 * Convert filter criteria to URL state
 */
export function filterCriteriaToURLState(criteria: FilterCriteria): URLState {
  return {
    languages: criteria.languages,
    performanceMin: criteria.performanceRange[0],
    performanceMax: criteria.performanceRange[1],
    successRateMin: criteria.successRateRange[0],
    successRateMax: criteria.successRateRange[1],
    executionTimeMin: criteria.executionTimeRange[0],
    executionTimeMax: criteria.executionTimeRange[1],
    minBenchmarks: criteria.minBenchmarks,
    minTests: criteria.minTests,
    excludeErrors: criteria.excludeErrors,
    onlyProductionReady: criteria.onlyProductionReady,
    sortBy: criteria.sortBy,
    sortOrder: criteria.sortOrder
  };
}

/**
 * Hook for managing tab state in URL
 */
export function useTabState(initialTab: string = 'overview') {
  const { state, updateState } = useURLState();

  const activeTab = state.tab || initialTab;

  const setActiveTab = useCallback((tab: string) => {
    updateState({ tab });
  }, [updateState]);

  return { activeTab, setActiveTab };
}

/**
 * Hook for managing filter state in URL
 */
export function useFilterState(defaultCriteria: FilterCriteria) {
  const { state, updateState } = useURLState();

  const filterCriteria = {
    ...defaultCriteria,
    ...urlStateToFilterCriteria(state, defaultCriteria)
  };

  const setFilterCriteria = useCallback((criteria: FilterCriteria) => {
    const urlState = filterCriteriaToURLState(criteria);
    updateState(urlState);
  }, [updateState]);

  return { filterCriteria, setFilterCriteria };
}

/**
 * Hook for sharing functionality
 */
export function useSharing() {
  const { generateShareableURL, isSharedView } = useURLState();

  const shareComparison = useCallback(async () => {
    const shareableURL = generateShareableURL();

    if (navigator.share) {
      try {
        await navigator.share({
          title: 'FHIRPath Implementation Comparison',
          text: 'Check out this FHIRPath implementation comparison',
          url: shareableURL
        });
        return { success: true, method: 'native' };
      } catch (error) {
        // Fall back to clipboard if native sharing fails
      }
    }

    // Fallback to clipboard
    try {
      await navigator.clipboard.writeText(shareableURL);
      return { success: true, method: 'clipboard', url: shareableURL };
    } catch (error) {
      return { success: false, error: 'Failed to copy to clipboard' };
    }
  }, [generateShareableURL]);

  return { shareComparison, isSharedView };
}
