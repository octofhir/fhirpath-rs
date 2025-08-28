export interface FileInfo {
  name: string;
  size: number;
  modified: string;
  type: string;
}

export interface UploadResponse {
  success: boolean;
  filename: string;
  message?: string;
}

export interface EvaluationResult {
  success: boolean;
  result?: any;
  error?: ErrorInfo;
  expression: string;
  fhir_version: string;
  metadata: ExecutionMetadata;
  trace?: string[];
}

export interface ErrorInfo {
  code: string;
  message: string;
  details?: string;
  location?: SourceLocation;
}

export interface ExecutionMetadata {
  execution_time_ms: number;
  cache_hits: number;
  ast_nodes: number;
  memory_used: number;
  engine_reused: boolean;
}

export interface SourceLocation {
  line: number;
  column: number;
  offset: number;
}

export interface Diagnostic {
  level: 'error' | 'warning' | 'info';
  message: string;
  location?: {
    line: number;
    column: number;
    length?: number;
  };
}

export interface Optimization {
  type: string;
  description: string;
  impact: 'high' | 'medium' | 'low';
}

export interface AnalysisResult {
  success: boolean;
  analysis?: AnalysisData;
  error?: ErrorInfo;
  expression: string;
  fhir_version: string;
  metadata: ExecutionMetadata;
}

export interface AnalysisData {
  type_info?: TypeInfo;
  validation_errors: ValidationErrorInfo[];
  type_annotations: number;
  function_calls: number;
  union_types: number;
}

export interface TypeInfo {
  return_type: string;
  constraints: string[];
  cardinality: string;
}

export interface ValidationErrorInfo {
  message: string;
  severity: string;
  location?: SourceLocation;
}

export type FhirVersion = 'r4' | 'r4b' | 'r5' | 'r6';

export type OperationType = 'evaluate' | 'analyze';

export interface ApiError {
  error: string;
  details?: string;
}