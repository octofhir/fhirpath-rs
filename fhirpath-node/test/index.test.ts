import { FhirPathEngine, getEngineInfo, exists } from '../index';

// Sample FHIR resource for testing
const patientResource = JSON.stringify({
  resourceType: 'Patient',
  id: 'example',
  name: [
    {
      use: 'official',
      family: 'Smith',
      given: ['John', 'Jacob']
    }
  ],
  gender: 'male',
  birthDate: '1974-12-25'
});

describe('FHIRPath Node.js Bindings', () => {
  let engine: FhirPathEngine;

  beforeEach(() => {
    engine = new FhirPathEngine();
  });

  test('should create an engine instance', () => {
    expect(engine).toBeInstanceOf(FhirPathEngine);
  });

  test('should return engine info', () => {
    const info = getEngineInfo();
    expect(info).toContain('FHIRPath Rust Engine');
  });

  test('should return engine version', () => {
    const version = engine.version();
    expect(version).toContain('FHIRPath Engine v');
  });

  test('should validate a valid FHIRPath expression', () => {
    const isValid = engine.validate('Patient.name.given');
    expect(isValid).toBe(true);
  });

  test('should invalidate an invalid FHIRPath expression', () => {
    const isValid = engine.validate('Patient.name.[');
    expect(isValid).toBe(false);
  });

  test('should evaluate a FHIRPath expression', () => {
    const result = engine.evaluate('Patient.name.given', patientResource);
    // Parse the result as JSON to check its structure
    const parsedResult = JSON.parse(result);
    // The result should be an array with two elements: "John" and "Jacob"
    expect(Array.isArray(parsedResult)).toBe(true);
    expect(parsedResult).toContain('John');
    expect(parsedResult).toContain('Jacob');
  });

  test('should evaluate a FHIRPath expression asynchronously', async () => {
    const result = await engine.evaluateAsync('Patient.name.given', patientResource);
    // Parse the result as JSON to check its structure
    const parsedResult = JSON.parse(result);
    // The result should be an array with two elements: "John" and "Jacob"
    expect(Array.isArray(parsedResult)).toBe(true);
    expect(parsedResult).toContain('John');
    expect(parsedResult).toContain('Jacob');
  });

  test('should check if expression returns results using exists function', () => {
    // Test with expression that returns results
    const hasName = exists('Patient.name.given', patientResource);
    expect(hasName).toBe(true);

    // Test with expression that returns no results
    const hasEmail = exists('Patient.telecom', patientResource);
    expect(hasEmail).toBe(false);

    // Test with expression that returns single result
    const hasGender = exists('Patient.gender', patientResource);
    expect(hasGender).toBe(true);
  });

  test('should handle async evaluation errors gracefully', async () => {
    // Test with invalid resource JSON
    await expect(engine.evaluateAsync('Patient.name', 'invalid json')).rejects.toThrow();
  });

  test('should handle exists function errors gracefully', () => {
    // Test with invalid resource JSON
    expect(() => exists('Patient.name', 'invalid json')).toThrow();
  });
});
