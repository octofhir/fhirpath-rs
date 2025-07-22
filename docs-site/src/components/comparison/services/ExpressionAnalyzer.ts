/**
 * Expression Analysis Service
 * Analyzes FHIRPath expressions to categorize patterns and provide insights
 */

import type { TestResult, TestResultSet, BenchmarkResult, BenchmarkResultSet } from '../types/comparison';

export interface ExpressionPattern {
  expression: string;
  category: ExpressionCategory;
  complexity: ExpressionComplexity;
  operations: string[];
  languages: LanguageResult[];
  benchmarkData?: BenchmarkResult[];
}

export interface LanguageResult {
  language: string;
  status: 'passed' | 'failed' | 'error' | 'not_tested';
  execution_time_ms?: number;
  error_message?: string;
  expected?: any;
  actual?: any;
}

export type ExpressionCategory =
  | 'navigation'
  | 'filtering'
  | 'aggregation'
  | 'string_manipulation'
  | 'type_conversion'
  | 'mathematical'
  | 'conditional'
  | 'collection_operations'
  | 'date_time'
  | 'reference_handling'
  | 'complex_mixed';

export type ExpressionComplexity = 'simple' | 'moderate' | 'complex' | 'very_complex';

export interface ExpressionAnalysis {
  totalExpressions: number;
  uniqueExpressions: number;
  categoryDistribution: Record<ExpressionCategory, number>;
  complexityDistribution: Record<ExpressionComplexity, number>;
  crossLanguageCompatibility: {
    fullyCompatible: number; // Works in all languages
    mostlyCompatible: number; // Works in 75%+ languages
    partiallyCompatible: number; // Works in 50-75% languages
    poorlyCompatible: number; // Works in <50% languages
  };
  topFailingExpressions: ExpressionPattern[];
  performanceInsights: {
    fastestCategory: ExpressionCategory;
    slowestCategory: ExpressionCategory;
    mostVariableCategory: ExpressionCategory;
  };
}

export class ExpressionAnalyzer {
  private patterns: Map<string, ExpressionPattern> = new Map();

  constructor(
    private testResults: TestResultSet[],
    private benchmarkResults: BenchmarkResultSet[]
  ) {
    this.analyzeExpressions();
  }

  private analyzeExpressions(): void {
    // Analyze test expressions
    for (const resultSet of this.testResults) {
      if (!resultSet.tests) continue;

      for (const test of resultSet.tests) {
        if (!test.expression) continue;

        const expression = test.expression;
        if (!this.patterns.has(expression)) {
          this.patterns.set(expression, {
            expression,
            category: this.categorizeExpression(expression),
            complexity: this.assessComplexity(expression),
            operations: this.extractOperations(expression),
            languages: [],
            benchmarkData: []
          });
        }

        const pattern = this.patterns.get(expression)!;
        pattern.languages.push({
          language: resultSet.language,
          status: test.status,
          execution_time_ms: test.execution_time_ms,
          error_message: test.error,
          expected: test.expected,
          actual: test.actual
        });
      }
    }

    // Analyze benchmark expressions
    for (const resultSet of this.benchmarkResults) {
      for (const benchmark of resultSet.benchmarks) {
        if (!benchmark.expression) continue;

        const expression = benchmark.expression;
        if (!this.patterns.has(expression)) {
          this.patterns.set(expression, {
            expression,
            category: this.categorizeExpression(expression),
            complexity: this.assessComplexity(expression),
            operations: this.extractOperations(expression),
            languages: [],
            benchmarkData: []
          });
        }

        const pattern = this.patterns.get(expression)!;
        pattern.benchmarkData!.push({
          ...benchmark,
          language: resultSet.language
        } as BenchmarkResult & { language: string });
      }
    }
  }

  private categorizeExpression(expression: string): ExpressionCategory {
    // Ensure expression is a string and handle edge cases
    if (typeof expression !== 'string') {
      expression = String(expression || '');
    }
    const expr = expression.toLowerCase();

    // Complex mixed operations (check first for most specific patterns)
    if (this.countOperations(expr) >= 4 &&
        (expr.includes('where') || expr.includes('select')) &&
        (expr.includes('count') || expr.includes('sum') || expr.includes('first')) &&
        (expr.includes('.') && expr.split('.').length >= 4)) {
      return 'complex_mixed';
    }

    // String manipulation
    if (expr.includes('substring') || expr.includes('matches') || expr.includes('replace') ||
        expr.includes('split') || expr.includes('contains') || expr.includes('startswith') ||
        expr.includes('endswith') || expr.includes('upper') || expr.includes('lower') ||
        expr.includes('trim') || expr.includes('length')) {
      return 'string_manipulation';
    }

    // Mathematical operations
    if (expr.includes('sum') || expr.includes('avg') || expr.includes('min') || expr.includes('max') ||
        expr.includes('+') || expr.includes('-') || expr.includes('*') || expr.includes('/') ||
        expr.includes('mod') || expr.includes('div') || expr.includes('abs') || expr.includes('ceiling') ||
        expr.includes('floor') || expr.includes('round') || expr.includes('sqrt')) {
      return 'mathematical';
    }

    // Aggregation operations
    if (expr.includes('count') || expr.includes('sum') || expr.includes('avg') ||
        expr.includes('min') || expr.includes('max') || expr.includes('distinct') ||
        expr.includes('aggregate') || expr.includes('reduce')) {
      return 'aggregation';
    }

    // Collection operations
    if (expr.includes('first') || expr.includes('last') || expr.includes('tail') ||
        expr.includes('take') || expr.includes('skip') || expr.includes('union') ||
        expr.includes('intersect') || expr.includes('exclude') || expr.includes('flatten') ||
        expr.includes('repeat') || expr.includes('descendants') || expr.includes('children')) {
      return 'collection_operations';
    }

    // Filtering operations
    if (expr.includes('where') || expr.includes('select') || expr.includes('exists') ||
        expr.includes('empty') || expr.includes('not') || expr.includes('all') ||
        expr.includes('any') || expr.includes('single') || expr.includes('oftype')) {
      return 'filtering';
    }

    // Conditional operations
    if (expr.includes('iif') || expr.includes('if') || expr.includes('then') || expr.includes('else') ||
        (expr.includes('and') && expr.includes('or')) || expr.includes('implies') ||
        expr.includes('xor')) {
      return 'conditional';
    }

    // Type conversion
    if (expr.includes('as') || expr.includes('is') || expr.includes('convertstointeger') ||
        expr.includes('convertstodecimal') || expr.includes('convertstostring') ||
        expr.includes('convertstoboolean') || expr.includes('convertstodatetime') ||
        expr.includes('convertstodate') || expr.includes('convertstotime') ||
        expr.includes('tostring') || expr.includes('tointeger') || expr.includes('todecimal')) {
      return 'type_conversion';
    }

    // Date/time operations
    if (expr.includes('now') || expr.includes('today') || expr.includes('timeofday') ||
        expr.includes('year') || expr.includes('month') || expr.includes('day') ||
        expr.includes('hour') || expr.includes('minute') || expr.includes('second') ||
        expr.includes('millisecond') || expr.includes('timezone') || expr.includes('utc') ||
        expr.includes('date') || expr.includes('time') || expr.includes('datetime') ||
        expr.includes('period') || expr.includes('duration')) {
      return 'date_time';
    }

    // Reference handling
    if (expr.includes('reference') || expr.includes('resolve') || expr.includes('conformsto') ||
        expr.includes('memberof') || expr.includes('subsumes') || expr.includes('subsumedby') ||
        expr.includes('hasvalue') || expr.includes('htmlchecks') || expr.includes('extension')) {
      return 'reference_handling';
    }

    // Default to navigation if it's simple path traversal
    return 'navigation';
  }

  private assessComplexity(expression: string): ExpressionComplexity {
    // Ensure expression is a string and handle edge cases
    if (typeof expression !== 'string') {
      expression = String(expression || '');
    }
    const operationCount = this.countOperations(expression);
    const pathDepth = expression.split('.').length;
    const hasNestedFunctions = /\w+\([^)]*\([^)]*\)/.test(expression);
    const hasComplexLogic = (expression.match(/and|or|implies|xor/gi) || []).length;
    const hasStringPatterns = /matches\s*\(|contains\s*\(|substring\s*\(/.test(expression);

    let complexityScore = 0;

    // Base complexity from operation count
    if (operationCount >= 8) complexityScore += 4;
    else if (operationCount >= 5) complexityScore += 3;
    else if (operationCount >= 3) complexityScore += 2;
    else if (operationCount >= 2) complexityScore += 1;

    // Path depth complexity
    if (pathDepth >= 6) complexityScore += 3;
    else if (pathDepth >= 4) complexityScore += 2;
    else if (pathDepth >= 3) complexityScore += 1;

    // Additional complexity factors
    if (hasNestedFunctions) complexityScore += 2;
    if (hasComplexLogic >= 3) complexityScore += 2;
    else if (hasComplexLogic >= 1) complexityScore += 1;
    if (hasStringPatterns) complexityScore += 1;

    // Expression length factor
    if (expression.length > 200) complexityScore += 2;
    else if (expression.length > 100) complexityScore += 1;

    if (complexityScore >= 8) return 'very_complex';
    if (complexityScore >= 5) return 'complex';
    if (complexityScore >= 2) return 'moderate';
    return 'simple';
  }

  private countOperations(expression: string): number {
    // Ensure expression is a string and handle edge cases
    if (typeof expression !== 'string') {
      expression = String(expression || '');
    }
    const operations = [
      'where', 'select', 'first', 'last', 'count', 'sum', 'avg', 'min', 'max',
      'exists', 'empty', 'not', 'and', 'or', 'implies', 'xor', 'iif',
      'substring', 'matches', 'contains', 'split', 'replace', 'upper', 'lower',
      'as', 'is', 'oftype', 'union', 'intersect', 'exclude', 'distinct',
      'take', 'skip', 'tail', 'repeat', 'descendants', 'children'
    ];

    let count = 0;
    const lowerExpr = expression.toLowerCase();

    for (const op of operations) {
      const regex = new RegExp(`\\b${op}\\b`, 'g');
      const matches = lowerExpr.match(regex);
      if (matches) count += matches.length;
    }

    return count;
  }

  private extractOperations(expression: string): string[] {
    // Ensure expression is a string and handle edge cases
    if (typeof expression !== 'string') {
      expression = String(expression || '');
    }
    const operations = new Set<string>();
    const lowerExpr = expression.toLowerCase();

    const operationPatterns = [
      'where', 'select', 'first', 'last', 'count', 'sum', 'avg', 'min', 'max',
      'exists', 'empty', 'not', 'and', 'or', 'implies', 'xor', 'iif',
      'substring', 'matches', 'contains', 'split', 'replace', 'upper', 'lower',
      'as', 'is', 'oftype', 'union', 'intersect', 'exclude', 'distinct',
      'take', 'skip', 'tail', 'repeat', 'descendants', 'children',
      'resolve', 'extension', 'hasvalue', 'memberof', 'subsumes'
    ];

    for (const op of operationPatterns) {
      if (new RegExp(`\\b${op}\\b`, 'i').test(expression)) {
        operations.add(op);
      }
    }

    return Array.from(operations);
  }

  public getAnalysis(): ExpressionAnalysis {
    const patterns = Array.from(this.patterns.values());
    const totalLanguages = this.testResults.length;

    // Category distribution
    const categoryDistribution: Record<ExpressionCategory, number> = {
      navigation: 0,
      filtering: 0,
      aggregation: 0,
      string_manipulation: 0,
      type_conversion: 0,
      mathematical: 0,
      conditional: 0,
      collection_operations: 0,
      date_time: 0,
      reference_handling: 0,
      complex_mixed: 0
    };

    // Complexity distribution
    const complexityDistribution: Record<ExpressionComplexity, number> = {
      simple: 0,
      moderate: 0,
      complex: 0,
      very_complex: 0
    };

    // Cross-language compatibility
    let fullyCompatible = 0;
    let mostlyCompatible = 0;
    let partiallyCompatible = 0;
    let poorlyCompatible = 0;

    const topFailingExpressions: ExpressionPattern[] = [];

    for (const pattern of patterns) {
      categoryDistribution[pattern.category]++;
      complexityDistribution[pattern.complexity]++;

      const passedLanguages = pattern.languages.filter(l => l.status === 'passed').length;
      const compatibilityRate = passedLanguages / Math.max(pattern.languages.length, 1);

      if (compatibilityRate === 1) fullyCompatible++;
      else if (compatibilityRate >= 0.75) mostlyCompatible++;
      else if (compatibilityRate >= 0.5) partiallyCompatible++;
      else poorlyCompatible++;

      // Track failing expressions
      const failedLanguages = pattern.languages.filter(l => l.status === 'failed' || l.status === 'error').length;
      if (failedLanguages > 0) {
        topFailingExpressions.push(pattern);
      }
    }

    // Sort failing expressions by failure count
    topFailingExpressions.sort((a, b) => {
      const aFailed = a.languages.filter(l => l.status === 'failed' || l.status === 'error').length;
      const bFailed = b.languages.filter(l => l.status === 'failed' || l.status === 'error').length;
      return bFailed - aFailed;
    });

    // Performance insights
    const categoryPerformance = new Map<ExpressionCategory, number[]>();

    for (const pattern of patterns) {
      const times = pattern.languages
        .filter(l => l.execution_time_ms !== undefined)
        .map(l => l.execution_time_ms!);

      if (times.length > 0) {
        if (!categoryPerformance.has(pattern.category)) {
          categoryPerformance.set(pattern.category, []);
        }
        categoryPerformance.get(pattern.category)!.push(...times);
      }
    }

    let fastestCategory: ExpressionCategory = 'navigation';
    let slowestCategory: ExpressionCategory = 'navigation';
    let mostVariableCategory: ExpressionCategory = 'navigation';
    let fastestAvg = Infinity;
    let slowestAvg = 0;
    let highestVariance = 0;

    Array.from(categoryPerformance.entries()).forEach(([category, times]) => {
      if (times.length === 0) return;

      const avg = times.reduce((sum, time) => sum + time, 0) / times.length;
      const variance = times.reduce((sum, time) => sum + Math.pow(time - avg, 2), 0) / times.length;

      if (avg < fastestAvg) {
        fastestAvg = avg;
        fastestCategory = category;
      }

      if (avg > slowestAvg) {
        slowestAvg = avg;
        slowestCategory = category;
      }

      if (variance > highestVariance) {
        highestVariance = variance;
        mostVariableCategory = category;
      }
    });

    return {
      totalExpressions: patterns.length,
      uniqueExpressions: patterns.length,
      categoryDistribution,
      complexityDistribution,
      crossLanguageCompatibility: {
        fullyCompatible,
        mostlyCompatible,
        partiallyCompatible,
        poorlyCompatible
      },
      topFailingExpressions: topFailingExpressions.slice(0, 10),
      performanceInsights: {
        fastestCategory,
        slowestCategory,
        mostVariableCategory
      }
    };
  }

  public getPatternsByCategory(category: ExpressionCategory): ExpressionPattern[] {
    return Array.from(this.patterns.values()).filter(p => p.category === category);
  }

  public getPatternsByComplexity(complexity: ExpressionComplexity): ExpressionPattern[] {
    return Array.from(this.patterns.values()).filter(p => p.complexity === complexity);
  }

  public searchPatterns(query: string): ExpressionPattern[] {
    const lowerQuery = query.toLowerCase();
    return Array.from(this.patterns.values()).filter(p =>
      p.expression.toLowerCase().includes(lowerQuery) ||
      p.operations.some(op => op.includes(lowerQuery))
    );
  }

  public getPattern(expression: string): ExpressionPattern | undefined {
    return this.patterns.get(expression);
  }

  public getAllPatterns(): ExpressionPattern[] {
    return Array.from(this.patterns.values());
  }
}
