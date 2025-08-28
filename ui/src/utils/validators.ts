export const validateFhirPathExpression = (expression: string): {
  valid: boolean;
  message: string;
  suggestions?: string[];
} => {
  if (!expression.trim()) {
    return {
      valid: true,
      message: 'Enter a FHIRPath expression',
    };
  }

  // Basic validation rules
  const trimmedExpression = expression.trim();
  
  // Check for basic syntax issues
  const openParens = (trimmedExpression.match(/\(/g) || []).length;
  const closeParens = (trimmedExpression.match(/\)/g) || []).length;
  
  if (openParens !== closeParens) {
    return {
      valid: false,
      message: 'Unmatched parentheses',
      suggestions: ['Check that all opening parentheses have matching closing ones'],
    };
  }

  // Check for basic FHIRPath patterns
  const fhirPathPatterns = [
    /^[A-Za-z]/,  // Should start with a letter
    /\.[A-Za-z]/,  // Property access should start with letter after dot
  ];

  // Check for common FHIRPath functions
  const commonFunctions = [
    'where', 'select', 'first', 'last', 'exists', 'empty', 'count', 'length',
    'substring', 'contains', 'startsWith', 'endsWith', 'matches', 'replaceMatches',
    'as', 'is', 'ofType', 'extension', 'hasValue', 'getValue', 'iif', 'trace',
  ];

  const hasValidPattern = fhirPathPatterns.some(pattern => pattern.test(trimmedExpression));
  const hasCommonFunction = commonFunctions.some(func => trimmedExpression.includes(func + '('));
  
  if (!hasValidPattern && !hasCommonFunction && trimmedExpression.length > 0) {
    return {
      valid: false,
      message: 'Invalid FHIRPath syntax',
      suggestions: [
        'FHIRPath expressions should start with a resource type or property name',
        'Examples: Patient.name, Observation.value, Bundle.entry',
        `Common functions: ${commonFunctions.slice(0, 5).join(', ')}...`,
      ],
    };
  }

  return {
    valid: true,
    message: 'Expression looks valid',
  };
};

export const validateJsonContent = (content: string): {
  valid: boolean;
  message: string;
  parsed?: any;
} => {
  if (!content.trim()) {
    return {
      valid: false,
      message: 'Empty content',
    };
  }

  try {
    const parsed = JSON.parse(content);
    return {
      valid: true,
      message: 'Valid JSON',
      parsed,
    };
  } catch (error) {
    return {
      valid: false,
      message: error instanceof Error ? error.message : 'Invalid JSON',
    };
  }
};

export const validateFileUpload = (file: File): {
  valid: boolean;
  message: string;
} => {
  // Check file type
  if (file.type !== 'application/json' && !file.name.endsWith('.json')) {
    return {
      valid: false,
      message: 'Only JSON files are allowed',
    };
  }

  // Check file size (max 10MB)
  const maxSize = 60 * 1024 * 1024; // 10MB
  if (file.size > maxSize) {
    return {
      valid: false,
      message: 'File size must be less than 10MB',
    };
  }

  // Check for empty files
  if (file.size === 0) {
    return {
      valid: false,
      message: 'File cannot be empty',
    };
  }

  return {
    valid: true,
    message: 'File is valid for upload',
  };
};

export const sanitizeFilename = (filename: string): string => {
  // Remove or replace potentially dangerous characters
  return filename
    .replace(/[^a-zA-Z0-9._-]/g, '_')
    .replace(/_{2,}/g, '_')
    .replace(/^_+|_+$/g, '');
};

export const validateFhirResource = (resource: any): {
  valid: boolean;
  message: string;
  resourceType?: string;
} => {
  if (!resource || typeof resource !== 'object') {
    return {
      valid: false,
      message: 'Resource must be a valid JSON object',
    };
  }

  if (!resource.resourceType) {
    return {
      valid: false,
      message: 'Resource must have a resourceType property',
    };
  }

  if (typeof resource.resourceType !== 'string') {
    return {
      valid: false,
      message: 'resourceType must be a string',
    };
  }

  // Basic FHIR resource type validation
  const validResourceTypes = [
    'Patient', 'Practitioner', 'Organization', 'Location', 'Device',
    'Observation', 'Condition', 'Procedure', 'MedicationRequest', 'DiagnosticReport',
    'Encounter', 'Appointment', 'Schedule', 'Slot', 'Bundle', 'Composition',
    'DocumentReference', 'MessageHeader', 'OperationOutcome', 'Parameters',
    // Add more as needed
  ];

  const isKnownResourceType = validResourceTypes.includes(resource.resourceType);
  
  return {
    valid: true,
    message: isKnownResourceType 
      ? `Valid ${resource.resourceType} resource` 
      : `Unknown resource type: ${resource.resourceType}`,
    resourceType: resource.resourceType,
  };
};