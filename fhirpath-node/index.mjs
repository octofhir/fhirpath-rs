/* ESM wrapper for FHIRPath Node.js bindings */

import { createRequire } from 'module';
const require = createRequire(import.meta.url);

// Import the CommonJS module
const binding = require('./index.cjs');

// Re-export as ESM
export const FhirPathEngine = binding.FhirPathEngine;
export const getEngineInfo = binding.getEngineInfo;
export const exists = binding.exists;

// Default export for convenience
export default {
  FhirPathEngine,
  getEngineInfo,
  exists
};
